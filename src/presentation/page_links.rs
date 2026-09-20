//! A link activated in a page, caught in the page and handed to the window.
//!
//! Every page this application shows in a browser control, the message
//! preview and the formatted message window, ran a `WEBVIEW_NAVIGATING`
//! handler that vetoed every navigation and handed the address to the
//! default browser. The tester reported on 2026-09-18 (#80) that Enter on a
//! link opened it in the same window instead. The probe for 11-11.1 found
//! why by reading rather than by watching: wxWidgets carries a navigating
//! event's address in `wxWebViewEvent::GetURL`, and wxdragon 0.9.17 hands a
//! `WebViewEventData` only the command event's string, which a navigating
//! event never sets. The handler read an empty string, took it for the
//! control loading its own document, and vetoed nothing, on every surface,
//! for every way of activating a link, since the day it was written. A veto
//! that never fires reads exactly like one that does, because nothing shows
//! the browser did what the handler was written to stop.
//!
//! So the link is caught before the browser navigates. A `click` event
//! fires on an anchor for a mouse click, for Enter on a focused link, and
//! for the default action a screen reader's Enter performs in browse mode,
//! which the browser delivers as a synthesised click. A listener on the
//! document calls `preventDefault` for an anchor and posts the address to
//! the window, which decides where it goes through
//! [`crate::application::opening_links::route`]. The route holds whether or
//! not the veto ever fires.
//!
//! One module holds both halves, the script that posts and the reader that
//! turns what it posted back into an ask, so the word the page sends and
//! the word the window matches cannot come to differ; a test walks every
//! kind the script posts through the reader, as `page_jumps` does. The way
//! back from a page, Backspace or Alt+Left, is posted from here too and
//! read here, and is not in `wx_app.rs` on purpose: a reading there holds
//! every key a page asks for to the section on getting out of the
//! conversation window, and Backspace does not get out of it.

use crate::application::opening_links::Asked;

/// What a page posted to its window about a link or the way back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Posted {
    /// A link activated, with the address as the page resolved it and how it
    /// was asked for: activated plainly, or with the browser's own modifiers
    /// for a new tab (Ctrl) or a new window (Shift).
    Link { href: String, asked: Asked },
    /// Backspace or Alt+Left: the message back in the page's place.
    Back,
}

/// The listener every page runs, injected after the way out.
///
/// An anchor's `href` is posted as the page resolved it. A fragment, `#top`,
/// is left to the browser: it moves within the document and leaves it. A key
/// pressed in a field somebody is typing in is theirs.
pub const SCRIPT: &str = r#"document.addEventListener('click', function(e) {
    // A link followed any way at all: a click, Enter on a focused link, and
    // the default action a screen reader's Enter performs in browse mode,
    // which the browser delivers as a click. Taken before the browser
    // navigates, because the window cannot read where a navigation is going
    // (wxdragon 0.9.17 hands it the event's string, which a navigating event
    // leaves empty), so the veto it used to rely on never saw a link.
    var link = e.target && e.target.closest ? e.target.closest('a[href], area[href]') : null;
    if (!link) { return; }
    var written = link.getAttribute('href') || '';
    // A fragment moves within the document and leaves it; the browser keeps it.
    if (written.charAt(0) === '#') { return; }
    e.preventDefault();
    e.stopPropagation();
    var data = { kind: 'link', href: link.href };
    // The browser's own conventions: Ctrl for a new tab, Shift for a new
    // window, which are the browser and the separate window here.
    if (e.ctrlKey) { data.asked = 'browser'; }
    else if (e.shiftKey) { data.asked = 'separate'; }
    window.contextMenu.postMessage(JSON.stringify(data));
}, true);
document.addEventListener('keydown', function(e) {
    // Backspace and Alt+Left bring the message back when a page stands in
    // its place; whether one does is the window's decision. Not from a field
    // somebody is typing in, where Backspace is a character going.
    var target = e.target;
    var typing = target && (target.isContentEditable ||
        /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName || ''));
    var back = (e.key === 'Backspace' && !typing && !e.altKey && !e.ctrlKey) ||
        (e.key === 'ArrowLeft' && e.altKey && !e.ctrlKey);
    if (!back) { return; }
    e.preventDefault();
    e.stopPropagation();
    window.contextMenu.postMessage(JSON.stringify({ kind: 'back' }));
}, true);"#;

