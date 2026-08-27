//! Every command that is handled has something that raises it.
//!
//! The failure this catches has now happened three times. A command id is
//! allocated, a handler is written for it, comments elsewhere describe what it
//! does, the documentation lists its shortcut, and nothing in the application
//! ever raises the event. Pressing the key does nothing, silently, which is
//! indistinguishable from a shortcut that works and lands somewhere quiet.
//!
//! `F6` was like that for as long as it had been documented: a handler that
//! moved focus between panes, an id, a test guarding the id against
//! collisions, three comments treating it as working, an injected script in
//! the message preview that posted back to the host when somebody pressed it,
//! and no menu item and no accelerator. `ID_NEW_CALENDAR` was like that too,
//! and was removed rather than wired.
//!
//! A compiler cannot see this: both halves are live code and the id is used at
//! both ends. Only asking "and who sends it?" finds it.
//!
//! What none of the menu and shortcut checks in this file can see, said once
//! here rather than nine times below. A key bound to a menu item proves Windows
//! will dispatch it and the handler will be entered. It says nothing about what
//! the handler then does, whether the right thing is on the screen when it
//! runs, or whether anybody hears the answer. Every one of them can be green
//! with the command doing the wrong thing every time it is pressed.
//!
//! The same holds for the handler checks further down, which read the source of
//! the main window because reaching those paths needs a window, an account with
//! stored credentials and a mail server. Each says what it is blind to in its
//! own words, because what stays green while the behaviour goes is different
//! for each of them.

use std::fs;
use std::path::{Path, PathBuf};

/// The presentation sources, which is where commands are raised and handled.
fn sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("src/presentation"), &mut found);
    found
}

fn collect(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

/// The command id following `marker`, if the next thing there is one.
///
/// Whitespace is skipped first. `rustfmt` puts the id on the line after
/// `append_item(` whenever the call is long enough to wrap, which is most of
/// them, and the first version of this read the newline, found nothing, and
/// reported fifty-six live menu items as dead.
fn names_after(text: &str, marker: &str) -> Vec<String> {
    text.match_indices(marker)
        .filter_map(|(at, _)| {
            let rest = text[at + marker.len()..].trim_start();
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                .collect();
            // Ours are SCREAMING_SNAKE and start with ID_. Anything else that
            // happens to follow the marker is not a command id.
            (name.starts_with("ID_") && name.len() > 3).then_some(name)
        })
        .collect()
}

#[test]
fn test_every_handled_command_has_something_that_raises_it() {
    let mut handled: Vec<(PathBuf, String)> = Vec::new();
    let mut raised: Vec<String> = Vec::new();

    for path in sources() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };

        // A handler arm: `_ if id == ID_THING =>` or `r if r == ID_THING =>`.
        for name in names_after(&text, "id == ") {
            handled.push((path.clone(), name));
        }
        for name in names_after(&text, "r == ") {
            handled.push((path.clone(), name));
        }

        // Something that can raise it: a menu item, a toolbar tool, or a
        // control built with the id.
        for marker in [
            "append_item(",
            "append_check_item(",
            "append_radio_item(",
            // The bare form, used when a menu is built item by item rather
            // than in one builder chain: a context menu whose entries depend
            // on what was clicked, or a submenu built from a list. Matched on
            // the method rather than on one receiver's name, because the
            // receiver is whatever the menu is called.
            ".append(",
            "with_id(",
            "add_tool(",
            "end_modal(",
        ] {
            raised.extend(names_after(&text, marker));
        }

        // The context menus raise their ids through a mapping rather than by
        // naming one at the point the menu is built, so the ids appear only
        // as the right hand side of that match.
        if path.ends_with("wx_context_menu.rs") {
            raised.extend(names_after(&text, "=> "));
        }

        // The notification area's menu is described as data in `wx_tray.rs`
        // and filled in from `wx_app.rs`, so neither file names an id beside a
        // menu call. The ids are the fields of `TrayCommands` where it is
        // built, which is the one place they are written down.
        if path.ends_with("wx_app.rs") {
            for field in [
                "open_window:",
                "new_message:",
                "check_mail:",
                "all_inboxes:",
                "quit:",
            ] {
                raised.extend(names_after(&text, field));
            }
        }
    }

    let mut dead: Vec<String> = handled
        .iter()
        .filter(|(_, name)| !raised.contains(name))
        .filter(|(_, name)| !is_a_standard_id(name))
        .map(|(_, name)| name.clone())
        .collect();
    dead.sort();
    dead.dedup();

    let unexpected: Vec<&String> = dead
        .iter()
        .filter(|d| !KNOWN_DEAD.contains(&d.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{} command handlers nothing raises:\n  {}\n\nEither give it a menu item, a \
         toolbar button or a shortcut, or delete the handler. If it is deliberate for now, \
         add it to KNOWN_DEAD in this file with a reason.",
        unexpected.len(),
        unexpected
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );

    // The list has to shrink, not rot. An id that has been wired since it was
    // written down should come off, or the next person reads the list and
    // believes something is broken that is not.
    let fixed: Vec<&&str> = KNOWN_DEAD
        .iter()
        .filter(|known| !dead.contains(&known.to_string()))
        .collect();
    assert!(
        fixed.is_empty(),
        "these are wired now and should come off KNOWN_DEAD: {fixed:?}"
    );
}

/// The same question asked the other way round: does pressing it do anything?
///
/// The check above proves every handler can be reached. It says nothing about a
/// control that can be reached and reaches nothing, which is the more common
/// mistake and the worse one: a handler nobody can raise is invisible, while a
/// button that answers no key looks broken to the person pressing it.
///
/// Attach File was built, given an id, given a label with a mnemonic, put in a
/// dialog and listed in a test about id ranges. Nothing anywhere handled it. It
/// was reported as "add attachments does not work", which is exactly what it
/// was.
#[test]
fn test_every_command_something_raises_is_handled() {
    let mut handled: Vec<String> = Vec::new();
    let mut raised: Vec<(PathBuf, String)> = Vec::new();

    for path in sources() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for marker in [
            "id == ", "r == ",
            // A block of ids handled by a range, which is how a menu built
            // from a list of unknown length is answered. The id it starts at
            // stands for the block: the check cannot enumerate the rest, and
            // the first one going unhandled is what a broken block looks like.
            "id >= ",
        ] {
            handled.extend(names_after(&text, marker));
        }
        // Bound directly to the control rather than through a command id, which
        // is just as good a way of making a button do something.
        // A dialog button is handled by returning its id, which the caller
        // then matches on. That is a handler, just not an `id ==` arm.
        handled.extend(names_after(&text, "end_modal("));
        // A guard of the form `SOME_IDS.contains(&id)` handles every id in
        // that list. The label keys are built by counting rather than by
        // naming, so without this neither half of the pair was visible here
        // and a whole submenu could have been unwired unnoticed.
        handled.extend(ids_in_lists_used_with_contains(&text));
        for marker in [
            "append_item(",
            "append_check_item(",
            "add_tool(",
            // A menu built item by item rather than in a builder chain, which
            // is how a submenu with a variable number of entries is made.
            ".append(",
        ] {
            raised.extend(
                names_after(&text, marker)
                    .into_iter()
                    .map(|name| (path.clone(), name)),
            );
        }
    }

    let mut deaf: Vec<String> = raised
        .iter()
        .filter(|(_, name)| !handled.contains(name))
        .filter(|(_, name)| !is_a_standard_id(name))
        .map(|(_, name)| name.clone())
        .collect();
    deaf.sort();
    deaf.dedup();

    let unexpected: Vec<&String> = deaf
        .iter()
        .filter(|d| !KNOWN_DEAF.contains(&d.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{} commands can be invoked and are handled by nothing:\n  {}\n\nGive each one a \
         handler, or take the control away. A control that answers no key is worse than \
         one that is not there, because somebody will press it and conclude the \
         application is broken.",
        unexpected.len(),
        unexpected
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );

    let fixed: Vec<&&str> = KNOWN_DEAF
        .iter()
        .filter(|known| !deaf.contains(&known.to_string()))
        .collect();
    assert!(
        fixed.is_empty(),
        "these are handled now and should come off KNOWN_DEAF: {fixed:?}"
    );
}

/// Commands that can be invoked and do nothing, awaiting a decision.
///
/// Same rule as `KNOWN_DEAD`: it shrinks, it does not rot.
const KNOWN_DEAF: &[&str] = &[
    // Not a command at all. It is the base of the range the formatting run is
    // allocated ids from, and the handler matches the whole range rather than
    // this one number, so it is handled and this test cannot see that.
    "ID_FORMAT_FIRST",
    // Reader, Go menu. Moving between messages works, but only from the key:
    // the movement lives inside the text control's own key handler and the
    // reader's menu handler does not know about it. So the menu says Ctrl+Down
    // and choosing the item does nothing, which is worse than the key being
    // undocumented, because it looks like the feature is broken rather than
    // hidden.
    "ID_NEXT_LANDMARK",
    "ID_PREV_LANDMARK",
    // Reader, Go menu. There is no find in the reader at all. The menu item and
    // its Ctrl+F were written for something that was never built.
    "ID_READER_FIND",
    // View menu. Nothing handles it, so the item is a no-op.
    "ID_THREAD_VIEW",
];

/// Handlers that are known to be unreachable, and are waiting on a decision.
///
/// Empty, and it should stay that way. Eleven were listed here on the day this
/// test was written, all the same shape as the F6 bug: a handler, an id, no
/// way to raise it. Ten of them are now menu items and the eleventh was a
/// second route to a dialog that already had a button.
///
/// If something has to go in here, put the reason next to it and a task
/// number. A name on this list with no explanation is how the list stops
/// meaning anything.
const KNOWN_DEAD: [&str; 0] = [];

/// Whether this is one of wxWidgets' own ids rather than one of ours.
///
/// `ID_YES` is what a message dialog returns, not a command this application
/// raises, so nothing here should be looking for the menu item that sends it.
fn is_a_standard_id(name: &str) -> bool {
    matches!(
        name,
        "ID_YES" | "ID_NO" | "ID_OK" | "ID_CANCEL" | "ID_ANY" | "ID_HIGHEST" | "ID_APPLY"
    )
}

