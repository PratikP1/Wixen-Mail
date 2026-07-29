//! The page the message editor is.
//!
//! The composer's body is a `contenteditable` in a web view rather than a
//! `wxRichTextCtrl`, and the reason is accessibility rather than formatting.
//! `wxRichTextCtrl` is drawn by wxWidgets on every platform, so it exposes no
//! per-range accessibility attributes anywhere, which means no misspelling can
//! ever be marked, no heading can report itself, and every announcement has to
//! be made by hand and will be slightly wrong forever.
//!
//! A web view gets all of that from the engine. `spellcheck` alone produces
//! native spelling annotations on all three platforms: UIA spelling errors from
//! Chromium, `AXMarkedMisspelled` from WebKit, AT-SPI `invalid:spelling` from
//! WebKitGTK. Each screen reader then announces them itself, which is always
//! better than us doing its job.
//!
//! # The security point, which is the whole reason this file is testable
//!
//! **A reply quotes a stranger's message.** The original body came from
//! whoever sent it, and putting it into a `contenteditable` puts it into a live
//! DOM in a real browser engine. The reader could get away with less care
//! because a text control renders nothing; this cannot.
//!
//! So every quoted body goes through the sanitiser before it reaches the page,
//! and what comes back out goes through it again, because somebody can paste
//! anything into an editor and the result is sent to another person.
//!
//! # Why the keys are bound in the page
//!
//! A web view swallows keys once it has focus. That is what trapped people in
//! the preview pane. Here the page is ours, so the keys that have to escape are
//! bound in it and posted back out: Escape, and Ctrl+Enter to send. Everything
//! else is the editor's, which is what somebody typing a message wants.

use crate::presentation::html_renderer::HtmlRenderer;

/// The name the page posts messages to.
///
/// Registered with `add_script_message_handler` on the Rust side. One channel
/// with a `kind` field rather than several, because each one is a separate
/// registration to get wrong.
pub const CHANNEL: &str = "wixenEditor";

/// The element the message is typed into.
const BODY_ID: &str = "wixen-body";

/// Build the editor page for a message body.
///
/// `body` is whatever the composer starts with: empty for a new message, a
/// quoted original for a reply or forward, a saved draft. It is treated as
/// untrusted in every case, because in two of those three it is.
///
/// `language` is the spelling language as a BCP 47 tag, which the engine reads
/// from `lang` to choose its dictionary. Getting this wrong does not fail
/// loudly, it just marks the wrong words, so it comes from the same setting the
/// send-time check uses.
pub fn editor_document(body: &str, language: &str) -> String {
    let renderer = HtmlRenderer::new();
    // Sanitised whether or not it looks like markup. "Looks like markup" is a
    // judgement about a stranger's input, and the cost of getting it wrong here
    // is script execution in an editor the reader is about to type into.
    let safe_body = renderer.sanitize_html(body);
    let language = safe_language(language);

    format!(
        r#"<!DOCTYPE html>
<html lang="{language}"><head><meta charset="utf-8"><style>
html, body {{ height: 100%; margin: 0; }}
body {{
    font-family: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
    font-size: 1rem; line-height: 1.6;
    color: #1a1a1a; background: #ffffff;
}}
#{BODY_ID} {{
    min-height: 100%; padding: 12px; outline: none;
    word-wrap: break-word;
}}
/* 2.4.11 and 2.4.13. The editor is where the caret lives, so the one time it
   is not focused it still has to be obvious which control it is. */
