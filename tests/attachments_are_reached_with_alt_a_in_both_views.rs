//! Alt+A reaches the attachments of an open message in both views, and F8 is
//! retired from both.
//!
//! #84, 11-04.1. The tester on 2026-09-18 under NVDA, on
//! `1.0.0-alpha.1+149.g744d05ef`: in an open message F8 does not reach the
//! attachments. Under the Formatted style, the default, a message opens in
//! the page window, which bound F7 and F8 as `KEY_DOWN` on the WebView
//! itself, and a WebView keeps every key once the browser has focus, so the
//! binding never fired and the sentence read on open, "F8 for them", was
//! false there. Pratik's decision the same day: Alt+A in both message
//! windows, F8 retired there; F8 keeps Columns in the main window and the
//! toolbar in the composer.
//!
//! Read from the source rather than run, because the route is a script
//! injected into a browser control inside a window with a running event loop,
//! and what a reading can hold is the shape: the page is given the script
//! that posts the jumps, the window's handler moves focus on each and says
//! where it landed, no binding sits on the browser control, the reader's item
//! carries the new key, the way back answers it in both views, and the pages
//! say it. Each reading is a function over the text with a companion that
//! plants the opposite and requires a complaint. What no reading can see,
//! said plainly: that the key arrives and NVDA says the landing; that is the
//! tester's ear and is on the ledger.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_READER: &str = "src/presentation/wx_reader.rs";
const THE_JUMPS: &str = "src/presentation/page_jumps.rs";
const THE_SHORTCUTS: &str = "docs/KEYBOARD_SHORTCUTS.md";

fn read(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .replace("\r\n", "\n")
}

fn shipped(path: &str) -> String {
    what_ships(&read(path))
}

/// One function's text, from its signature to the closing brace at column
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

/// The part of the document under one heading, up to the next heading at the
/// same level or above it. A heading that is not there answers nothing, and a
/// reading over nothing complains, which is the right direction.
fn under(doc: &str, heading: &str) -> String {
    let level = heading.chars().take_while(|letter| *letter == '#').count();
    let mut reading = false;
    let mut kept = Vec::new();
    for line in doc.lines() {
        if line.trim_end() == heading {
            reading = true;
            continue;
        }
        if !reading {
            continue;
        }
        let here = line.chars().take_while(|letter| *letter == '#').count();
        if here > 0 && here <= level && line[here..].starts_with(' ') {
            break;
        }
        kept.push(line);
    }
    kept.join("\n")
}

/// The page window is given the script that posts the jumps, the preview is
/// not, and the script names Alt+A and F7 and no F8.
fn the_page_window_is_given_the_jumps(app: &str, jumps: &str) -> Result<(), String> {
    for (posted, key) in [("kind: 'attachments'", "Alt+A"), ("kind: 'warning'", "F7")] {
        if !jumps.contains(posted) {
            return Err(format!(
                "the page's script posts nothing for {key}: {posted} is not in it, so the key \
                 never leaves the browser"
            ));
        }
    }
    if !jumps.contains("e.key === 'F7'") || !jumps.contains("e.altKey") {
        return Err("the page's script reads neither F7 nor the Alt key".to_string());
    }
    if jumps.contains("F8") {
        return Err("the page's script still names F8, which is retired from the page".to_string());
    }
    let wiring = body_of(app, "fn wire_the_way_out(")?;
    if !wiring.contains("page_jumps::SCRIPT") {
        return Err(
            "wire_the_way_out never appends the jumps script, so no page can post a jump"
                .to_string(),
        );
    }
    if !app.contains(
        "wire_the_way_out(&page, \"conversation window\", PageKeys::TheWayOutAndTheJumps)",
    ) {
        return Err(
            "the page window is not given the jumps, so Alt+A and F7 stay inside the browser there"
                .to_string(),
        );
    }
    if !app.contains("wire_the_way_out(&preview, \"preview\", PageKeys::TheWayOut)") {
        return Err(
            "the preview is given the jumps, and it has no list and no bar to jump to".to_string(),
        );
    }
    Ok(())
}