#[test]
fn test_the_check_found_the_commands_at_all() {
    // Without this, a marker string that stopped matching would leave the
    // test above passing on two empty lists and reporting everything wired.
    let mut handled = 0;
    let mut raised = 0;
    for path in sources() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        handled += names_after(&text, "id == ").len();
        raised += names_after(&text, "append_item(").len();
    }

    assert!(handled > 30, "only {handled} handlers found");
    assert!(raised > 20, "only {raised} menu items found");
}

#[test]
fn test_no_two_menu_items_claim_the_same_shortcut() {
    // What this cannot see: whether any of these menus is ever built. It
    // reads labels out of the source, so a menu never added to the bar, or
    // one the operating system takes the key from first, keeps this green.
    // Whether a key works is a keyboard on a running window.
    // wxWidgets builds the accelerator table from the menu labels, and when
    // two items claim one key only the first gets it. The other looks bound,
    // reads as bound to a screen reader announcing the menu, and does
    // nothing, which is the same silent failure as a shortcut nothing raises.
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    let mut claims: Vec<(String, String)> = Vec::new();
    for label in app.split('"').skip(1).step_by(2) {
        // A menu label is "Text\tShortcut". The tab is written as an escape
        // in the source, so it is two characters here rather than one.
        let Some((text, key)) = label.split_once("\\t") else {
            continue;
        };
        // Menu labels only. Anything else with a tab escape in it is a format
        // string or a fixture.
        if key.is_empty() || key.contains(' ') || text.len() > 40 {
            continue;
        }
        // The New submenu writes its keys with a placeholder, filled in from
        // ItemKind at build time. Those are added below from the same source
        // the menu reads, rather than skipped: one of them is why this test
        // exists.
        if key.contains('{') {
            continue;
        }
        claims.push((key.to_string(), text.to_string()));
    }

    // The computed half. New Reminder is Ctrl+Shift+D, and Open Draft was
    // written as Ctrl+Shift+D too, which this caught before it shipped.
    for kind in wixen_mail::application::new_item::ItemKind::ALL {
        claims.push((kind.shortcut().to_string(), format!("New {kind:?}")));
    }

    assert!(
        claims.len() > 20,
        "only {} shortcuts found, so this is not reading the menus",
        claims.len()
    );

    let mut collisions = Vec::new();
    for (at, (key, text)) in claims.iter().enumerate() {
        for (other_key, other_text) in claims.iter().skip(at + 1) {
            if key == other_key {
                collisions.push(format!(
                    "{key} is claimed by both {text:?} and {other_text:?}"
                ));
            }
        }
    }

    assert!(collisions.is_empty(), "{}", collisions.join("\n  "));
}

#[test]
fn test_every_context_menu_line_has_a_handler() {
    // The other direction from the sweep above. That one finds handlers with
    // nothing to raise them; this finds a menu line that raises something
    // nothing handles, which is a line somebody chooses and nothing happens.
    //
    // The context menus are the place this matters most: they are read out
    // one line at a time by somebody who cannot see the panel, so a line that
    // does nothing costs a moment every time it is passed and teaches nothing
    // when it is chosen.
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let mapping =
        fs::read_to_string("src/presentation/wx_context_menu.rs").expect("the context menus");

    // Every id the mapping hands out.
    let mut raised: Vec<String> = names_after(&mapping, "=> ");
    raised.sort();
    raised.dedup();

    assert!(
        raised.len() >= 10,
        "only {} ids mapped, so this is not reading the mapping",
        raised.len()
    );

    let handled = names_after(&app, "id == ");
    let orphans: Vec<&String> = raised.iter().filter(|id| !handled.contains(id)).collect();

    assert!(
        orphans.is_empty(),
        "context menu lines that raise a command nothing handles: {orphans:?}"
    );
}

#[test]
fn test_the_menu_key_is_bound_with_the_numbers_wxwidgets_uses() {
    // What this cannot see: whether the handler runs. It reads the numbers
    // out of the source and checks them against what wxWidgets uses. Bound
    // to a control that never takes focus, this still passes.
    // Both bugs this had, written down as the test that would have found
    // them.
    //
    // The first was binding the context menu event, which is the obvious way
    // and never fires: wxdragon offers it on frames and panels only, and
    // wxWidgets does not hand it up from a native list or tree.
    //
    // The second was using Windows' own number for the Applications key, 93.
    // wxWidgets renumbers every non-character key, so the handler was correct
    // and unreachable. It is WXK_WINDOWS_MENU, 395, and WXK_F10 is 349.
    // Both were found by pressing the key against the running application and
    // watching nothing happen.
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let wiring = app
        .split("fn wire_context_menu")
        .nth(1)
        .expect("the helper that gives a control the menu key");
    let helper: String = wiring.chars().take(1600).collect();

    assert!(
        helper.contains("395"),
        "the Applications key is not WXK_WINDOWS_MENU, so the menu key does nothing"
    );
    assert!(
        helper.contains("349"),
        "Shift+F10 is not bound, and some keyboards have no Applications key"
    );
    assert!(
        helper.contains("shift_down"),
        "F10 is bound without checking Shift, so plain F10 opens the menu"
    );
    assert!(
        helper.contains("KEY_DOWN"),
        "bound to something other than a key press"
    );
}

#[test]
fn test_the_menu_key_is_given_to_every_list_and_tree() {
    // What this cannot see: whether the key reaches any of these controls.
    // It counts bindings in the source. A control that never takes focus,
    // or a binding shadowed by another, keeps this green.
    // Eleven controls hold focus in the main window, and a menu key that
    // works on some of them is worse than one that works on none: it teaches
    // that the key works, and then it does not.
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    // The definition is written `wire_context_menu<W>(`, so it is not one of
    // these. Every match is a control being given the key.
    let bound = app.matches("wire_context_menu(").count();

    // The message list, the mail folder tree, five module lists, and four
    // module sidebars. The reminders sidebar is not among them: it holds
    // buckets rather than containers somebody made, and there is nothing to
    // do to one, so it has no menu rather than an empty one.
    assert_eq!(
        bound, 11,
        "eleven controls should have the menu key, {bound} do"
    );
}

/// A read receipt is filed against the message it is about.
///
/// It was sent with no `In-Reply-To` for one reason only: the list row did not
/// carry the original's `Message-ID`. It does now, so the receipt arrives in
/// the sender's thread instead of as loose mail with a subject line.
///
/// What this cannot see: whether the identifier read off the row is the right
/// one, or whether it is empty. It asks only that nothing files a receipt
/// against nothing and that something on that path reads the original's
/// identifier. A receipt naming the wrong message keeps this green.
#[test]
fn test_a_read_receipt_names_the_message_it_is_about() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    assert!(
        !app.contains("message_id: None,\n            read_at:"),
        "a read receipt still goes out naming no message"
    );
    assert!(
        app.contains("header_message_id"),
        "nothing on the receipt path reads the original's header identifier"
    );
}

/// One function's text, from its signature to the closing brace at column nought.
///
/// Panics when the signature is gone, rather than reading the whole file and
/// quietly passing: a guard that no longer knows where to look is measuring
/// nothing, and that has to fail loudly the moment somebody renames the thing.
fn body_of(source: &str, signature: &str) -> String {
    let at = source.find(signature).unwrap_or_else(|| {
        panic!("{signature} is no longer in this file, so this guard is measuring nothing")
    });
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    rest[..ends].to_string()
}

/// The window asks about a delete rather than deciding one for itself.
///
/// Whether a delete is sent at all is now decided and carried out in one place
/// away from this window, where a test can hold a server and find that it was
/// told nothing. What is left here is the wiring: this window has to ask that
/// code, and it must not word either refusal itself, because a sentence written
/// twice is two answers to one question and they drift.
///
/// What this cannot see: whether the handler is ever reached. Delete could be
/// unbound, the window could send some other command, the whole branch could
/// become unreachable, and this stays green. The behaviour is measured against
/// a server in `application::deleting_at_the_server` and in
/// `application::mail_controller::against_a_server_that_answers`; this only
/// says the window asks that code rather than deciding for itself.
#[test]
fn test_a_delete_with_no_recognised_trash_is_refused_rather_than_sent() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let handler = body_of(&app, "fn spawn_server_change(");

    assert!(
        handler.contains("delete_a_message"),
        "the delete handler works out for itself whether to send a delete, so \
         nothing that runs can say whether a message on an account with no \
         recognised trash reaches the server"
    );
    assert!(
        handler.contains("UIUpdate::CommandRefused"),
        "the refusal goes to the status bar and nowhere else, which from the \
         keyboard is indistinguishable from a key that was never wired up"
    );
    for refusal in ["NO_TRASH_FOLDER_FOUND", "NO_FOLDERS_KNOWN_YET"] {
        assert!(
            !handler.contains(refusal),
            "the window words {refusal} itself again, so the sentence somebody \
             hears and the sentence beside the decision can drift apart"
        );
    }
}

/// The row and the sentence are decided in one place for both handlers.
///
/// They have to agree. A delete or a move whose copy landed and whose original
/// the server then left alone must keep its row, and must say so; the version
/// before this took the row out for every answer that was not a failure and
/// wrote its own sentence a few lines below one that said the same event
/// differently. Two pieces of code answering one question is how the list and
/// the server came to disagree in the first place.
///
/// What this cannot see: whether the one place gives the right answer, or
/// whether what it gives back is used. It asks that the handler calls it and
/// words no outcome itself. A handler that calls it, ignores the answer and
/// takes the row out every time keeps this green.
///
/// Nor can it see a delete sentence written anywhere else in that file. It
/// reads one handler body, and a second spelling of "Deleted" sat a few dozen
/// lines above the one it reads, on a method of the type the handler is handed.
/// `test_only_the_delete_owner_words_what_a_delete_did`, beside that code,
/// reads the whole of the file with only its tests taken out, and
/// `test_nothing_that_words_its_own_outcome_here_says_what_a_delete_said` asks
/// the same question of the values rather than of the text.
#[test]
fn test_the_delete_and_move_handlers_ask_one_place_what_to_say_and_what_to_do() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    for (signature, asks) in [
        ("fn spawn_server_change(", "server_delete::after_a_delete"),
        ("fn spawn_folder_move(", "server_delete::after_a_move"),
    ] {
        let handler = body_of(&app, signature);
        assert!(
            handler.contains(asks),
            "{signature} decides for itself what happened, so the sentence and \
             the row can drift apart again"
        );
        for word in ["Deleted", "Moved to", "was not deleted", "was not moved"] {
            assert!(
                !handler.contains(&format!("{word}: {{"))
                    && !handler.contains(&format!("{word} {{")),
                "{signature} still words an outcome itself: {word}"
            );
        }
    }
}