/// What the page posted, read from the message it sent.
///
/// `None` for the way out, the jumps, the context menu and anything
/// unreadable, which the window answers elsewhere or ignores. A `link` with
/// no address is nothing: there is nothing to route. An `asked` nobody wrote
/// is a plain activation, which follows the setting, the safest of the
/// three answers.
pub fn what_the_page_posted(json: &str) -> Option<Posted> {
    let posted = serde_json::from_str::<serde_json::Value>(json).ok()?;
    match posted.get("kind").and_then(serde_json::Value::as_str)? {
        "link" => {
            let href = posted.get("href").and_then(serde_json::Value::as_str)?;
            let asked = match posted.get("asked").and_then(serde_json::Value::as_str) {
                Some("browser") => Asked::InBrowser,
                Some("separate") => Asked::InSeparateWindow,
                _ => Asked::Activated,
            };
            Some(Posted::Link {
                href: href.to_string(),
                asked,
            })
        }
        "back" => Some(Posted::Back),
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
    fn test_a_click_on_an_anchor_is_taken_from_the_browser_and_posted_with_its_address() {
        // Before the browser navigates: the window cannot read where a
        // navigation is going, so the address has to arrive from the page.
        assert!(SCRIPT.contains("addEventListener('click'"), "{SCRIPT}");
        assert!(
            SCRIPT.contains("closest('a[href], area[href]')"),
            "{SCRIPT}"
        );
        assert!(SCRIPT.contains("e.preventDefault();"), "{SCRIPT}");
        assert!(SCRIPT.contains("kind: 'link'"), "{SCRIPT}");
        assert!(SCRIPT.contains("href: link.href"), "{SCRIPT}");

        assert_eq!(
            what_the_page_posted(r#"{"kind":"link","href":"https://example.com/where-it-went"}"#),
            Some(Posted::Link {
                href: "https://example.com/where-it-went".to_string(),
                asked: Asked::Activated,
            })
        );
    }

    #[test]
    fn test_the_browsers_own_modifiers_ask_for_the_browser_and_a_separate_window() {
        // Ctrl-click is a new tab and Shift-click a new window in every
        // browser; here they are the browser and the separate window.
        assert!(SCRIPT.contains("e.ctrlKey"), "{SCRIPT}");
        assert!(SCRIPT.contains("e.shiftKey"), "{SCRIPT}");
        assert!(SCRIPT.contains("asked = 'browser'"), "{SCRIPT}");
        assert!(SCRIPT.contains("asked = 'separate'"), "{SCRIPT}");

        assert_eq!(
            what_the_page_posted(
                r#"{"kind":"link","href":"https://a.example/","asked":"browser"}"#
            ),
            Some(Posted::Link {
                href: "https://a.example/".to_string(),
                asked: Asked::InBrowser,
            })
        );
        assert_eq!(
            what_the_page_posted(
                r#"{"kind":"link","href":"https://a.example/","asked":"separate"}"#
            ),
            Some(Posted::Link {
                href: "https://a.example/".to_string(),
                asked: Asked::InSeparateWindow,
            })
        );
        // A word nobody wrote is read as a plain activation, which follows
        // the setting: the safest of the three answers.
        assert_eq!(
            what_the_page_posted(
                r#"{"kind":"link","href":"https://a.example/","asked":"elsewhere"}"#
            ),
            Some(Posted::Link {
                href: "https://a.example/".to_string(),
                asked: Asked::Activated,
            })
        );
    }

    #[test]
    fn test_a_fragment_link_is_left_to_the_browser() {
        // `#top` moves within the document; posting it would refuse it aloud.
        assert!(SCRIPT.contains("getAttribute('href')"), "{SCRIPT}");
        assert!(SCRIPT.contains("charAt(0) === '#'"), "{SCRIPT}");
    }

    #[test]
    fn test_backspace_and_alt_left_post_the_way_back_and_not_from_a_field() {
        assert!(SCRIPT.contains("e.key === 'Backspace'"), "{SCRIPT}");
        assert!(
            SCRIPT.contains("e.key === 'ArrowLeft' && e.altKey"),
            "{SCRIPT}"
        );
        assert!(SCRIPT.contains("isContentEditable"), "{SCRIPT}");
        assert!(SCRIPT.contains("kind: 'back'"), "{SCRIPT}");

        assert_eq!(
            what_the_page_posted(r#"{"kind":"back"}"#),
            Some(Posted::Back)
        );
    }

    #[test]
    fn test_a_link_with_no_address_is_nothing() {
        assert_eq!(what_the_page_posted(r#"{"kind":"link"}"#), None);
        assert_eq!(what_the_page_posted(r#"{"kind":"link","href":7}"#), None);
    }

    #[test]
    fn test_every_kind_the_script_posts_is_one_the_window_reads() {
        let kinds = every_kind_the_script_posts();
        assert_eq!(kinds.len(), 2, "{kinds:?}");
        for kind in kinds {
            let posted = format!(r#"{{"kind":"{kind}","href":"https://a.example/"}}"#);
            assert!(
                what_the_page_posted(&posted).is_some(),
                "the script posts {kind:?} and the window reads it as nothing"
            );
        }
    }

    #[test]
    fn test_the_way_out_the_jumps_the_context_menu_and_nonsense_are_not_links() {
        for not_a_link in [
            r#"{"kind":"leave"}"#,
            r#"{"kind":"leave","back":true}"#,
            r#"{"kind":"context","x":1,"y":2,"href":"https://a.example/"}"#,
            r#"{"kind":"attachments"}"#,
            r#"{"kind":"warning"}"#,
            r#"{"href":"https://example.com"}"#,
            "not json",
            "",
        ] {
            assert_eq!(what_the_page_posted(not_a_link), None, "{not_a_link}");
        }
    }
}