/// The page window's handler moves focus on each jump and says where it
/// landed, says so when there is nothing to go to, and no key binding sits
/// on the browser control, where one never fires.
fn the_page_window_acts_on_each_jump(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn show_conversation_as_page(")?;
    let attachments = between(
        &body,
        "Some(page_jumps::Jump::Attachments) =>",
        "Some(page_jumps::Jump::Warning) =>",
    )?;
    if !attachments.contains("list.set_focus()") {
        return Err(
            "the attachments arm does not move focus to the list, so Alt+A says something and \
             goes nowhere"
                .to_string(),
        );
    }
    if !attachments.contains("\"Attachments, {}\"") || !attachments.contains("\"No attachments\"") {
        return Err(
            "the attachments arm does not say where it landed, or says nothing when there is no \
             list"
                .to_string(),
        );
    }
    let warning = between(&body, "Some(page_jumps::Jump::Warning) =>", "None =>")?;
    if !warning.contains("bar.set_focus()") {
        return Err("the warning arm does not move focus to the bar".to_string());
    }
    if !warning.contains("\"Security warning\"") || !warning.contains("\"No warning\"") {
        return Err(
            "the warning arm does not say where it landed, or says nothing when there is no bar"
                .to_string(),
        );
    }
    if body.contains("page.bind_internal(EventType::KEY_DOWN") {
        return Err(
            "a key is bound on the browser control, where a KEY_DOWN never fires while the \
             browser has focus; that is the binding #84 met"
                .to_string(),
        );
    }
    Ok(())
}

/// Alt+A on the page window's attachments list goes back to the message, and
/// the sentence read on open names Alt+A and not F8.
fn the_page_window_goes_back_and_says_alt_a(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn show_conversation_as_page(")?;
    let the_list = between(
        &body,
        "list.bind_internal(EventType::KEY_DOWN",
        "match (key, event.control_down())",
    )?;
    if !the_list.contains("key == 65 && event.alt_down()") || !the_list.contains("page.set_focus()")
    {
        return Err(
            "Alt+A on the page window's attachments list does not go back to the message"
                .to_string(),
        );
    }
    for sentence in ["Alt+A for it.", "Alt+A for them."] {
        if !body.contains(sentence) {
            return Err(format!(
                "the sentence read when the page window opens no longer says {sentence:?}"
            ));
        }
    }
    if body.contains("F8") {
        return Err(
            "the page window still names F8, which never reached anything there".to_string(),
        );
    }
    Ok(())
}

/// The reader's Attachments item is on Alt+A, the count sentence names it,
/// the way back answers it from the list and from the menu, and F8 is gone
/// from the file.
fn the_reader_is_on_alt_a(reader: &str) -> Result<(), String> {
    if !reader.contains("\"&Attachments\\tAlt+A\"") {
        return Err("the reader's Attachments item is not on Alt+A".to_string());
    }
    if !reader.contains("Alt+A for the list.") {
        return Err("the reader's count sentence does not name Alt+A".to_string());
    }
    let from_the_list = body_of(reader, "    fn wire_attachment_list(")?;
    if !from_the_list.contains("Some(65) if event.alt_down()") {
        return Err("Alt+A on the reader's attachments list does not go back".to_string());
    }
    let from_the_menu = body_of(reader, "    pub fn wire_menu(")?;
    if !from_the_menu.contains("Some(list) if list.has_focus()") {
        return Err(
            "the reader's menu handler does not answer Alt+A from the list, and the frame's \
             accelerator takes the chord before the list sees it"
                .to_string(),
        );
    }
    if reader.contains("F8") {
        return Err("the reader still names F8".to_string());
    }
    Ok(())
}

/// The shortcuts page says Alt+A where it described the reader, and F7.
fn the_shortcuts_page_says_alt_a(doc: &str) -> Result<(), String> {
    let section = under(doc, "### The Reader Window");
    if section.is_empty() {
        return Err(
            "docs/KEYBOARD_SHORTCUTS.md has no `### The Reader Window` section".to_string(),
        );
    }
    let row = section
        .lines()
        .find(|line| line.starts_with("| Attachments |"))
        .ok_or("the reader's table has no Attachments row")?;
    if !row.contains("`Alt+A`") || row.contains("`F8`") {
        return Err(format!("the Attachments row does not give Alt+A: {row}"));
    }
    if !section.contains("Either way, `Alt+A` reaches the attachments") {
        return Err(
            "the sentence for both views no longer says Alt+A reaches the attachments".to_string(),
        );
    }
    if !section.contains("`F7`") {
        return Err("the reader's section no longer names F7".to_string());
    }
    Ok(())
}

#[test]
fn test_the_page_window_is_given_the_jumps_and_the_preview_is_not() {
    the_page_window_is_given_the_jumps(&shipped(THE_MAIN_WINDOW), &shipped(THE_JUMPS))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_page_window_moves_focus_on_each_jump_and_binds_nothing_on_the_browser() {
    the_page_window_acts_on_each_jump(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_alt_a_goes_back_from_the_page_windows_list_and_the_open_sentence_says_alt_a() {
    the_page_window_goes_back_and_says_alt_a(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_readers_item_is_on_alt_a_and_f8_is_gone() {
    the_reader_is_on_alt_a(&shipped(THE_READER)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_shortcuts_page_gives_alt_a_for_the_reader_window() {
    the_shortcuts_page_says_alt_a(&read(THE_SHORTCUTS)).unwrap_or_else(|why| panic!("{why}"));
}

// ── Companions: each reading complains when the opposite is planted ─────────

fn with(text: &str, from: &str, to: &str) -> String {
    assert!(
        text.contains(from),
        "the companion's anchor is not in the text any more, so it plants nothing: {from}"
    );
    text.replacen(from, to, 1)
}

#[test]
fn test_the_reading_complains_when_the_script_or_the_wiring_loses_a_jump() {
    let app = shipped(THE_MAIN_WINDOW);
    let jumps = shipped(THE_JUMPS);

    let why = the_page_window_is_given_the_jumps(&app, &with(&jumps, "kind: 'attachments'", ""))
        .expect_err("a script posting nothing for Alt+A was passed over");
    assert!(why.contains("Alt+A"), "{why}");

    let why = the_page_window_is_given_the_jumps(
        &with(
            &app,
            "PageKeys::TheWayOutAndTheJumps)",
            "PageKeys::TheWayOut)",
        ),
        &jumps,
    )
    .expect_err("a page window given only the way out was passed over");
    assert!(why.contains("not given the jumps"), "{why}");
}

#[test]
fn test_the_reading_complains_when_focus_stays_put_or_a_key_is_bound_on_the_browser() {
    let app = shipped(THE_MAIN_WINDOW);

    let why = the_page_window_acts_on_each_jump(&with(&app, "list.set_focus();", ""))
        .expect_err("an attachments arm that moves no focus was passed over");
    assert!(why.contains("does not move focus"), "{why}");

    let why = the_page_window_acts_on_each_jump(&with(
        &app,
        "frame.set_sizer(sizer, true);",
        "page.bind_internal(EventType::KEY_DOWN, |_| {});\n    frame.set_sizer(sizer, true);",
    ))
    .expect_err("a key bound on the browser control was passed over");
    assert!(why.contains("never fires"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_open_sentence_says_f8() {
    let app = shipped(THE_MAIN_WINDOW);
    let why =
        the_page_window_goes_back_and_says_alt_a(&with(&app, "Alt+A for them.", "F8 for them."))
            .expect_err("an open sentence promising F8 was passed over");
    assert!(why.contains("Alt+A for them."), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_readers_item_goes_back_to_f8() {
    let reader = shipped(THE_READER);
    let why = the_reader_is_on_alt_a(&with(
        &reader,
        "\"&Attachments\\tAlt+A\"",
        "\"&Attachments\\tF8\"",
    ))
    .expect_err("a reader item on F8 was passed over");
    assert!(why.contains("not on Alt+A"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_page_gives_f8() {
    let doc = read(THE_SHORTCUTS);
    let why = the_shortcuts_page_says_alt_a(&with(
        &doc,
        "| Attachments | `Alt+A`",
        "| Attachments | `F8`",
    ))
    .expect_err("an Attachments row giving F8 was passed over");
    assert!(why.contains("Attachments row"), "{why}");
}