/// A draft the server would not take is said, not written to a log.
///
/// Both halves of filing a draft at the server used to fail into
/// `tracing::warn!` inside a background task that held no way of speaking. The
/// old copy was removed before the new one was offered, so a refused save left
/// no copy on the server at all, and the only record of it was a line in a file
/// nobody reads. The person went on writing, believing their other devices had
/// it.
///
/// What this cannot see: whether the sentence is ever sent. It asks that the
/// handler works the answer out and that the routine below it holds the call
/// that speaks. A handler that works the answer out and drops it on a branch
/// nothing takes keeps this green, and so does one that speaks a sentence
/// saying the opposite of what happened.
#[test]
fn test_a_draft_the_server_would_not_take_reaches_the_person() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let handler = body_of(&app, "fn spawn_draft_append(");

    assert!(
        handler.contains("replace_the_filed_copy"),
        "the draft handler decides the order for itself again, and the order is \
         what keeps a copy on the server at every moment"
    );
    assert!(
        handler.contains("needs_saying") && handler.contains("what_happened"),
        "the answer is worked out and dropped, which reaches nobody"
    );
    assert!(
        !handler.contains("tracing::warn!"),
        "a failure to file the draft goes to a log and nowhere else again"
    );
    assert!(
        body_of(&app, "fn file_draft_copy(").contains("send_status"),
        "a draft that could not be put in the folder on this computer is still \
         only logged"
    );
}

/// The send loop files its copies through one session, and closes it.
///
/// Filing a copy used to sign in to the mail server and disconnect again around
/// every single message, so a queue of fifty was fifty sign-ins and some
/// providers turn that down. The session is opened once for the queue now.
///
/// One opened per queue and never closed is worse than what it replaced: a
/// connection left held at the server after every send, and a provider counts
/// those.
///
/// What this cannot see: it reads the send loop as text. It says the three
/// calls are written there. It cannot say the session really carried a copy,
/// which is measured against a server in `application::mail_controller`, and it
/// cannot say the close is reached on every path out of the loop.
#[test]
fn test_the_send_loop_files_its_copies_through_one_session_and_closes_it() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let loop_body = body_of(&app, "fn flush_outbox(");

    assert!(
        loop_body.contains("a_session_for"),
        "the send loop no longer opens one session for the queue"
    );
    assert!(
        loop_body.contains("offer_through"),
        "the copies no longer go through the queue's own session"
    );
    assert!(
        loop_body.contains("close()"),
        "the session the queue opened is never closed, which leaves a connection \
         held at the server after every send"
    );
    assert!(
        !loop_body.contains("ServerCopy {"),
        "the send loop builds the adapter itself again, which is how it came to \
         sign in once per message"
    );
}

/// The calendar sync asks whether anything can send what is still waiting.
///
/// The sweep is a plain function over the account's rows, so it is tested
/// properly in `application::calendar`. What no test there can say is whether
/// the sync calls it: reaching the sync means a window, an account and a
/// runtime, and it is the calling that was missing before. A change nothing
/// can send waits for ever and is looked at again on every sync, and the only
/// place anybody learns that is the summary this feeds.
///
/// What this cannot see: whether the summary it is added to is ever read out.
/// It asks that the sweep is called and that its answer is kept. A sync that
/// keeps the answer in a list nothing reports keeps this green.
#[test]
fn test_the_calendar_sync_says_what_nothing_can_send() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    assert!(
        app.contains("changes_nothing_can_send(&cache, aid)"),
        "the calendar sync never asks what nothing can send, so a change that \
         waits for ever is passed over in silence on every sync"
    );
    assert!(
        app.contains("Ok(said) => total_cannot_be_saved.extend(said)"),
        "the answer is asked for and dropped, which reaches nobody"
    );
}

/// Importing and exporting contact cards name the account being looked at.
///
/// Both named a fixed word instead. Every other part of the contacts module
/// works on `active_account_id`: the list is drawn from it, the sync runs for
/// it, and a contact belongs to it. So with any account signed in, a file of
/// cards was read into a corner of the database nothing reads, the count said
/// the contacts had arrived, the list did not change, and no sync ever saw
/// them. Export had the matching half and wrote out an empty file.
///
/// Read from the source because reaching either handler needs a window, a
/// dialog somebody clicks through and a folder of files. What is asked here is
/// only that neither call passes a literal, which is the whole of the defect.
///
/// What this cannot see: whether the value handed in is the account on the
/// screen. It asks that a name rather than a quoted word is passed. A handler
/// that passes the wrong account, or one that is empty, keeps this green.
#[test]
fn test_importing_and_exporting_cards_use_the_account_being_looked_at() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    for call in ["import_contacts_from_vcard(", "export_contacts_to_vcard("] {
        for (at, _) in app.match_indices(call) {
            let handed = &app[at + call.len()..];
            assert!(
                !handed.starts_with('"'),
                "{call} is handed a fixed account, so what it reads or writes \
                 is not the account whose contacts are on the screen"
            );
        }
    }
    assert!(
        app.contains("import_contacts_from_vcard(&account_id, &data)"),
        "nothing imports cards into the account being looked at"
    );
    assert!(
        app.contains("export_contacts_to_vcard(&account_id)"),
        "nothing exports the cards of the account being looked at"
    );
}

/// The composer asks whether a signature was wanted before it puts one on.
///
/// The compose tab has offered this answer for as long as the settings screen
/// has existed. It was hard-set to ticked, handed back by nothing and saved by
/// nothing, so a screen reader announced a setting that was not there and
/// every message got a signature either way.
///
/// The rule itself is `application::sign_off::opens_with` and is tested there.
/// What no test there can say is whether the window asks it, and asking is the
/// half that was missing: reaching the composer needs a window, an account and
/// a runtime.
///
/// What this cannot see: whether the setting read is the one the settings
/// screen writes, or whether the composer ever opens. It asks that three calls
/// are written and that the composer is handed the answer of the third.
#[test]
fn test_the_composer_asks_whether_a_signature_was_wanted() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    assert!(
        app.contains("cfg.add_signature_automatically"),
        "nothing on the compose path reads the setting, so a signature goes on \
         whatever the settings screen was told"
    );
    assert!(
        app.contains("sign_off::opens_with(sign_it, &stored_signature)"),
        "the setting is read and nothing decides the signature with it"
    );
    assert!(
        app.contains("preview_first,\n        signature,\n        autosave,"),
        "the composer is handed something other than the signature that answer \
         decided, so deciding it changes nothing"
    );
}

/// The account dialog asks for the name recipients see.
///
/// This pins that the code asks for the right thing. It can never mean a
/// screen reader heard it: only a run with NVDA says that.
///
/// The dialog now holds three boxes that look interchangeable and are not.
/// "Account Name" is the label somebody gave the account, usually "Work".
/// "Username" is what signs in to the server. This one is the name that goes
/// in front of their address on every message they send, and its label says
/// what happens rather than naming a header.
#[test]
fn test_the_account_dialog_asks_for_the_name_recipients_see() {
    // What this cannot see: whether the dialog is ever opened, and whether
    // the label is heard. It reads the source for the field and its label.
    // Only a run with a screen reader says the second.
    let dialog =
        fs::read_to_string("src/presentation/wx_account_manager.rs").expect("the account dialog");

    assert!(
        dialog.contains("The na&me people see when your mail arrives:"),
        "the account dialog does not ask for the name recipients see"
    );
    assert!(
        dialog.contains("sender_name:"),
        "the account dialog asks for a name and does not put it on the account"
    );
}

#[test]
fn test_the_key_that_replies_to_the_author_alone_is_bound_to_a_menu_item() {
    // What this cannot see: whether pressing the key replies to anybody. It
    // reads the accelerator off the menu item in the source. The handler
    // could be gone, or reply to everybody, and this stays green.
    // Replying to one person on a mailing list is the command that stops a
    // private answer reaching two thousand strangers, and it had a handler, a
    // toolbar button and three lines in the shortcuts document while the key
    // was bound to nothing. Windows dispatches menu accelerators, so a command
    // with no menu item is a command with no key, and a toolbar button is the
    // least reachable control on the window for the people this is for.
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    let on_the_menu = app.match_indices("ID_REPLY_SENDER").any(|(at, _)| {
        let after = &app[at..(at + 200).min(app.len())];
        after.contains(r"\tAlt+Shift+R")
    });
    assert!(
        on_the_menu,
        "nothing gives ID_REPLY_SENDER the Alt+Shift+R accelerator, so the key does nothing"
    );
}

#[test]
fn test_f6_and_shift_f6_reach_the_pane_handler() {
    // What this cannot see: whether focus moves. It reads the binding out of
    // the source. The handler could move focus nowhere, or somewhere that is
    // never announced, and this passes either way.
    // The specific one. Named separately from the sweep above because it is
    // the shortcut somebody uses to get out of a pane they are stuck in, and
    // "it is in the general test" is how it went unnoticed the first time.
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    assert!(
        app.contains(r"\tF6"),
        "nothing carries the F6 accelerator, so F6 does nothing"
    );
    assert!(
        app.contains(r"\tShift+F6"),
        "nothing carries the Shift+F6 accelerator"
    );
    assert!(
        app.contains("id == ID_CYCLE_PANES"),
        "the F6 command is raised and not handled"
    );
}

