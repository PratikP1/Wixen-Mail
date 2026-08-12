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

/// Nothing asks a server to delete a message before asking where it should go.
///
/// The order is the whole of the fix. On an account whose trash this program
/// does not recognise, the ordinary Delete used to flag the message and remove
/// it from the server with no copy kept anywhere, and announce that as a
/// deletion. Two of the four answers to "where does this go" are refusals, and
/// a refusal that has already opened a session is a refusal decided too late.
///
/// Read from the source because reaching this handler needs a window, an
/// account with stored credentials and a mail server.
#[test]
fn test_nothing_asks_a_server_to_delete_before_asking_where_it_goes() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let handler = body_of(&app, "fn spawn_server_change(");

    let decided = handler
        .find("where_a_deleted_message_goes")
        .expect("the delete handler never asks where a deleted message goes");
    let connected = handler
        .find("connect_imap")
        .expect("the delete handler never connects, so this guard measures nothing");

    assert!(
        decided < connected,
        "the server is connected to before anything has asked where the message \
         should go, so an account with no recognised trash gets as far as the wire"
    );
}

/// A delete with no recognised trash is refused out loud, not sent anyway.
///
/// The two refusals are sentences kept beside the decision so they can be
/// tested. What no test there can say is whether the handler says them: an arm
/// that falls through to "there is nowhere to move it to" reads as a tidy
/// simplification and puts the destructive delete straight back.
#[test]
fn test_a_delete_with_no_recognised_trash_is_refused_rather_than_sent() {
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let handler = body_of(&app, "fn spawn_server_change(");

    for refusal in ["NO_TRASH_FOLDER_FOUND", "NO_FOLDERS_KNOWN_YET"] {
        assert!(
            handler.contains(refusal),
            "the delete handler no longer says why it did nothing, so a message \
             whose account has no recognised trash is removed from the server \
             again and announced as deleted"
        );
    }
    assert!(
        handler.contains("UIUpdate::CommandRefused"),
        "the refusal goes to the status bar and nowhere else, which from the \
         keyboard is indistinguishable from a key that was never wired up"
    );
}

/// The row and the sentence are decided in one place for both handlers.
///
/// They have to agree. A delete or a move whose copy landed and whose original
/// the server then left alone must keep its row, and must say so; the version
/// before this took the row out for every answer that was not a failure and
/// wrote its own sentence a few lines below one that said the same event
/// differently. Two pieces of code answering one question is how the list and
/// the server came to disagree in the first place.
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

/// The calendar sync asks whether anything can send what is still waiting.
///
/// The sweep is a plain function over the account's rows, so it is tested
/// properly in `application::calendar`. What no test there can say is whether
/// the sync calls it: reaching the sync means a window, an account and a
/// runtime, and it is the calling that was missing before. A change nothing
/// can send waits for ever and is looked at again on every sync, and the only
/// place anybody learns that is the summary this feeds.
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