#{BODY_ID}:focus-visible {{ outline: 3px solid #0066cc; outline-offset: -3px; }}
blockquote {{
    border-left: 3px solid #767676; margin-left: 0; padding-left: 12px;
    color: #555;
}}
img {{ max-width: 100%; height: auto; }}
@media (prefers-color-scheme: dark) {{
    body {{ background: #1e1e1e; color: #d4d4d4; }}
    blockquote {{ border-color: #9a9a9a; color: #bbb; }}
    #{BODY_ID}:focus-visible {{ outline-color: #569cd6; }}
}}
</style></head><body>
<div id="{BODY_ID}" contenteditable="true" spellcheck="true"
     role="textbox" aria-multiline="true" aria-label="Message body">{safe_body}</div>
<script>
(function () {{
  var body = document.getElementById({BODY_ID:?});
  function post(message) {{
    try {{ window.chrome.webview.postMessage(JSON.stringify(message)); }}
    catch (e) {{
      try {{ window.webkit.messageHandlers[{CHANNEL:?}].postMessage(JSON.stringify(message)); }}
      catch (e2) {{ }}
    }}
  }}
  // The keys that have to leave. Everything else belongs to the editor, which
  // is what somebody typing a message wants, and is why this is a window of
  // its own rather than a pane sharing one.
  body.addEventListener('keydown', function (event) {{
    if (event.key === 'Escape') {{
      event.preventDefault();
      post({{ kind: 'cancel' }});
    }} else if (event.key === 'Enter' && event.ctrlKey) {{
      event.preventDefault();
      post({{ kind: 'send' }});
    }} else if (event.key === 's' && event.ctrlKey) {{
      event.preventDefault();
      post({{ kind: 'save' }});
    }}
  }});
  // Focus lands in the message, at the start of it. A reply that opens with
  // the caret after the quoted original means typing below somebody else's
  // words, which is not where a reply goes.
  body.focus();
  var range = document.createRange();
  range.setStart(body, 0);
  range.collapse(true);
  var selection = window.getSelection();
  selection.removeAllRanges();
  selection.addRange(range);
}})();
</script>
</body></html>"#
    )
}

/// The script that reads the message back out of the page.
///
/// Returned as a string for `run_script` rather than run here, because the web
/// view is the only thing that can execute it and this module stays pure.
pub fn read_body_script() -> String {
    format!("document.getElementById({BODY_ID:?}).innerHTML")
}

/// The script that reads the message as plain text.
///
/// `innerText` rather than a conversion of the HTML, because it is what the
/// engine computed the reader to be looking at: the line breaks are where they
/// appear, the hidden things are absent, and the list markers are the ones on
/// screen. A message goes out as multipart/alternative, and this is the half
/// that everything which does not want HTML will show.
pub fn read_plain_script() -> String {
    format!("document.getElementById({BODY_ID:?}).innerText")
}

/// The script for one formatting command.
///
/// `execCommand` is deprecated and is still the only thing every engine
/// implements for rich editing in a `contenteditable`. When that changes this
/// is the one function to rewrite.
pub fn format_script(command: Format) -> String {
    format!(
        "document.execCommand({:?}, false, null); \
         document.getElementById({BODY_ID:?}).focus();",
        command.as_command()
    )
}

/// A formatting command the toolbar offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Bold,
    Italic,
    Underline,
    Undo,
    Redo,
}

impl Format {
    const fn as_command(self) -> &'static str {
        match self {
            Self::Bold => "bold",
            Self::Italic => "italic",
            Self::Underline => "underline",
            Self::Undo => "undo",
            Self::Redo => "redo",
        }
    }

    /// What is announced when it is applied.
    ///
    /// A formatting change nobody is told about is one that happened to
    /// somebody rather than one they made. The state is not read back from the
    /// engine, so this says what was asked for rather than what is now true,
    /// which is honest and is why it does not say "bold on".
    pub const fn spoken(self) -> &'static str {
        match self {
            Self::Bold => "Bold applied",
            Self::Italic => "Italic applied",
            Self::Underline => "Underline applied",
            Self::Undo => "Undone",
            Self::Redo => "Redone",
        }
    }
}

/// What the page asked the application to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMessage {
    Send,
    Save,
    Cancel,
}

/// Read one message posted by the page.
///
/// `None` for anything unrecognised. The page is ours, so an unknown message is
/// a bug rather than an attack, and doing nothing beats guessing which command
/// somebody meant.
pub fn parse_message(raw: &str) -> Option<EditorMessage> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    match value.get("kind")?.as_str()? {
        "send" => Some(EditorMessage::Send),
        "save" => Some(EditorMessage::Save),
        "cancel" => Some(EditorMessage::Cancel),
        _ => None,
    }
}