/// Every key the menus bind is written in the shortcuts document, saying the
/// same thing.
///
/// The document is the promise. A reader who cannot see the menu bar finds out
/// what a key does by reading it, so a key written there that nothing binds is
/// a key somebody presses, hears nothing, and concludes is broken. It said
/// `Ctrl+]` for the next unread message in two places while the menu bound
/// `Ctrl+U`, and the project rule that the document is updated in the same
/// commit as the shortcut had nothing checking it.
///
/// Menus are the source of truth, because they are what Windows dispatches.
#[test]
fn test_the_shortcuts_document_and_the_menus_agree() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let doc = fs::read_to_string("docs/KEYBOARD_SHORTCUTS.md").expect("the shortcuts document");

    let mut missing = Vec::new();
    for key in accelerators(&app) {
        // Written as `Ctrl+U` in the menu and as `` `Ctrl+U` `` in the table.
        if !doc.contains(&key) {
            missing.push(key);
        }
    }

    assert!(
        missing.is_empty(),
        "these keys are bound in the menus and not in docs/KEYBOARD_SHORTCUTS.md: {missing:?}"
    );
}

/// Every key the shortcuts document names is mentioned somewhere in the code.
///
/// The direction that went wrong. The document said `Ctrl+]` and `Ctrl+[` for
/// the next and previous unread message, in a table and again in a tips
/// section, and nothing anywhere bound either of them: the menu had `Ctrl+U`
/// and `Ctrl+Shift+U` all along. Somebody reading the document pressed a key,
/// heard nothing, and had no way to tell that from a broken application.
///
/// Where it can, it asks the code for its keys rather than reading them out of
/// the source text: the editor's formatting keys and the new item keys are
/// built from tables that spell them, so those tables are the answer. The rest
/// falls back to whether the key appears in the source at all, because a key
/// can be bound by a menu accelerator, by a handler matching a code, or by a
/// mnemonic in a label, and this should not have to know which.
#[test]
fn test_the_shortcuts_document_names_no_key_the_code_has_never_heard_of() {
    // What this cannot see: whether any key in the document works. The
    // document is the thing being checked, against the source rather than
    // against a running window, so a key both places name and neither binds
    // is agreement about nothing.
    let doc = fs::read_to_string("docs/KEYBOARD_SHORTCUTS.md").expect("the shortcuts document");
    let code: String = sources()
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .collect();

    // The keys the code states outright, which no amount of reading the source
    // as text would find: all three are assembled from parts at run time, so
    // they are kept apart from the source text and matched by name.
    let mut stated = String::new();
    for format in wixen_mail::presentation::editor_document::Format::ALL {
        stated.push_str(&format!("\n{}", format.shortcut()));
    }
    for kind in wixen_mail::application::new_item::ItemKind::ALL {
        stated.push_str(&format!("\n{}", kind.shortcut()));
    }
    // The label keys, which the menu builds by counting rather than by naming.
    for number in 1..=wixen_mail::application::tagging::REACHABLE_BY_KEY {
        stated.push_str(&format!("\nCtrl+{number}"));
    }

    let mut invented = Vec::new();
    for key in documented_combinations(&doc) {
        if bound_somewhere(&key, &code, &stated) || invented.contains(&key) {
            continue;
        }
        invented.push(key);
    }
    invented.retain(|key| !bound_by_a_handler_rather_than_a_menu().contains(&key.as_str()));

    assert!(
        invented.is_empty(),
        "docs/KEYBOARD_SHORTCUTS.md names these keys and nothing in src/presentation binds them: {invented:?}"
    );
}

/// Whether anything in the source binds this key.
///
/// `Alt` and a single letter is a mnemonic, which is not written anywhere as
/// `Alt+S`: it is the ampersand in a label, so `&Save Note` is what binds
/// Alt+S. Matched without case because a label capitalises where it likes.
///
/// Anything else has to be an accelerator, which on Windows means a menu item
/// whose label ends in a tab and the key. Merely appearing in the source is not
/// enough and used to be: `Alt+Shift+R` was in the document, in a toolbar
/// tooltip that read "(Alt+Shift+R)", and bound to nothing, so the key did
/// nothing and this check said it was fine. That is the second documented key
/// to be dead, so the check now asks the question it was written for.
///
/// The key has to end where the accelerator ends. Matching it as a prefix let a
/// third documented key be dead: the document said `Ctrl+D` opens a draft, the
/// menu binds `Ctrl+Shift+O`, and `Ctrl+D` matched the `Ctrl+Down` on the
/// reader's Next Message item. `Ctrl+Shift+S` sitting inside `Ctrl+Shift+Space`
/// would be the same mistake in the other direction.
fn bound_somewhere(key: &str, code: &str, stated: &str) -> bool {
    if let Some(letter) = key.strip_prefix("Alt+")
        && letter.chars().count() == 1
        && letter.chars().all(|c| c.is_ascii_alphabetic())
    {
        let lower = format!("&{}", letter.to_ascii_lowercase());
        let upper = format!("&{}", letter.to_ascii_uppercase());
        return code.contains(&lower) || code.contains(&upper);
    }
    // `\t` as two characters, because that is how it is written in the source
    // of a menu label: `"Reply &All\tCtrl+Shift+R"`. A few labels carry a real
    // tab instead, which is the same thing to Windows.
    whole_key_appears_after(r"\t", key, code)
        || whole_key_appears_after("\t", key, code)
        || whole_key_appears_after("\n", key, stated)
}

/// Whether `key` follows `opener` somewhere in `text` and ends there.
///
/// Ends there means the next character is not one a key name can carry, so
/// `Ctrl+D` does not match the start of `Ctrl+Down`.
fn whole_key_appears_after(opener: &str, key: &str, text: &str) -> bool {
    let looking_for = format!("{opener}{key}");
    let mut from = 0;
    while let Some(at) = text[from..].find(&looking_for) {
        let ends = from + at + looking_for.len();
        match text[ends..].chars().next() {
            None => return true,
            Some(next) if !next.is_ascii_alphanumeric() && next != '+' => return true,
            _ => from = ends,
        }
    }
    false
}

/// The keys nothing binds with a menu accelerator, and what does bind them.
///
/// Every one of these was checked by hand and works. They are here because
/// there is no way to read a key code out of the source text and know which
/// key it is, and the alternative, accepting any key that appears anywhere,
/// is what let three documented keys be dead at once.
///
/// Adding to this list means saying where the key is bound. A key that only
/// appears in a tooltip does not belong here.
fn bound_by_a_handler_rather_than_a_menu() -> Vec<&'static str> {
    vec![
        // The notebook's own, bound by wxWidgets, named nowhere in this source.
        "Ctrl+Tab",
        "Ctrl+Shift+Tab",
        // The editor's key handler, which posts Send back out of the document.
        // src/presentation/editor_document.rs.
        "Ctrl+Enter",
        // The same handler, posting back a request to move to the toolbar.
        // A web view keeps every key once it has focus, so the page binds the
        // ones that have to leave and hands them over; there is no wx-side
        // binding for this source to find.
        "Ctrl+\\",
        // The column list's key handler, which moves the selected column.
        // src/presentation/wx_columns.rs, the `key.alt_down()` arm.
        "Alt+Up",
        "Alt+Down",
    ]
}

/// Every backticked `Ctrl+...` or `Alt+...` combination in the document.
///
/// Only the modified ones. A bare `Space` or `Home` in the document is a key
/// this reads no meaning into, and looking for the word "Space" in the source
/// would match anything.
///
/// Modifiers on their own are not combinations either. The document writes "the
/// six `Ctrl+Shift` keys" and "the heading keys use `Ctrl+Alt`", which name a
/// family rather than a key, and there is nothing for a menu to bind.
fn documented_combinations(doc: &str) -> Vec<String> {
    const MODIFIERS: [&str; 3] = ["Ctrl", "Alt", "Shift"];
    let mut found = Vec::new();
    for piece in doc.split('`').skip(1).step_by(2) {
        let piece = piece.trim();
        let modified = piece.starts_with("Ctrl+") || piece.starts_with("Alt+");
        let all_modifiers = piece.split('+').all(|part| MODIFIERS.contains(&part));
        // One combination, not a sentence that happens to contain one, and not
        // a sequence like "Ctrl+N, M" whose halves are pressed separately.
        if modified
            && !all_modifiers
            && !piece.contains(' ')
            && !piece.contains(',')
            && !found.contains(&piece.to_string())
        {
            found.push(piece.to_string());
        }
    }
    found
}

/// Every accelerator a menu item carries, as it is written after the tab.
///
/// A label built with `format!` carries a second key chosen at run time after
/// the fixed one, as in `"&Message\tCtrl+N, {}"`. Stopping at the comma takes
/// the fixed half and leaves the placeholder, which is not a key and is not
/// what the document would spell out.
fn accelerators(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (start, _) in source.match_indices(r"\t") {
        let rest = &source[start + 2..];
        let end = rest.find(['"', '\\', ',', '{']).unwrap_or(rest.len());
        let key = rest[..end].trim().to_string();
        if !key.is_empty() && !found.contains(&key) {
            found.push(key);
        }
    }
    found
}

/// Every command id in a list that a match guard tests with `contains`.
///
/// `_ if SOME_IDS.contains(&id) =>` handles all of them, and the plain search
/// for `id == ID_THING` cannot see that. Written generally rather than for the
/// one list that needed it, so the next one is covered without a change here.
fn ids_in_lists_used_with_contains(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (at, _) in text.match_indices(".contains(&id)") {
        // The list's name is the identifier just before the dot.
        let before = &text[..at];
        let name: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if name.is_empty() {
            continue;
        }
        let Some(declared) = text.find(&format!("const {name}:")) else {
            continue;
        };
        let Some(opens) = text[declared..].find('[').map(|at| declared + at) else {
            continue;
        };
        // The second `[` opens the values; the first is the array's type.
        let Some(values) = text[opens + 1..].find('[').map(|at| opens + 1 + at) else {
            continue;
        };
        let Some(closes) = text[values..].find(']').map(|at| values + at) else {
            continue;
        };
        for piece in text[values + 1..closes].split(',') {
            let id = piece.trim();
            if id.starts_with("ID_") {
                found.push(id.to_string());
            }
        }
    }
    found
}

