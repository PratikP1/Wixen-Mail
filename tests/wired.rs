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
/// Found by this test the day it was written, all of the same shape as `F6`:
/// a handler, an id, no way to raise it. Each needs somebody to decide whether
/// Wixen Mail wants the command, and that is a product question rather than a
/// mechanical fix, so they are listed rather than quietly wired or quietly
/// deleted. Task #71.
const KNOWN_DEAD: [&str; 11] = [
    "ID_CALENDAR",
    "ID_CONTACT_MGR",
    "ID_FILTER_MGR",
    "ID_GET_OLDER",
    "ID_NEXT_UNREAD",
    "ID_OPEN_DRAFT",
    "ID_PREV_UNREAD",
    "ID_REFRESH_FOLDER",
    "ID_SIG_MGR",
    "ID_TAG_MGR",
    "ID_TOGGLE_STAR",
];

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