/// Take the plain text back out of what the page returned.
///
/// Not sanitised, because it is text rather than markup and there is nothing in
/// it left to execute. It is unwrapped for the same reason the HTML is: some
/// backends hand a script result back as a JSON string.
pub fn plain_from_editor(raw: &str) -> String {
    serde_json::from_str::<String>(raw).unwrap_or_else(|_| raw.to_string())
}

/// Take the body back out of what the page returned.
///
/// Sanitised on the way out as well as on the way in. Anything can be pasted
/// into an editor, and what comes out of this one is sent to another person, so
/// this is the last point at which it is ours to check.
pub fn body_from_editor(raw: &str) -> String {
    // `run_script` returns the value as a JSON string on some backends and
    // bare on others, so a quoted result is unwrapped before anything else
    // looks at it.
    let unquoted = serde_json::from_str::<String>(raw).unwrap_or_else(|_| raw.to_string());
    HtmlRenderer::new().sanitize_html(&unquoted)
}

/// A language tag safe to put in an attribute.
///
/// It comes from a settings file, which is a file somebody can edit. A tag with
/// a quote in it would close the attribute and start writing markup.
fn safe_language(tag: &str) -> String {
    let cleaned: String = tag
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(35)
        .collect();
    if cleaned.is_empty() {
        "en".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The opening `<html ...>` tag, which is where an injected attribute
    /// would have to land.
    fn first_tag(page: &str) -> &str {
        page.split_once("<html")
            .and_then(|(_, rest)| rest.split_once('>'))
            .map(|(tag, _)| tag)
            .unwrap_or("")
    }

    #[test]
    fn test_the_editor_asks_the_engine_to_check_spelling() {
        // The whole reason the composer is a web view. Without this the engine
        // marks nothing and no screen reader announces anything, and the
        // control swap bought nothing at all.
        let page = editor_document("", "en-GB");

        assert!(page.contains(r#"spellcheck="true""#), "{page}");
    }

    #[test]
    fn test_the_language_reaches_the_engine() {
        // The engine picks its dictionary from lang. Getting this wrong does
        // not fail loudly, it marks the wrong words, which is worse.
        let page = editor_document("", "fr-FR");

        assert!(page.contains(r#"<html lang="fr-FR""#), "{page}");
    }

    #[test]
    fn test_a_language_from_a_file_cannot_close_the_attribute() {
        // It comes from a settings file, which somebody can edit. A quote in
        // it would end the attribute and start writing markup.
        let page = editor_document("", r#"en" onload="alert(1)"#);

        // The letters survive, harmlessly, as part of the value: the tag reads
        // lang="enonloadalert1". What must not survive is anything that could
        // end the attribute and begin another, so the test is for an injected
        // attribute rather than for the word.
        assert!(
            !page.contains(" onload="),
            "an attribute was injected: {page}"
        );
        assert!(
            page.lines().next().is_some() && !first_tag(&page).contains('\''),
            "a quote reached the tag: {page}"
        );
        assert!(page.contains(r#"<html lang="enonloadalert1""#), "{page}");
    }

    #[test]
    fn test_an_empty_language_still_produces_a_valid_document() {
        let page = editor_document("", "");

        assert!(page.contains(r#"<html lang="en""#), "{page}");
    }

    #[test]
    fn test_a_quoted_reply_cannot_bring_a_script_into_the_editor() {
        // The security point. A reply quotes a stranger's message, and this
        // puts it in a live DOM in a real browser engine. The reader got away
        // with less care because a text control renders nothing; this cannot.
        let page = editor_document(
            "<p>Original</p><script>alert('x')</script><img src=x onerror=alert(1)>",
            "en",
        );

        assert!(!page.contains("alert('x')"), "script survived: {page}");
        assert!(!page.contains("onerror"), "handler survived: {page}");
        assert!(page.contains("Original"), "the quoted message was lost");
    }

    #[test]
    fn test_the_editor_has_a_name_and_says_it_takes_more_than_one_line() {
        // A contenteditable is a div until it is told otherwise. Without these
        // it reports as a group with no name, which is what a custom control
        // always does until somebody remembers.
        let page = editor_document("", "en");

        assert!(page.contains(r#"role="textbox""#), "{page}");
        assert!(page.contains(r#"aria-multiline="true""#), "{page}");
        assert!(page.contains(r#"aria-label="Message body""#), "{page}");
    }

    #[test]
    fn test_the_three_keys_that_have_to_escape_are_bound() {
        // A web view swallows keys once it has focus, which is what trapped
        // people in the preview pane. The page is ours, so the ones that have
        // to leave are bound in it.
        let page = editor_document("", "en");

        assert!(page.contains("'Escape'"), "{page}");
        assert!(page.contains("event.ctrlKey"), "{page}");
        for kind in ["'cancel'", "'send'", "'save'"] {
            assert!(page.contains(kind), "{kind} not posted: {page}");
        }
    }

    #[test]
    fn test_a_message_from_the_page_is_understood() {
        assert_eq!(
            parse_message(r#"{"kind":"send"}"#),
            Some(EditorMessage::Send)
        );
        assert_eq!(
            parse_message(r#"{"kind":"save"}"#),
            Some(EditorMessage::Save)
        );
        assert_eq!(
            parse_message(r#"{"kind":"cancel"}"#),
            Some(EditorMessage::Cancel)
        );
    }

    #[test]
    fn test_a_message_nobody_recognises_does_nothing() {
        // Doing nothing beats guessing which command somebody meant, and every
        // one of these is a command that would change or discard a message.
        for raw in [
            "",
            "not json",
            r#"{"kind":"delete-everything"}"#,
            r#"{"nope":1}"#,
            r#"{"kind":42}"#,
        ] {
            assert_eq!(parse_message(raw), None, "for {raw:?}");
        }
    }

    #[test]
    fn test_what_comes_out_of_the_editor_is_checked_too() {
        // Anything can be pasted into an editor, and what leaves this one is
        // sent to another person. This is the last point at which it is ours.
        let out = body_from_editor("<p>Hello</p><script>alert(1)</script>");

        assert!(!out.contains("<script"), "{out}");
        assert!(out.contains("Hello"), "{out}");
    }

    #[test]
    fn test_a_quoted_result_is_unwrapped_before_it_is_read() {
        // run_script returns the value as a JSON string on some backends and
        // bare on others. Reading the quoted form as markup would send a
        // message full of backslash-escaped quotes.
        let out = body_from_editor(r#""<p>Hello</p>""#);

        assert!(out.contains("Hello"), "{out}");
        assert!(!out.contains("\\\""), "the escaping survived: {out}");
    }

    #[test]
    fn test_the_message_can_be_read_as_text_as_well_as_markup() {
        // A message goes out as multipart/alternative, and this is the half
        // everything that does not want HTML shows. Without it the markup
        // itself is what a text-only reader sees, tags and all, which is what
        // swapping the control to a web view broke until this existed.
        assert!(read_plain_script().contains("innerText"));
        assert!(read_body_script().contains("innerHTML"));
    }

    #[test]
    fn test_plain_text_is_not_sanitised_because_it_is_not_markup() {
        // Nothing in it is left to execute, and running it through the
        // sanitiser would escape the angle brackets somebody deliberately
        // typed into a message about markup.
        let out = plain_from_editor("if a < b && c > d, see <notes>");

        assert_eq!(out, "if a < b && c > d, see <notes>");
    }

    #[test]
    fn test_the_plain_text_is_unwrapped_like_the_markup_is() {
        assert_eq!(plain_from_editor(r#""Hello there""#), "Hello there");
    }

    #[test]
    fn test_every_formatting_command_names_itself_when_applied() {
        // A formatting change nobody is told about is one that happened to
        // somebody rather than one they made.
        for format in [
            Format::Bold,
            Format::Italic,
            Format::Underline,
            Format::Undo,
            Format::Redo,
        ] {
            assert!(!format.spoken().is_empty(), "{format:?}");
            assert!(
                format_script(format).contains(format.as_command()),
                "{format:?}"
            );
        }
    }

    #[test]
    fn test_formatting_puts_the_caret_back_where_it_was_typing() {
        // A toolbar button takes focus. Without this, applying bold from the
        // toolbar leaves somebody on the button rather than in the message,
        // and the next thing they type goes nowhere.
        assert!(format_script(Format::Bold).contains("focus()"));
    }
}