/// Choosing a row in the contacts sidebar changes what the contact list shows.
///
/// The tree has said "Team A, 3 people" and answered nothing when a row was
/// clicked since groups were first built: nothing read the selection, so a
/// group somebody made and put people in never once narrowed the list it
/// sits beside.
///
/// What this cannot see: whether the handler resolves the row clicked to the
/// right group, whether the list visibly narrows, or whether a screen reader
/// hears about it. `application::contact_groups::belongs` is where that
/// decision is made and tested on its own; this only says a selection
/// handler exists to call it from.
#[test]
fn test_choosing_a_row_in_the_contacts_sidebar_changes_what_the_list_shows() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    assert!(
        app.contains("contacts_sb.tree.on_selection_changed"),
        "nothing reads a selection made in the contacts sidebar tree"
    );
}

/// Typing in the contacts search box changes what the contact list shows.
///
/// The search box read the query typed into it and posted "Searching
/// contacts: ..." to the status bar and nothing else: `state.contacts`,
/// `state.all_contacts`, and the list's own row count were all untouched, so
/// typing there announced that a search was starting and narrowed nothing.
///
/// What this cannot see: whether the narrowed list is what a screen reader
/// actually hears, or whether the announced count matches the list. It only
/// says the handler reaches the one shared pipeline the sidebar selection and
/// a fresh contact load already trust, `recompute_which_contacts_are_shown`,
/// rather than a second filter of its own, and that it records what was
/// typed rather than only saying it out loud.
#[test]
fn test_typing_in_the_contacts_search_box_changes_what_the_list_shows() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let start = app
        .find("contacts_cp.search_input.on_text_changed")
        .expect("the search box handler is no longer wired, so this guard measures nothing");
    let end = app[start..]
        .find("contacts_sb.tree.on_selection_changed")
        .map(|at| start + at)
        .expect("the sidebar selection handler that follows the search box handler is gone");
    let handler = &app[start..end];

    assert!(
        handler.contains("recompute_which_contacts_are_shown"),
        "the search box handler no longer reaches the shared filter pipeline, so it \
         either does nothing or filters the list a second, different way"
    );
    assert!(
        handler.contains("contacts_search"),
        "the search box handler never records what was typed, so the shared pipeline \
         has nothing of the search box's own to filter by"
    );
    assert!(
        handler.contains("set_item_count"),
        "the list's row count is never updated, so the control a screen reader reads \
         stays at whatever it last held"
    );
    assert!(
        !handler.contains("Searching contacts"),
        "the search box handler still only announces that a search is starting, the \
         stub sentence this was meant to replace"
    );
}

/// An account somebody adds or edits reaches the database.
///
/// Nothing wrote one. Every call to `save_account` in the whole tree was a
/// test calling it; the Account Manager ended by assigning to the in-memory
/// list and never touched the cache; and startup read a table nothing had
/// ever written. So every account was gone on the next start.
///
/// The second failure is worse and follows from the first. The uninstall and
/// the erase-all-data command work out which credential-store entries to
/// remove by walking that table, so it found nothing, and every OAuth refresh
/// token ever written stayed on the machine, one per sign-in, carrying full
/// mail, contacts, calendar and tasks scope.
///
/// Why nothing caught it is the reason this test is in this file rather than
/// beside either half. The data layer proves the round trip against itself,
/// the Account Manager's own tests deliberately scope to the in-memory state
/// and say so in their file comment, and no test spanned the join. The
/// missing call sat exactly in the gap, invisible from both sides.
///
/// What this cannot see: whether the list handed over is the right one, or
/// whether the write succeeds. It asks that the handler calls the one method
/// that stores a whole list, which is the half nothing asked before.
#[test]
fn test_an_account_that_was_added_or_edited_reaches_the_database() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    let handler = app
        .split_once("fn handle_account_mgr")
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split_once("\n}\n").map(|(body, _)| body))
        .expect("the main window still handles the Account Manager");

    assert!(
        handler.contains("replace_accounts"),
        "the Account Manager changes the accounts in memory and nothing writes \
         them, so they are gone on the next start and their tokens are left \
         behind for the uninstall to miss"
    );
}

/// A draft that has been sent stops being a draft.
///
/// Nothing removed one. Sending did not, and the Open Draft dialog only
/// opens, so every draft ever saved stayed in the list for ever, the sent
/// ones alongside the unfinished ones. Automatic saving means one is written
/// every few minutes, so the list only ever grew.
///
/// What this cannot see: whether the removal succeeds, or whether the right
/// draft is named. It asks that sending removes one at all, and that it
/// happens after the message is queued rather than before, because a draft
/// removed first and a queue that then refused would lose the message.
#[test]
fn test_sending_a_draft_stops_it_being_a_draft() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    let sending = app
        .split_once("ComposeResult::Send(data) => {")
        .map(|(_, rest)| rest)
        .and_then(|rest| {
            rest.split_once("ComposeResult::SaveDraft")
                .map(|(arm, _)| arm)
        })
        .expect("the main window still handles Send");

    assert!(
        sending.contains("delete_draft"),
        "sending never removes the draft it was written in, so every draft \
         ever saved stays in the list, sent ones included"
    );

    let queued = sending
        .find("queue_for_sending")
        .expect("sending still queues the message");
    let removed = sending.find("delete_draft").expect("checked just above");
    assert!(
        queued < removed,
        "the draft is removed before the message is queued, so a queue that \
         refuses would lose the message altogether"
    );
}

/// Set Active in the Account Manager changes which account is active.
///
/// The dialog tracked the choice, announced it, and handed back only the
/// account list and the default. The active account was dropped on the way
/// out, and the main window only ever touches it when it names nothing or
/// names a deleted account, so a multi-account user was pinned to whichever
/// account came first at startup with no way to change it.
///
/// That is what made two other faults ordinary rather than exotic: commands
/// that took the open account instead of the message's own were reaching for
/// a value the person could not correct.
///
/// What this cannot see: whether the value carried is the one chosen. It asks
/// that it is carried at all and that the main window applies it.
#[test]
fn test_set_active_reaches_the_main_window() {
    let manager =
        fs::read_to_string("src/presentation/wx_account_manager.rs").expect("the account manager");
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    let handed_back = manager
        .split_once("AccountManagerAction::Updated {")
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split_once("\n    } else").map(|(arm, _)| arm))
        .expect("the account manager still hands back an update");
    assert!(
        handed_back.contains("active_id"),
        "the account manager drops the active account on the way out, so \
         Set Active says it worked and changes nothing"
    );

    let handler = app
        .split_once("fn handle_account_mgr")
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split_once("\n}\n").map(|(body, _)| body))
        .expect("the main window still handles the Account Manager");
    assert!(
        handler.contains("active_id: chosen_active"),
        "the main window ignores the active account the dialog chose"
    );
}

/// One dialog, one meaning for each Alt key.
///
/// Windows cycles between controls that share a mnemonic rather than pressing
/// one, so two controls claiming the same letter turn a shortcut into a
/// guess. The recurring-event dialog had Alt+C on both Continue and Cancel,
/// and Continue is the button whose answer can act on every day of a series.
///
/// Nothing covered this: the menu check in this file parses "Text\tShortcut"
/// out of menu labels and cannot see a `with_label("&X")` mnemonic, so no
/// dialog shortcut in the project was checked by anything at all.
///
/// What this cannot see: controls built in a loop, whose label is a variable
/// rather than a literal, and pages of a notebook, where two letters may
/// safely repeat because only one page shows at a time. It reads the literal
/// labels of one builder function at a time, which is where the collisions
/// found so far have been.
#[test]
fn test_no_two_controls_in_one_dialog_claim_the_same_alt_key() {
    let mut clashes = Vec::new();

    for path in fs::read_dir("src/presentation").expect("the presentation layer") {
        let path = path.expect("a directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        // Test code builds throwaway dialogs whose labels are not a person's
        // to press, and a notebook page is a scope of its own.
        let production = text.split("#[cfg(test)]").next().unwrap_or_default();
        for (name, body) in builder_bodies(production) {
            let mut claimed: std::collections::HashMap<char, Vec<String>> =
                std::collections::HashMap::new();
            for label in mnemonic_labels(body) {
                if let Some(key) = alt_key_of(&label) {
                    claimed.entry(key).or_default().push(label);
                }
            }
            for (key, labels) in claimed {
                if labels.len() > 1 {
                    clashes.push(format!(
                        "{}: {name} gives Alt+{} to {}",
                        path.display(),
                        key.to_ascii_uppercase(),
                        labels.join(" and ")
                    ));
                }
            }
        }
    }

    assert!(
        clashes.is_empty(),
        "{} dialog shortcut(s) mean two things at once:\n  {}",
        clashes.len(),
        clashes.join("\n  ")
    );
}

/// Each top-level function in a file, by name, with its body.
fn builder_bodies(text: &str) -> Vec<(String, &str)> {
    let mut found = Vec::new();
    let mut from = 0;
    // The nearer of the two, not the first that matches. Asking for `\nfn `
    // and falling back to `\npub fn ` only when that found nothing skipped
    // every public builder in any file that still had a private function
    // later in it, which is most of them: the check ran and read almost none
    // of what it claimed to. It went green while the Account Manager gave
    // Alt+D to both Delete and Set as Default.
    while let Some(at) = [text[from..].find("\nfn "), text[from..].find("\npub fn ")]
        .into_iter()
        .flatten()
        .min()
    {
        let start = from + at + 1;
        let name: String = text[start..]
            .lines()
            .next()
            .unwrap_or_default()
            .trim_start_matches("pub ")
            .trim_start_matches("fn ")
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let end = text[start..]
            .find("\n}\n")
            .map_or(text.len(), |offset| start + offset);
        found.push((name, &text[start..end]));
        from = start + 1;
    }
    found
}

/// Every `with_label("...")` literal in one body that carries a mnemonic.
///
/// A page of a notebook is its own scope, so a body that builds one is split
/// on the call that adds a page and each page read separately.
fn mnemonic_labels(body: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let scope = body.split("add_page").next().unwrap_or(body);
    let mut from = 0;
    while let Some(at) = scope[from..].find("with_label(\"") {
        let start = from + at + "with_label(\"".len();
        let Some(len) = scope[start..].find('"') else {
            break;
        };
        labels.push(scope[start..start + len].to_string());
        from = start + len;
    }
    labels
}

/// The letter a label claims, when it claims one.
///
/// `&&` is a literal ampersand rather than a mnemonic, the same rule Windows
/// itself follows.
fn alt_key_of(label: &str) -> Option<char> {
    let bytes: Vec<char> = label.chars().collect();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == '&' {
            match bytes.get(at + 1) {
                Some('&') => at += 2,
                Some(letter) if letter.is_alphanumeric() => {
                    return Some(letter.to_ascii_lowercase());
                }
                _ => at += 1,
            }
        } else {
            at += 1;
        }
    }
    None
}

