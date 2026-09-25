//! The keys a page gives back to its window so somebody can jump inside it.
//!
//! A WebView keeps every key once the browser has focus. A `KEY_DOWN` bound
//! on the control itself never fires there, which is how the formatted
//! message window's F8 sat dead from the day it was written: the binding was
//! on the browser control, the sentence read on open promised the key, and
//! the key never left the page (#84, reported 2026-09-18 under NVDA on
//! `1.0.0-alpha.1+149.g744d05ef`). The way out, Escape and F6, had met the
//! same fault earlier and was answered with a script injected into the
//! document that posts the key to the window. The jumps take the same route.
//!
//! Pratik's decision of 2026-09-18: Alt+A reaches the attachments of an open
//! message in both message windows, and F8 is retired there. F8 keeps its
//! other meanings, the Columns dialog in the main window and the toolbar in
//! the composer. F7 reaches the security warning, as it does in the plain
//! text reader.
//!
//! One module holds both halves, the script that posts a kind and the reader
//! that turns the kind back into a jump, so the word the page sends and the
//! word the window matches cannot come to differ; a test here walks every
//! kind the script posts through the reader.

/// Where a page asked its window to move focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jump {
    /// Alt+A: the list of attachments, when the message has any.
    Attachments,
    /// F7: the security warning above the message, when there is one.
    Warning,
    /// Ctrl+P: print what the window shows, through Windows' print dialog.
    /// Not a move of focus, and still a key the page hands to its window.
    Print,
}

/// The listener a page runs to post the jumps, injected after the way out.
///
/// Alt+A is read as the Alt key with the letter in either case, and never
/// with Control held: Control+Alt is AltGr on many keyboard layouts, where
/// the same chord types a letter of that language, and a shortcut that eats
/// a letter somebody is typing is worse than none. F7 is the bare key.
pub const SCRIPT: &str = r#"document.addEventListener('keydown', function(e) {
    // The jumps inside the window: Alt+A to the attachments, F7 to the
    // warning. Taken from the browser before it acts on them, the way Escape
    // and F6 are, because the browser's own F7 is caret browsing.
    if (e.key === 'F7' && !e.altKey && !e.ctrlKey) {
        e.preventDefault();
        e.stopPropagation();
        window.contextMenu.postMessage(JSON.stringify({ kind: 'warning' }));
    }
    if (e.altKey && !e.ctrlKey && (e.key === 'a' || e.key === 'A')) {
        e.preventDefault();
        e.stopPropagation();
        window.contextMenu.postMessage(JSON.stringify({ kind: 'attachments' }));
    }
    // Print, the reader window's key for the same command (#45). Taken from
    // the browser, whose own Ctrl+P would print the page as it draws it.
    if (e.ctrlKey && !e.altKey && (e.key === 'p' || e.key === 'P')) {
        e.preventDefault();
        e.stopPropagation();
        window.contextMenu.postMessage(JSON.stringify({ kind: 'print' }));
    }
}, true);"#;

/// The jump a page asked for, read from the message it posted.
///
/// `None` for the way out, the context menu and anything unreadable, which
/// the window answers elsewhere or ignores.
pub fn the_jump_the_page_asked_for(json: &str) -> Option<Jump> {
    let posted = serde_json::from_str::<serde_json::Value>(json).ok()?;
    match posted.get("kind").and_then(serde_json::Value::as_str)? {
        "attachments" => Some(Jump::Attachments),
        "warning" => Some(Jump::Warning),
        "print" => Some(Jump::Print),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `kind: '...'` the script posts, spelled as the script spells it.
    fn every_kind_the_script_posts() -> Vec<String> {
        SCRIPT
            .match_indices("kind: '")
            .filter_map(|(at, marker)| {
                let rest = &SCRIPT[at + marker.len()..];
                rest.find('\'').map(|end| rest[..end].to_string())
            })
            .collect()
    }

    #[test]
    fn test_alt_a_in_the_page_posts_the_attachments_and_the_window_reads_it_back() {
        // The letter in either case, with Alt and without Control, so AltGr
        // on a layout where that chord types a letter is left alone.
        assert!(SCRIPT.contains("e.altKey && !e.ctrlKey"), "{SCRIPT}");
        assert!(
            SCRIPT.contains("e.key === 'a' || e.key === 'A'"),
            "{SCRIPT}"
        );
        assert!(SCRIPT.contains("kind: 'attachments'"), "{SCRIPT}");

        assert_eq!(
            the_jump_the_page_asked_for(r#"{"kind":"attachments"}"#),
            Some(Jump::Attachments)
        );
    }

    #[test]
    fn test_f7_in_the_page_posts_the_warning_and_the_window_reads_it_back() {
        assert!(SCRIPT.contains("e.key === 'F7'"), "{SCRIPT}");
        assert!(SCRIPT.contains("kind: 'warning'"), "{SCRIPT}");

        assert_eq!(
            the_jump_the_page_asked_for(r#"{"kind":"warning"}"#),
            Some(Jump::Warning)
        );
    }

    #[test]
    fn test_ctrl_p_in_the_page_posts_print_and_the_window_reads_it_back() {
        // Control without Alt, so AltGr on a layout where Control+Alt+P types
        // a letter is left alone, as Alt+A leaves Control alone.
        assert!(SCRIPT.contains("e.ctrlKey && !e.altKey"), "{SCRIPT}");
        assert!(
            SCRIPT.contains("e.key === 'p' || e.key === 'P'"),
            "{SCRIPT}"
        );
        assert!(SCRIPT.contains("kind: 'print'"), "{SCRIPT}");

        // Named rather than only something, now the jump exists to name: the
        // red half could ask only whether the window read the kind at all.
        assert_eq!(
            the_jump_the_page_asked_for(r#"{"kind":"print"}"#),
            Some(Jump::Print)
        );
    }

    #[test]
    fn test_the_script_takes_the_key_from_the_browser_before_posting_it() {
        // Without this the browser still acts on the key after the window
        // has: F7 is caret browsing in Edge, and Alt+A would reach whatever
        // the page put on it.
        for taken in ["e.preventDefault();", "e.stopPropagation();"] {
            assert!(SCRIPT.contains(taken), "{taken} is not in the script");
        }
        assert!(
            !SCRIPT.contains("F8"),
            "F8 is retired from the page: {SCRIPT}"
        );
    }

    #[test]
    fn test_every_kind_the_script_posts_is_one_the_window_reads() {
        let kinds = every_kind_the_script_posts();
        assert_eq!(kinds.len(), 3, "{kinds:?}");
        for kind in kinds {
            let posted = format!(r#"{{"kind":"{kind}"}}"#);
            assert!(
                the_jump_the_page_asked_for(&posted).is_some(),
                "the script posts {kind:?} and the window reads it as nothing"
            );
        }
    }

    #[test]
    fn test_the_way_out_the_context_menu_and_nonsense_are_not_jumps() {
        for not_a_jump in [
            r#"{"kind":"leave"}"#,
            r#"{"kind":"leave","back":true}"#,
            r#"{"kind":"context","x":1,"y":2}"#,
            r#"{"kind":"toolbar"}"#,
            r#"{"href":"https://example.com"}"#,
            "not json",
            "",
        ] {
            assert_eq!(
                the_jump_the_page_asked_for(not_a_jump),
                None,
                "{not_a_jump}"
            );
        }
    }
}
