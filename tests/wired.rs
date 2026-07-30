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
            // The bare form, used when a menu is built conditionally rather
            // than in one builder chain. The message preview's context menu
            // adds its link items this way, only when the click was on a link.
            "menu.append(",
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