/// The Account Manager's buttons tab in the order they are shown in.
///
/// wxWidgets gives a window its place in the tab order when it is created,
/// not when it is added to a sizer. Sign In Again was built before Set as
/// Default and shown after it, so Tab from Set Active landed on Sign In Again
/// while the button beside it on screen was Set as Default. Nothing says so
/// out loud: the window looks right, and only somebody moving through it by
/// keyboard meets the mismatch.
///
/// Written for this one window rather than as a rule over all of them. The
/// general version was tried and is not honest: a panel or a notebook has to
/// be built before the controls that go inside it, so building out of display
/// order is ordinary and correct for a container, and these buttons are
/// placed in a loop that a reading of the source cannot follow back to the
/// order they were made in. A check that reports the wrong answer in both
/// directions is worse than none.
#[test]
fn test_the_account_manager_buttons_tab_in_the_order_they_are_shown() {
    let manager =
        fs::read_to_string("src/presentation/wx_account_manager.rs").expect("the account manager");

    let built: Vec<usize> = ["ID_SET_ACTIVE", "ID_SET_DEFAULT", "ID_REAUTHORIZE"]
        .iter()
        .map(|id| {
            manager
                .find(&format!(".with_id({id})"))
                .unwrap_or_else(|| panic!("{id} is no longer a button here"))
        })
        .collect();

    let shown = manager
        .split_once("for b in [")
        .map(|(_, rest)| rest.split_once(']').map(|(row, _)| row).unwrap_or(""))
        .expect("the buttons are still placed as a row");
    let placed: Vec<usize> = ["&active", "&set_default", "&reauth"]
        .iter()
        .map(|name| {
            shown
                .find(name)
                .unwrap_or_else(|| panic!("{name} is no longer in the button row"))
        })
        .collect();

    let ordered = |v: &[usize]| v.windows(2).all(|pair| pair[0] < pair[1]);
    assert_eq!(
        ordered(&built),
        ordered(&placed),
        "Set Active, Set as Default and Sign In Again are built in one order \
         and shown in another, so Tab goes somewhere other than the button \
         beside the one you are on"
    );
}

/// A message that was sent is taken out of the queue, and a failure to take it
/// out is said rather than dropped.
///
/// The send has already happened by this point. If the row cannot be removed,
/// the next flush finds it still waiting and sends it a second time, and the
/// person it is addressed to gets two copies. That is the one failure in this
/// routine that reaches somebody outside the program, so it is the one that
/// must not be swallowed.
///
/// What this cannot see: whether the sentence is ever shown, or whether the
/// row really goes. It asks that the answer is looked at rather than discarded.
#[test]
fn test_a_sent_message_that_will_not_leave_the_queue_is_reported() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let flush = body_of(&app, "fn flush_outbox(");

    assert!(
        !flush.contains("let _ = cache.delete_outbox_message("),
        "the outbox drops the answer to whether a sent message left the queue. \
         It has already gone to the server at that point, so a row left behind \
         is sent again on the next flush and somebody receives it twice."
    );
    assert!(
        flush.contains("delete_outbox_message"),
        "nothing takes a sent message out of the queue, so every message would \
         be sent again on the next flush"
    );
}

/// A mail watch that stops, or never starts, is said rather than only logged.
///
/// The window watches the inbox so new mail arrives without asking for it.
/// When that watch ends, because the connection dropped or the server closed
/// it, nothing restarts it: the ordinary cycle starts a fresh watch only after
/// mail arrives and the folder is re-read, which is exactly what stops
/// happening. New mail then stops arriving on its own, the mailbox looks no
/// different, and the only record is a line in a file nobody reads.
///
/// `ImapIdleEvent::Stopped` says as much where it is declared: silence is what
/// a dropped connection looks like, which is why it is an event at all.
///
/// What this cannot see: whether either sentence reaches anybody, or whether
/// what it says is true. It asks only that both failures reach the routine
/// that speaks, and that both are still handled here.
#[test]
fn test_a_mail_watch_that_ends_is_reported_rather_than_only_logged() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let watcher = body_of(&app, "fn spawn_mail_watch(");

    for (marker, what) in [
        ("Could not watch the inbox", "a watch that never starts"),
        ("Stopped watching", "a watch that ends"),
    ] {
        assert!(
            watcher.contains(marker),
            "{what} is no longer handled here, so this guard is measuring nothing"
        );
    }
    assert!(
        watcher.matches("say_the_watch_is_off").count() >= 2,
        "one or both of the two ways a mail watch can fail goes to a log and \
         nowhere else. New mail stops arriving on its own and the mailbox looks \
         no different, which is the whole reason Stopped is an event."
    );
}

/// Removing one account erases exactly what uninstalling would erase for it.
///
/// These were two lists and they disagreed. Uninstalling named an account's
/// password and a token entry per provider; removing a single account erased
/// the password and nothing else. So a removed account left its refresh token
/// on the machine, and the sweep that would have caught it walks the accounts
/// that still exist, which no longer included that one. The token outlived the
/// program.
///
/// What this cannot see: whether either really deletes anything. The credential
/// store is behind a seam for the accounts' own passwords and not for the
/// tokens, so what is checked here is that one list is asked for in both
/// places rather than written out twice.
#[test]
fn test_one_list_says_which_secrets_belong_to_an_account() {
    let sweep = fs::read_to_string("src/application/forget.rs").expect("the erase-all sweep");
    let removal = fs::read_to_string("src/data/message_cache/accounts.rs").expect("the accounts");

    assert!(
        sweep.contains("entries_for_account"),
        "the uninstall sweep lists an account's token entries itself again, so it \
         can disagree with what removing that account erases"
    );
    assert!(
        !sweep.contains("OAuthService::providers()"),
        "the uninstall sweep walks the providers itself, which is the second \
         answer that let the two come apart"
    );
    assert!(
        removal.contains("forget_every_token_for"),
        "removing an account erases its password and leaves its sign-in tokens, \
         which nothing names again once the account has gone"
    );
}

/// A link this program will not open says so.
///
/// Two places open a link a sender wrote: the reader's own navigation, and
/// Save Link on the context menu. Both check the address first and refuse
/// anything that is not a scheme worth opening, which is right. Both then did
/// nothing at all, and wrote a line to a log.
///
/// Clicking a link and having nothing happen is indistinguishable from the
/// command being broken, and it is the case where somebody most needs telling:
/// a refused link is usually a link worth being wary of. This project already
/// took the same decision for the compose toolbar, where doing nothing was
/// replaced by saying it is not built.
///
/// What this cannot see: whether the sentence is shown, or whether it is true.
#[test]
fn test_a_link_that_will_not_be_opened_is_not_refused_in_silence() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    let refusals = app.matches("Refused to open").count();
    assert!(
        refusals >= 2,
        "one of the two places that refuse a link no longer does, so this guard \
         is measuring less than it was written for"
    );
    assert_eq!(
        // The call sites, not the definition, which also carries the name.
        app.matches("say_the_link_was_refused(&").count(),
        refusals,
        "a link refused as unsafe is still logged and not said. Nothing happens \
         when somebody activates it, which reads as the command being broken \
         rather than as a warning about the link."
    );
}

/// Every label in one stretch of menu-building code that claims a letter.
///
/// Reads the string literals rather than the built menu, because the menus are
/// built by wxWidgets at run time and there is nothing to ask at test time.
/// Both plain labels and the ones built by `format!` are string literals on one
/// line, so one rule finds both.
fn menu_labels_claiming_a_letter(segment: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in segment.lines() {
        // Odd-numbered pieces of a split on the quote are the contents of the
        // string literals. The even ones are the code between them, which is
        // where `&format!` lives and is exactly what must not be read as a
        // mnemonic.
        for piece in line.split('"').skip(1).step_by(2) {
            if alt_key_of(piece).is_none() {
                continue;
            }
            // The accelerator after the tab is a different thing from the
            // mnemonic and is checked by its own guard.
            found.push(piece.split("\\t").next().unwrap_or(piece).to_string());
        }
    }
    found
}

/// Everything that builds one named menu.
///
/// Two halves, because wxWidgets only lets a submenu be added once the builder
/// chain has finished: the chain itself, and every item or submenu appended to
/// the menu afterwards. A menu built entirely the second way has an empty first
/// half and is read just as well.
fn menu_block(ship: &str, name: &str) -> String {
    let mut block = String::new();
    if let Some(at) = ship.find(&format!("let {name} = Menu::builder()")) {
        let chain = &ship[at..];
        let ends = chain.find(".build();").unwrap_or(chain.len());
        block.push_str(&chain[..ends]);
    }
    let lines: Vec<&str> = ship.lines().collect();
    let mut at = 0;
    while at < lines.len() {
        let trimmed = lines[at].trim_start();
        if !(trimmed.starts_with(&format!("{name}.append"))
            || trimmed.starts_with(&format!("{name}.prepend")))
        {
            at += 1;
            continue;
        }
        // rustfmt wraps a long call over several lines and puts the label on
        // one of its own, so taking only the first line would read the call and
        // miss the very thing being checked.
        let mut end = at;
        while end + 1 < lines.len() && !lines[end].trim_end().ends_with(");") {
            end += 1;
        }
        for line in &lines[at..=end] {
            block.push('\n');
            block.push_str(line);
        }
        at = end + 1;
    }
    block
}

/// The source with every space taken out, so a call can be found whatever way
/// rustfmt has chosen to wrap it.
fn without_whitespace(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Whether a command is on a menu somewhere, by any of the ways one is added.
fn is_on_a_menu(squashed: &str, id: &str) -> bool {
    [
        format!("append({id},"),
        format!("append_item({id},"),
        format!("append_check_item({id},"),
        format!("append_radio_item({id},"),
    ]
    .iter()
    .any(|call| squashed.contains(call))
}

/// No two items on one menu claim the same letter.
///
/// Inside an open menu a mnemonic is meant to be one keystroke: press the
/// letter and the item runs. Two items sharing a letter silently downgrades
/// both to a three-key cycle, press, press again, then Enter, and nothing tells
/// the person that the letter they were taught no longer finishes the job. It
/// is the kind of fault that only shows up under the keyboard, which is how
/// this application is meant to be used.
///
/// Written before moving a pile of module commands onto the Action menu. A
/// longer menu is where this stops being theoretical, and the check has to
/// exist before the items land rather than after.
#[test]
fn test_no_two_items_on_one_menu_claim_the_same_letter() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let ship = &app[..app.find("\n#[cfg(test)]").unwrap_or(app.len())];

    let mut menus_checked = 0;
    for (at, _) in ship.match_indices("= Menu::builder()") {
        let binding = ship[..at].rfind("let ").expect("a menu is bound to a name");
        let name = ship[binding + "let ".len()..at].trim();

        // A submenu, or an item appended once the builder has finished, sits on
        // the menu like any other and claims a letter like any other.
        let labels = menu_labels_claiming_a_letter(&menu_block(ship, name));
        if labels.len() < 2 {
            continue;
        }
        menus_checked += 1;

        for (index, label) in labels.iter().enumerate() {
            let letter = alt_key_of(label).expect("only labels claiming a letter are collected");
            let clash = labels[..index]
                .iter()
                .find(|earlier| alt_key_of(earlier) == Some(letter));
            if let Some(earlier) = clash {
                panic!(
                    "on the {name} menu, \"{label}\" and \"{earlier}\" both claim {letter}, \
                     so pressing {letter} runs neither and cycles between them instead"
                );
            }
        }
    }

    assert!(
        menus_checked >= 6,
        "only {menus_checked} menus were read, so this guard is measuring almost nothing"
    );
}

/// Everything you can do to the thing in front of you is on the menu bar.
///
/// Each of these had a handler and a place on the menu raised by the
/// Applications key, and no place on the bar at all. The context menu is
/// keyboard-reachable, so none of this was unusable; it was undiscoverable,
/// which is a different fault with the same ending. Somebody who does not know
/// a command exists cannot press the key that raises the menu that would have
/// told them.
///
/// New items are the deliberate exception and stay under File, New. Making a
/// thing is not something done to the thing in front of you.
#[test]
fn test_every_command_that_acts_on_a_selection_is_on_the_menu_bar() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let ship = &app[..app.find("\n#[cfg(test)]").unwrap_or(app.len())];
    let squashed = without_whitespace(ship);

    for (id, what) in [
        ("ID_MOVE_TO_FOLDER", "move this somewhere else"),
        (
            "ID_COPY_TO_FOLDER",
            "put a copy of this message in another folder",
        ),
        ("ID_CONTEXT_COPY_TO_TASK", "make a task out of this message"),
        (
            "ID_CONTEXT_COPY_TO_EVENT",
            "make a calendar event out of it",
        ),
        ("ID_CONTEXT_COPY_TO_NOTE", "make a note out of it"),
        ("ID_PIM_TOGGLE_DONE", "mark this task or reminder done"),
        ("ID_PIM_TOGGLE_PIN", "pin this note"),
        (
            "ID_CONTEXT_WRITE_TO_GROUP",
            "write to everybody in this group",
        ),
        ("ID_CONTEXT_ADD_TO_GROUP", "put this contact in a group"),
        (
            "ID_CONTEXT_REMOVE_FROM_GROUP",
            "take this contact out of a group",
        ),
        ("ID_CONTEXT_RENAME_CONTAINER", "rename this group"),
        (
            "ID_CONTEXT_DELETE_CONTAINER",
            "delete this calendar, list or group",
        ),
        (
            "ID_CONTEXT_SYNC_NOW",
            "fetch this module from the provider now",
        ),
        ("ID_REFRESH_FOLDER", "read this folder again"),
        ("ID_GET_OLDER", "fetch older messages in this folder"),
        (
            "ID_CHOOSE_FOLDERS",
            "choose which folders are kept up to date",
        ),
    ] {
        assert!(
            is_on_a_menu(&squashed, id),
            "\"{what}\" ({id}) is on no menu on the bar, so the only way to find \
             out it exists is to already know"
        );
    }
}

/// Move follows the module rather than always meaning a mail folder.
///
/// One item on the Action menu and one key, the rule Delete beside it already
/// follows. In Mail it asks which folder; anywhere else it asks which calendar,
/// list or note folder, because offering a mail folder as a home for a task is
/// not a mistake worth making reachable.
///
/// The arm that answers for the other five modules has to come first, since a
/// match is read top down and the mail arm below it takes the command
/// unconditionally.
#[test]
fn test_move_follows_the_module_rather_than_always_meaning_a_mail_folder() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let ship = &app[..app.find("\n#[cfg(test)]").unwrap_or(app.len())];
    let squashed = without_whitespace(ship);

    assert!(
        squashed.contains("id==ID_MOVE_TO_FOLDER&&showing!=PimModule::Mail"),
        "Move is no longer answered by the module it was raised in, so choosing \
         it on a task offers a list of mail folders"
    );
    assert!(
        squashed.contains("id==ID_CONTEXT_MOVE_ITEM||id==ID_MOVE_TO_FOLDER{PimCommand::Move"),
        "the menu bar's Move no longer asks for a move, so it reaches the item \
         handler and does something else there"
    );

    let pim_arm = squashed
        .find("id==ID_MOVE_TO_FOLDER&&showing!=PimModule::Mail")
        .expect("the arm for the other five modules");
    let mail_arm = squashed
        .find("id==ID_MOVE_TO_FOLDER||id==ID_COPY_TO_FOLDER")
        .expect("the arm for mail");
    assert!(
        pim_arm < mail_arm,
        "the mail arm is read first, so Move in Tasks or Notes offers mail \
         folders and the module arm below it is never reached"
    );
}

/// No control that has a label of its own is built without one.
///
/// Windows takes a native control's UI Automation name, which is what Narrator
/// reads, from the control's own window text. `set_accessible_name` writes
/// MSAA, which is what NVDA reads, and nothing else. So a check box built with
/// an empty label and named only in code has a name on one channel and none on
/// the other, and it reads correctly right up until somebody uses the other
/// screen reader.
///
/// Two check boxes in the item form were built that way, an event's "All day"
/// and a note's "Pin to the top": the label went onto a static text beside the
/// control, which is right for a text box and wrong for the one field kind that
/// has somewhere of its own to put it.
///
/// `tests/checkbox_labels.rs` asks the built controls the same question, which
/// is the stronger check. This one is cheap, and it covers every dialog rather
/// than the ones a live test builds.
#[test]
fn test_no_control_with_a_label_of_its_own_is_built_without_one() {
    let mut built_blank = Vec::new();
    for file in sources() {
        let text = fs::read_to_string(&file).expect("a source file");
        let squashed = without_whitespace(&text);
        for kind in [
            "CheckBox",
            "RadioButton",
            "Button",
            "ToggleButton",
            "RadioBox",
        ] {
            if squashed.contains(&format!("{kind}::builder(&dlg).with_label(\"\")"))
                || squashed.contains(&format!("{kind}::builder(parent).with_label(\"\")"))
                || squashed.contains(&format!("{kind}::builder(&dialog).with_label(\"\")"))
            {
                built_blank.push(format!("{} builds a {kind} with no label", file.display()));
            }
        }
    }

    assert!(
        built_blank.is_empty(),
        "these carry no name on the UI Automation channel, so one screen reader \
         hears them and the other hears an unnamed control:\n  {}",
        built_blank.join("\n  ")
    );
}

/// The signature editor offers no box that reaches no message.
///
/// It offered two. One was the signature; the other was headed "HTML version
/// (optional)", was stored, was carried through three layers of conversion, was
/// written to the database and was never read when a message was composed. The
/// send path asks for the default signature, takes `content_plain` off it and
/// drops the rest, so anybody who filled that box in was typing into nothing
/// and was told it saved.
///
/// Both directions, because one alone rots. Checking only that the box is gone
/// would pass on a build where the box that remains has also stopped being
/// read; checking only that something is read would pass on a build that offers
/// three boxes and reads one.
#[test]
fn test_the_signature_editor_offers_no_box_that_reaches_no_message() {
    let managers = fs::read_to_string("src/presentation/wx_managers.rs").expect("the managers");
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    assert!(
        !managers.contains("Signature, HTML version"),
        "the signature editor still offers a box for markup, and nothing on the \
         send path reads it. The signature is written in Markdown now, which is \
         what that box was for"
    );

    // And the one that is left is really read, so this does not pass by both
    // halves being absent.
    assert!(
        app.contains("s.content_plain"),
        "nothing on the send path takes the signature's text any more, so every \
         box in that editor is now one that reaches no message"
    );
}

/// The chosen font reaches every list, not only the mail one.
///
/// The size setting existed for as long as the settings screen did and reached
/// the message list alone, so somebody who made the text bigger because they
/// could not read it found their contacts, calendar, tasks, notes and reminders
/// exactly as they were. That is the setting most likely to be reached for by
/// somebody with low vision, and it worked in a sixth of the places it claimed.
///
/// Counted rather than eyeballed, because a list added later is a list that
/// silently keeps the system font.
#[test]
fn test_every_item_list_is_drawn_in_the_font_that_was_chosen() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let squashed = without_whitespace(&app);

    assert!(
        squashed.contains("fnthe_chosen_list_font()"),
        "there is no one answer for what font a list is drawn in, so each list \
         reads the settings for itself and they can disagree"
    );

    assert!(
        squashed.contains("msg_list.set_font("),
        "the message list is not given the chosen font"
    );

    // Only the block that hands the font to the other five, not the whole
    // file: every one of these names appears elsewhere, so searching the file
    // would find them however the font is applied. The first version of this
    // guard did exactly that and passed with a list taken out.
    let at = squashed
        .find("forlistin[")
        .expect("the loop that gives the other lists their font");
    let loop_body = &squashed[at..squashed.len().min(at + 300)];

    for list in [
        "cal_event_list",
        "contact_list",
        "reminder_list",
        "task_list",
        "note_list",
    ] {
        assert!(
            loop_body.contains(list),
            "{list} is not in the loop that gives the lists their font, so \
             changing the font or the size leaves it alone: {loop_body}"
        );
    }

    // And nothing reads the size on its own any more. A second reader is a
    // second place the typeface can be forgotten.
    assert!(
        !squashed.contains("app_config().font_size"),
        "something reads the font size without the typeface beside it, which is \
         how the size came to apply where the typeface does not"
    );

    // Applied again when settings closes, not only where the lists are built.
    // Read once at startup, changing the font or the size did nothing at all
    // until the next start, which reads as a settings screen that ignores you.
    let after_settings = squashed
        .find("handle_settings(&frame")
        .map(|at| &squashed[at..squashed.len().min(at + 900)])
        .expect("the settings command");
    assert!(
        after_settings.contains("the_chosen_list_font()"),
        "the font is not applied again when settings closes, so changing it or          the size does nothing until the next start"
    );
}

/// One copy runs, and the rest hand what they were given to it.
///
/// Three pieces, in three files, and any one of them missing turns this back
/// into what it replaced: a second copy of the whole mail client for every link
/// clicked, sharing one database and one outbox. The outbox has no notion of
/// who is sending, so two copies both send the queued message and the person on
/// the other end receives it twice, which cannot be taken back.
///
/// Checked here rather than trusted because the three are far apart and none of
/// them fails loudly on its own. A copy that does not hand over just works, and
/// a copy that does not listen just works, and the fault only shows up on
/// somebody else's machine as mail sent twice.
#[test]
fn test_a_second_copy_hands_over_rather_than_running_alongside_the_first() {
    let main = fs::read_to_string("src/main.rs").expect("the program's entry point");
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let squashed_main = without_whitespace(&main);
    let squashed_app = without_whitespace(&app);

    assert!(
        squashed_main.contains("how_to_start(another_was_already_running"),
        "src/main.rs does not ask whether another copy is running, so every link \
         clicked starts a second whole mail client"
    );
    assert!(
        squashed_main.contains("handover::hand_over(&argument)"),
        "src/main.rs decides to hand over and then does not, so a second copy \
         runs alongside the first"
    );
    assert!(
        squashed_app.contains("handover::listen("),
        "nothing listens for another copy, so every handover is refused and \
         every link opens a second window"
    );
    assert!(
        squashed_app.contains("UIUpdate::HandedOver(argument)=>"),
        "a handover arrives and nothing answers it, so a link clicked while \
         Wixen Mail is running does nothing at all"
    );

    // And the departing copy stops. Without the return it hands over and then
    // opens a window as well, which is the fault plus a wasted message.
    let at = squashed_main
        .find("handover::hand_over(&argument)")
        .expect("the handover");
    let after = &squashed_main[at..squashed_main.len().min(at + 200)];
    assert!(
        after.contains("return"),
        "the copy that hands over does not stop, so it hands over and then runs \
         anyway: {after}"
    );
}

/// Every long prose box says Markdown is understood in it.
///
/// The four fields the item form builds are held to this by a test beside their
/// table, in `application::item_fields`. These two are built by hand and that
/// test cannot see them, which is exactly how the contact notes box ended up
/// being the one long field in the application where Markdown did nothing and
/// nothing said so either way.
///
/// The promise has to be on the control rather than only in the documentation.
/// A screen reader reads the description when the field takes focus, and that
/// is the moment somebody decides whether to type a hash.
#[test]
fn test_every_long_prose_box_says_markdown_is_understood() {
    let managers = fs::read_to_string("src/presentation/wx_managers.rs").expect("the managers");
    let squashed = without_whitespace(&managers);

    for (control, what) in [
        ("&notes_f", "the notes on a contact"),
        ("&content_f", "a signature"),
    ] {
        let at = squashed
            .find(&format!("set_accessible_name_and_description({control},"))
            .unwrap_or_else(|| {
                panic!(
                    "{what} ({control}) carries a name with no description, so nothing \
                     tells anybody Markdown is understood in it"
                )
            });
        let described = &squashed[at..squashed.len().min(at + 400)];
        assert!(
            described.contains("Markdown"),
            "the description on {what} does not mention Markdown, so the field \
             promises nothing and somebody types a hash to find out"
        );
    }
}

/// Making a calendar, list, folder or group is under File, New.
///
/// Making a thing is not something done to the thing in front of you, so it
/// belongs with the other New entries rather than on the Action menu. It was in
/// neither place: the six item kinds were all under New and the four containers
/// that hold them were reachable only by the Applications key, so somebody
/// looking for how to start a second calendar had nowhere to find out.
#[test]
fn test_making_a_calendar_or_list_is_under_file_new() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let ship = &app[..app.find("\n#[cfg(test)]").unwrap_or(app.len())];

    let new_menu = without_whitespace(&menu_block(ship, "new_sub"));
    assert!(
        new_menu.contains("ID_NEW_EVENT"),
        "the New submenu was not found, so this guard is measuring nothing"
    );
    assert!(
        new_menu.contains("ID_CONTEXT_NEW_CONTAINER"),
        "making a calendar, task list, note folder or contact group is on no \
         menu at all, so the only way to start a second one is to already know \
         the Applications key offers it"
    );
}

/// File and Edit hold nothing that acts on the selected thing.
///
/// Both had collected commands that belong to neither. File carried move and
/// copy, which act on the chosen message rather than on a file, and Edit
/// carried marking a task done and pinning a note, which are not edits in the
/// sense every other Windows application uses that menu for. Edit keeps Search,
/// where Find has lived on this platform for thirty years.
#[test]
fn test_file_and_edit_hold_nothing_that_acts_on_a_selection() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let ship = &app[..app.find("\n#[cfg(test)]").unwrap_or(app.len())];

    for (menu, moved) in [
        (
            "file",
            [
                "ID_MOVE_TO_FOLDER",
                "ID_COPY_TO_FOLDER",
                "ID_REFRESH_FOLDER",
                "ID_GET_OLDER",
                "ID_CHOOSE_FOLDERS",
            ]
            .as_slice(),
        ),
        (
            "edit",
            ["ID_PIM_TOGGLE_DONE", "ID_PIM_TOGGLE_PIN"].as_slice(),
        ),
    ] {
        let block = menu_block(ship, menu);
        assert!(
            block.contains("append"),
            "the {menu} menu was not found, so this guard is measuring nothing"
        );
        for id in moved {
            assert!(
                !block.contains(id),
                "{id} is still on the {menu} menu, so it is offered in two places \
                 and the one it was moved to is not the only answer"
            );
        }
    }
}

/// No two menus on the bar answer to the same Alt key.
///
/// The bar is reached by Alt and then a letter, so two menus claiming one
/// letter means one of them cannot be opened that way at all. Somebody working
/// by keyboard loses a whole menu and nothing says why.
///
/// Written when Message became Action. That freed M and claimed A, and A was
/// only free by luck rather than by anything checking.
#[test]
fn test_no_two_menus_on_the_bar_claim_the_same_alt_key() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let bar = body_of(&app, "        MenuBar::builder()");

    let mut claimed: Vec<(char, String)> = Vec::new();
    for line in bar.lines() {
        let Some(at) = line.find(".append(") else {
            continue;
        };
        let Some(amp) = line[at..].find("\"&") else {
            continue;
        };
        let rest = &line[at + amp + 2..];
        let Some(letter) = rest.chars().next() else {
            continue;
        };
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
        claimed.push((letter.to_ascii_uppercase(), name));
    }

    assert!(
        claimed.len() >= 5,
        "only {} menus were read off the bar, so this guard is measuring almost \
         nothing: {claimed:?}",
        claimed.len()
    );
    for (index, (letter, name)) in claimed.iter().enumerate() {
        if let Some((_, other)) = claimed[..index].iter().find(|(seen, _)| seen == letter) {
            panic!(
                "the {name} menu and the {other} menu both answer to Alt+{letter}, so one \
                 of them cannot be opened from the keyboard at all"
            );
        }
    }
}

/// Bringing the window forward has to undo minimising as well.
///
/// `show(true)` and `raise()` between them do not restore a window somebody
/// minimised: on Windows it stays in the taskbar, having been shown and
/// raised, and whatever asked for it looks exactly like something that did
/// nothing. Both callers are the ones where that matters most, because both
/// are reached when the window is not on screen: clicking the tray icon, and
/// following a `mailto:` link that hands over to the copy already running.
///
/// Read out of the source rather than driven, because the failure only appears
/// on a window that is really minimised on a real desktop.
#[test]
fn test_nothing_raises_the_window_without_also_taking_it_out_of_the_taskbar() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");

    let raising: Vec<usize> = app
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("frame.raise()"))
        .map(|(at, _)| at)
        .collect();

    assert!(
        !raising.is_empty(),
        "nothing raises the window at all, so this guard is measuring nothing"
    );

    let lines: Vec<&str> = app.lines().collect();
    for at in raising {
        // The three lines in front of it: the un-minimising and the showing.
        let looked_at = lines[at.saturating_sub(3)..=at].join("\n");
        assert!(
            looked_at.contains("iconize(false)"),
            "line {} raises the window without taking it out of the taskbar, so \
             a minimised window stays minimised:\n{looked_at}",
            at + 1
        );
    }
}
