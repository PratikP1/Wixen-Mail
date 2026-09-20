//! A Markdown block marker typed with its space at the start of any line of
//! the message body becomes the structure it stands for (#79, LIST-18).
//!
//! The tester on 2026-09-18, under NVDA, on `1.0.0-alpha.1` at `744d05ef`:
//! typed Markdown made no heading. The probe of the same day, on this
//! machine's WebView2 runtime, made an `<h2>` on the first line of an empty
//! message at every build of the round and found the rule refusing a marker
//! on any line that is not the first: the guard in `blockMarkdown` asked
//! whether the text node had a previous sibling, and every line after the
//! first of a reply, forward or mailto body that arrived as plain text is a
//! text node after a `<br>`, as is a line after Shift+Enter, as is a line
//! after Enter on the empty first line. Refused before the marker was read,
//! with no post, no announcement and no log line, since 2026-07-29.
//!
//! Nothing in `tests/` drove the real editor page with keystrokes before
//! this file: `src/presentation/editor_page_harness.rs` runs the page's
//! recognisers in `boa`, which is where every rule bug so far had been, and
//! its own header says `execCommand` is outside what it models. The guard
//! that failed here is not a recogniser. It is a question about the document
//! the engine built, so only the engine can answer it.
//!
//! **How this drives the page.** It builds the real compose dialog through
//! `wx_compose::build_compose_dialog`, sets the shipped page the way
//! `set_body` does, and delivers each character as a `WM_CHAR` posted to the
//! window the browser gave the keyboard focus, one per timer tick, so the
//! page's `input` event fires once per character the way it does under
//! somebody's fingers. Enter, the arrows and End go as `WM_KEYDOWN` and
//! `WM_KEYUP`. Shift+Enter is the one key a posted message cannot carry,
//! because the engine reads the Shift state from the keyboard and not from
//! the message, so that step runs `insertLineBreak`, the command the key
//! runs, from script. Not `SendInput`: that lands on the foreground window,
//! which is not this dialog when a test runs it, and taking the foreground
//! would take it from whoever is using the machine. So the keys arrive as
//! posted characters and not through a screen reader's hook, and what the
//! tester hears is still his ear's; the ledger holds the by-ear steps.
//!
//! **What it asserts.** The document's `innerHTML`, read back through
//! `editor_document::read_body_script()`, and the messages the page posted,
//! read through `editor_document::parse_message` the way the composer reads
//! them. Never an announcement: the composer's arms have their own tests.
//!
//! **One function.** The budget is one `wxdragon::main` per process
//! (`tests/theme_reach.rs` explains it, and this file spends its one), so
//! every shape is a step inside one run, each asserted on its own, and the
//! failures are gathered rather than stopping at the first, so a red run
//! says which steps were red. The first thing read is how many `input`
//! events the page saw, so a run where no key arrived is told apart from a
//! run where the rule refused; and each character is waited for before the
//! next goes, for the reason `wait_for_the_key` gives.
//!
//! Runs under `WIXEN_NO_AUDIO` as CI does, and on a temporary data
//! directory in `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wixen_mail::common::types::MessageBody;
use wixen_mail::presentation::editor_document::{
    self, EditorMessage, Format, WhyAMarkerWasRefused,
};
use wixen_mail::presentation::wx_compose;
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::WebView;

const BODY_ID: &str = "wixen-body";
const TICK_MS: i32 = 30;
/// Ticks before the run says it waited too long and stops: a minute.
const GIVE_UP_AFTER_TICKS: u32 = 2000;
/// Ticks of nothing before a reading, so a posted message has arrived.
const TICKS_TO_SETTLE: u32 = 4;

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_CHAR: u32 = 0x0102;
const VK_RETURN: u16 = 0x0D;
const VK_END: u16 = 0x23;
const VK_DOWN: u16 = 0x28;

/// A reply to a message that arrived as plain text, the shape
/// `wx_compose::format_reply_body` makes: the separator under two empty
/// lines, the original under it, and every line break a bare `<br>` once
/// `escaped_plain_text` has turned it, with no block round any of them.
fn a_reply_to_plain_text() -> MessageBody {
    MessageBody::Plain("\n\n--- Original Message ---\nTheir words\non two lines".to_string())
}

/// A message holding a link, so that typing at the end of the line lands in
/// a text node after an inline element: the engine keeps typed text out of
/// an anchor, which is the one shape that reliably puts a marker mid-line in
/// a node of its own.
fn a_message_ending_in_a_link() -> MessageBody {
    MessageBody::Html("<p>See <a href=\"https://example.com/\">the page</a></p>".to_string())
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetFocus() -> isize;
    fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
}

thread_local! {
    static FOUND: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

fn descendants_of(parent: isize) -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumChildWindows(parent, collect, 0) };
    FOUND.with(|found| found.borrow().clone())
}

fn class_name(hwnd: isize) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

/// The window a key is posted to: the one this thread has the keyboard
/// focus on, which the browser sets when the page takes focus, or failing
/// that the browser's own top window under the control.
fn where_keys_go(body_editor: &WebView) -> isize {
    // SAFETY: a plain read.
    let focus = unsafe { GetFocus() };
    if focus != 0 {
        return focus;
    }
    descendants_of(body_editor.get_handle() as isize)
        .into_iter()
        .find(|hwnd| class_name(*hwnd) == "Chrome_WidgetWin_1")
        .unwrap_or(0)
}

fn describe(hwnd: isize) -> String {
    if hwnd == 0 {
        return "no window".to_string();
    }
    format!("0x{hwnd:x} ({})", class_name(hwnd))
}

fn post_char(hwnd: isize, ch: char) {
    // SAFETY: a live window handle; the message carries a character.
    unsafe {
        PostMessageW(hwnd, WM_CHAR, ch as usize, 1);
    }
}

fn post_key(hwnd: isize, vk: u16) {
    // SAFETY: a live window handle; the messages carry a virtual key.
    unsafe {
        PostMessageW(hwnd, WM_KEYDOWN, vk as usize, 1);
        PostMessageW(hwnd, WM_KEYUP, vk as usize, 0xC000_0001_u32 as i32 as isize);
    }
}

/// The wx result of a string expression is JSON, so quoted.
fn unquoted(answer: Option<String>) -> String {
    match answer {
        Some(text) => text.trim().trim_matches('"').to_string(),
        None => "(run_script answered None)".to_string(),
    }
}

fn script_with_body(rest: &str) -> String {
    format!("(function () {{ var b = document.getElementById({BODY_ID:?}); {rest} }})()")
}

/// What the page held and what it posted, read for one step's assertions.
struct Seen {
    /// The body's `innerHTML`.
    html: String,
    /// What the page posted since the last clearing, as the composer reads
    /// it, in order.
    posts: Vec<EditorMessage>,
    /// The raw text of every post since the last clearing, for the message
    /// when an assertion fails.
    raw: Vec<String>,
    /// How many `input` events the page saw since it was opened.
    inputs: usize,
}

impl Seen {
    fn formatted(&self, format: Format) -> bool {
        self.posts.contains(&EditorMessage::Formatted(format))
    }

    fn any_format_post(&self) -> bool {
        self.posts
            .iter()
            .any(|post| matches!(post, EditorMessage::Formatted(_)))
    }

    fn styled_with(&self, delimiter: &str) -> bool {
        self.posts.iter().any(
            |post| matches!(post, EditorMessage::Styled(style) if style.delimiter == delimiter),
        )
    }

    fn refused(&self, why: WhyAMarkerWasRefused) -> bool {
        self.posts.contains(&EditorMessage::BlockMarkerRefused(why))
    }

    /// What one element holds, for an element that occurs once.
    fn inside(&self, tag: &str) -> Option<&str> {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let start = self.html.find(&open)? + open.len();
        let end = self.html[start..].find(&close)? + start;
        Some(&self.html[start..end])
    }
}

/// One thing the run does on one tick.
enum Act {
    /// The name of the step every failure until the next `Step` is filed
    /// under.
    Step(&'static str),
    /// Load a body into the editor and wait for its page to be up and
    /// instrumented.
    Open(MessageBody),
    /// One character, as the person types it.
    Char(char),
    /// One key that is not a character.
    Key(u16),
    /// A script run in the page, for the one key a posted message cannot
    /// carry.
    Script(&'static str),
    /// Forget what was posted so far, so the next reading is one step's.
    ClearPosts,
    /// Ticks of nothing, so the last key's post has arrived before a
    /// reading.
    Settle,
    /// Read the body and the posts and assert on them.
    Check(fn(&Seen) -> Result<(), String>),
}

fn typed(text: &str) -> Vec<Act> {
    text.chars().map(Act::Char).collect()
}

/// A reading, after the page has settled.
fn checked(assertion: fn(&Seen) -> Result<(), String>) -> [Act; 2] {
    [Act::Settle, Act::Check(assertion)]
}

/// Where the run is between ticks.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WaitingForBrowser,
    /// The page was set; waiting for its script to be up, then instrumenting
    /// it and putting the focus in the body.
    WaitingForPage,
    /// Ticks of nothing before the next act, so posts have arrived.
    Settling(u32),
    /// A character was posted; waiting for the page's `input` event for it
    /// before the next key goes, and posting it again if it never comes.
    WaitingForInput {
        ch: char,
        /// How many `input` events the page will have seen once this one
        /// lands.
        expected: usize,
        /// The tick it was posted on, so a character the page never saw is
        /// told from one it has not seen yet.
        posted_at: u32,
        /// How many times it has been posted.
        tries: u32,
    },
    Acting,
    Done,
}

/// Ticks to wait for a character's `input` event before posting it again:
/// a second, which is many times longer than the engine takes when it is
/// not dropping the key.
const TICKS_BEFORE_A_KEY_IS_POSTED_AGAIN: u32 = 33;
/// How many times one character is posted before the step gives up on it.
const TRIES_PER_KEY: u32 = 4;

struct Run {
    acts: Vec<Act>,
    next: usize,
    phase: Phase,
    ticks: u32,
    posted: Vec<String>,
    failures: Vec<String>,
    step: &'static str,
    /// Where each key of the current step went, as the window's class, so a
    /// step that lost a key says which window it was posted to.
    delivered: Vec<String>,
    /// A tick is running. `run_script` waits through `wxYield`, which
    /// delivers the next `WM_TIMER` into the middle of this one; a nested
    /// tick returns at once rather than running the same act twice.
    busy: bool,
}

fn the_steps() -> Vec<Act> {
    let mut acts = Vec::new();

    // 1. The first line of an empty message: the calibration case, green
    // at every build. A run where this fails with no input events seen is
    // a run where no key arrived, and says so.
    acts.push(Act::Step("1: ## on the first line of an empty message"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("## Heading"));
    acts.extend(checked(|seen| {
        if seen.inputs == 0 {
            return Err(
                "the page saw no input event at all, so no key reached it: a delivery \
                 failure, not a rule failure"
                    .to_string(),
            );
        }
        expect(
            seen.inside("h2") == Some("Heading"),
            seen,
            "an <h2> holding Heading",
        )?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 2. A line after a line break in a reply that arrived as plain text:
    // the case the tester was most likely in. Down from the first line
    // lands between the two <br>s above the separator.
    acts.push(Act::Step(
        "2: ## on a line after a <br> in a reply to plain text",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_DOWN));
    acts.extend(typed("## Heading"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("h2") == Some("Heading"),
            seen,
            "an <h2> holding Heading alone",
        )?;
        expect(
            seen.html.contains("--- Original Message ---")
                && seen.html.contains("Their words<br>on two lines"),
            seen,
            "the quoted original still there and still text",
        )?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 3. Enter on the empty first line of the same reply, then the marker.
    acts.push(Act::Step(
        "3: ## after Enter on the empty first line of a reply",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_RETURN));
    acts.extend(typed("## x"));
    acts.extend(checked(|seen| {
        expect(seen.inside("h2") == Some("x"), seen, "an <h2> holding x")?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 4. A word, Shift+Enter, then the marker, in a new message.
    acts.push(Act::Step("4: ## after Shift+Enter in a new message"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("Hello"));
    acts.push(Act::Script(
        "document.execCommand('insertLineBreak'); 'broke the line'",
    ));
    acts.extend(typed("## x"));
    acts.extend(checked(|seen| {
        expect(seen.inside("h2") == Some("x"), seen, "an <h2> holding x")?;
        expect(
            seen.html.contains("Hello") && !seen.html.contains("<h2>Hello"),
            seen,
            "Hello still there and outside the heading",
        )?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 5. No space after the marker: not a marker, and never was. Pins the
    // space as the trigger so a later change cannot widen the rule.
    acts.push(Act::Step("5: ## with no space after it"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("##Heading "));
    acts.extend(checked(|seen| {
        expect(
            seen.html.contains("##Heading") && !seen.html.contains("<h2"),
            seen,
            "the text left as it was typed",
        )?;
        expect(!seen.any_format_post(), seen, "no format post")?;
        expect(
            !seen.refused(WhyAMarkerWasRefused::NotAtTheStartOfItsLine),
            seen,
            "no refusal either, since nothing was a marker",
        )
    }));

    // 6. After step 2's heading: bold and a bullet on fresh lines, so the
    // inline rule and the list rule are read on the real engine too.
    acts.push(Act::Step(
        "6: **bold** and - item on fresh lines after a heading",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_DOWN));
    acts.extend(typed("## Heading"));
    acts.push(Act::Key(VK_RETURN));
    acts.push(Act::ClearPosts);
    acts.extend(typed("**bold** "));
    acts.push(Act::Key(VK_RETURN));
    acts.extend(typed("- item"));
    acts.extend(checked(|seen| {
        expect(
            seen.html.contains("<strong>bold"),
            seen,
            "a <strong> holding bold",
        )?;
        expect(
            seen.html.contains("<ul><li>") && seen.html.contains("item</"),
            seen,
            "a bulleted list holding item",
        )?;
        expect(seen.styled_with("**"), seen, "a style post for Bold")?;
        expect(
            seen.formatted(Format::BulletList),
            seen,
            "a format post for Bulleted list",
        )
    }));

    // 7. What a refusal posts, and what an ordinary space does not. A
    // marker after words on the same line is not a marker, and posts
    // nothing: every ordinary space would otherwise post. A marker in a
    // node of its own that is not at the start of its line is a marker the
    // rule refused, and says so on the wire so the log can say which path.
    acts.push(Act::Step(
        "7a: ## after words on the same line posts nothing",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_DOWN));
    acts.extend(typed("Their ## "));
    acts.extend(checked(|seen| {
        expect(
            seen.html.contains("Their ## ") || seen.html.contains("Their ##&nbsp;"),
            seen,
            "the words left as typed",
        )?;
        expect(
            seen.raw.iter().all(|raw| raw.contains("\"word\"")),
            seen,
            "no post but the words",
        )
    }));
    acts.push(Act::Step(
        "7b: ## in a node of its own after a link posts the refusal",
    ));
    acts.push(Act::Open(a_message_ending_in_a_link()));
    acts.push(Act::Key(VK_END));
    acts.extend(typed("## "));
    acts.extend(checked(|seen| {
        expect(
            !seen.html.contains("<h2"),
            seen,
            "no heading made of a marker mid-line",
        )?;
        expect(!seen.any_format_post(), seen, "no format post")?;
        expect(
            seen.refused(WhyAMarkerWasRefused::NotAtTheStartOfItsLine),
            seen,
            "a refused post saying the marker was not at the start of its line",
        )
    }));

    // 8. Text after a closing inline delimiter is plain. The probe of
    // 2026-09-18 found the caret continuing the <strong> the applier had
    // made, so the space and every line after it stayed bold; the applier
    // has to release the style as well as move the caret, and a code span
    // has no command to release it with, so it is read as well.
    acts.push(Act::Step("8a: the word after **bold** is not bold"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("**bold** next"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("strong") == Some("bold"),
            seen,
            "a <strong> holding bold alone",
        )?;
        expect(
            seen.html.contains("</strong>") && seen.html.ends_with("next"),
            seen,
            "next outside the <strong>, at the end",
        )?;
        expect(seen.styled_with("**"), seen, "a style post for Bold")
    }));
    acts.push(Act::Step("8b: the word after `code` is not code"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("`code` after"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("code") == Some("code"),
            seen,
            "a <code> holding code alone",
        )?;
        expect(
            seen.html.contains("</code>") && seen.html.ends_with("after"),
            seen,
            "after outside the <code>, at the end",
        )?;
        expect(seen.styled_with("`"), seen, "a style post for Code")
    }));

    // 9. A list marker as the first thing in an empty message. July's note
    // on `block()` says `insertUnorderedList` did nothing on an empty root
    // from the menu; the typed rule is measured here.
    acts.push(Act::Step(
        "9: - item as the first thing in an empty message",
    ));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("- item"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("li") == Some("item"),
            seen,
            "a list item holding item",
        )?;
        expect(
            seen.formatted(Format::BulletList),
            seen,
            "a format post for Bulleted list",
        )
    }));

    // 8c. Enter right after the closing delimiter: the engine carries the
    // style onto the new line, which is the "every line after it" half of
    // the side finding, so the first thing typed there has to be plain too.
    acts.push(Act::Step(
        "8c: the line after **bold** and Enter is not bold",
    ));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("**bold**"));
    acts.push(Act::Key(VK_RETURN));
    acts.extend(typed("- item"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("strong") == Some("bold"),
            seen,
            "a <strong> holding bold alone",
        )?;
        expect(
            seen.inside("li") == Some("item"),
            seen,
            "a list item holding item with no style round it",
        )
    }));

    acts
}

fn expect(held: bool, seen: &Seen, what: &str) -> Result<(), String> {
    if held {
        Ok(())
    } else {
        Err(format!(
            "expected {what}; the body holds {:?} and the page posted {:?}",
            seen.html, seen.raw
        ))
    }
}

fn say(line: &str) {
    println!("{line}");
}

#[test]
fn test_a_marker_typed_at_the_start_of_any_line_makes_its_structure() {
    let data_dir = tempfile::tempdir().expect("a temporary data directory");
    // SAFETY: set before any thread is started and before anything reads it.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data_dir.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }

    let outcome: Arc<Mutex<Result<(), Vec<String>>>> = Arc::new(Mutex::new(Ok(())));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let widgets = wx_compose::build_compose_dialog(
                &frame,
                "Compose New Message",
                &["person@example.com".to_string()],
                0,
                None,
            );
            let dialog = widgets.dialog;
            let body_editor = widgets.body_editor;
            let browser = widgets.browser.clone();
            let run = Rc::new(RefCell::new(Run {
                acts: the_steps(),
                next: 0,
                phase: Phase::WaitingForBrowser,
                ticks: 0,
                posted: Vec::new(),
                failures: Vec::new(),
                step: "before the first step",
                delivered: Vec::new(),
                busy: false,
            }));

            body_editor.on_script_message_received({
                let run = run.clone();
                move |event| {
                    if let Some(raw) = event.get_string() {
                        run.borrow_mut().posted.push(raw);
                    }
                }
            });

            dialog.show(true);

            let ticker = Rc::new(Timer::new(&dialog));
            ticker.on_tick({
                let run = run.clone();
                let outcome = outcome.clone();
                let browser = browser.clone();
                let ticker = ticker.clone();
                move |_| {
                    let phase = {
                        let mut run = run.borrow_mut();
                        if run.busy {
                            return;
                        }
                        run.busy = true;
                        run.ticks += 1;
                        if run.ticks > GIVE_UP_AFTER_TICKS && run.phase != Phase::Done {
                            let step = run.step;
                            run.failures
                                .push(format!("{step}: the run waited too long and gave up"));
                            run.phase = Phase::Done;
                            run.next = run.acts.len();
                        }
                        run.phase
                    };
                    let next = match phase {
                        Phase::WaitingForBrowser => {
                            if browser.is_ready() {
                                Phase::Acting
                            } else {
                                phase
                            }
                        }
                        Phase::WaitingForPage => {
                            if the_page_is_up(&body_editor) {
                                instrument(&body_editor);
                                body_editor.set_focus();
                                Phase::Settling(TICKS_TO_SETTLE)
                            } else {
                                phase
                            }
                        }
                        Phase::Settling(left) => {
                            if left > 1 {
                                Phase::Settling(left - 1)
                            } else {
                                Phase::Acting
                            }
                        }
                        Phase::WaitingForInput {
                            ch,
                            expected,
                            posted_at,
                            tries,
                        } => wait_for_the_key(&run, &body_editor, ch, expected, posted_at, tries),
                        Phase::Acting => one_act(&run, &body_editor),
                        Phase::Done => Phase::Done,
                    };
                    let finished = next == Phase::Done && phase != Phase::Done;
                    {
                        let mut run = run.borrow_mut();
                        run.phase = next;
                        run.busy = false;
                    }
                    if finished {
                        let failures = std::mem::take(&mut run.borrow_mut().failures);
                        *outcome.lock().unwrap() = if failures.is_empty() {
                            Ok(())
                        } else {
                            Err(failures)
                        };
                        // Stopped before its owner goes: a timer firing on a
                        // destroyed dialog is an access violation.
                        ticker.stop();
                        browser.destroy_when_ready(dialog);
                        wxdragon::call_after(Box::new(move || {
                            app.exit_main_loop();
                        }));
                    }
                }
            });
            ticker.start(TICK_MS, false);
            // Dropping the last handle destroys the timer, and it must
            // outlive on_init. The tick closure holds the other handle.
            std::mem::forget(ticker);
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
    let outcome = outcome.lock().unwrap();
    if let Err(failures) = &*outcome {
        panic!(
            "{} step(s) did not hold on the real page:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

/// Whether the shipped page's script is up and it is a page this run has
/// not instrumented yet, which is how a fresh page is told from the one
/// before it after `set_page`.
fn the_page_is_up(body_editor: &WebView) -> bool {
    unquoted(body_editor.run_script(
        "(typeof window.wixenRules === 'object' && !window.__inputs_seen) ? 'up' : 'not yet'",
    )) == "up"
}

/// Count the page's `input` events, so a run where no key arrived is told
/// apart from one where a rule refused.
fn instrument(body_editor: &WebView) {
    let _ = body_editor.run_script(&script_with_body(
        "window.__inputs_seen = 0; \
         b.addEventListener('input', function () { window.__inputs_seen++; }); \
         return 'instrumented';",
    ));
}

/// How many `input` events the page has seen since it was instrumented.
fn inputs_seen(body_editor: &WebView) -> usize {
    unquoted(body_editor.run_script("String(window.__inputs_seen || 0)"))
        .parse()
        .unwrap_or(0)
}

/// Post one character and wait for the page to see it.
fn post_a_char(
    run: &Rc<RefCell<Run>>,
    body_editor: &WebView,
    ch: char,
    expected: usize,
    tries: u32,
) -> Phase {
    let target = where_keys_go(body_editor);
    let posted_at = {
        let mut run = run.borrow_mut();
        let went = if tries == 1 {
            format!("{ch:?} to {}", describe(target))
        } else {
            let again = format!("{ch:?} again, try {tries}, to {}", describe(target));
            say(&format!("POSTED AGAIN {again}"));
            again
        };
        run.delivered.push(went);
        run.ticks
    };
    post_char(target, ch);
    Phase::WaitingForInput {
        ch,
        expected,
        posted_at,
        tries,
    }
}

/// Whether the character posted has reached the page, and what to do when
/// it has not.
///
/// Posted one per tick with nothing waited for, the engine dropped a
/// character now and then in the tenth of a second after it had rebuilt the
/// line: measured on 2026-09-20, three runs in ten lost two or three
/// characters right after an Enter or right after a marker had made its
/// block, with every key posted to the same window. A person's keys go
/// through the input method's own queue and a posted message does not. So
/// each character is waited for before the next goes, which was twenty runs
/// green with no character posted twice; and one the page has not seen
/// after a second is posted again, with a line saying so, so a run that
/// needed it is not read as a clean one.
fn wait_for_the_key(
    run: &Rc<RefCell<Run>>,
    body_editor: &WebView,
    ch: char,
    expected: usize,
    posted_at: u32,
    tries: u32,
) -> Phase {
    if inputs_seen(body_editor) >= expected {
        return Phase::Acting;
    }
    let now = run.borrow().ticks;
    if now - posted_at < TICKS_BEFORE_A_KEY_IS_POSTED_AGAIN {
        return Phase::WaitingForInput {
            ch,
            expected,
            posted_at,
            tries,
        };
    }
    if tries >= TRIES_PER_KEY {
        let step = run.borrow().step;
        run.borrow_mut().failures.push(format!(
            "{step}: the page never saw {ch:?} after {tries} postings, so the step's keys \
             did not all arrive"
        ));
        return Phase::Acting;
    }
    post_a_char(run, body_editor, ch, expected, tries + 1)
}

fn read(run: &Rc<RefCell<Run>>, body_editor: &WebView) -> Seen {
    let html = unquoted(body_editor.run_script(&editor_document::read_body_script()));
    let inputs = inputs_seen(body_editor);
    let raw = run.borrow().posted.clone();
    let posts = raw
        .iter()
        .filter_map(|raw| editor_document::parse_message(raw))
        .collect();
    Seen {
        html,
        posts,
        raw,
        inputs,
    }
}

/// One act, and the phase after it.
fn one_act(run: &Rc<RefCell<Run>>, body_editor: &WebView) -> Phase {
    let index = run.borrow().next;
    if index >= run.borrow().acts.len() {
        return Phase::Done;
    }
    run.borrow_mut().next = index + 1;
    // The act is borrowed for the match and released before anything that
    // borrows the run again.
    let act = std::mem::replace(&mut run.borrow_mut().acts[index], Act::ClearPosts);
    match act {
        Act::Step(name) => {
            say(&format!("STEP {name}"));
            let mut run = run.borrow_mut();
            run.step = name;
            run.delivered.clear();
            Phase::Acting
        }
        Act::Open(body) => {
            run.borrow_mut().posted.clear();
            body_editor.set_page(&editor_document::editor_document(&body, "en", true), "");
            Phase::WaitingForPage
        }
        Act::Char(ch) => {
            let expected = inputs_seen(body_editor) + 1;
            post_a_char(run, body_editor, ch, expected, 1)
        }
        Act::Key(vk) => {
            let target = where_keys_go(body_editor);
            run.borrow_mut()
                .delivered
                .push(format!("key 0x{vk:02x} to {}", describe(target)));
            post_key(target, vk);
            Phase::Settling(TICKS_TO_SETTLE)
        }
        Act::Script(script) => {
            let _ = body_editor.run_script(script);
            Phase::Settling(TICKS_TO_SETTLE)
        }
        Act::ClearPosts => {
            run.borrow_mut().posted.clear();
            Phase::Acting
        }
        Act::Settle => Phase::Settling(TICKS_TO_SETTLE),
        Act::Check(assertion) => {
            let seen = read(run, body_editor);
            let step = run.borrow().step;
            match assertion(&seen) {
                Ok(()) => say(&format!("HELD {step}: {:?}", seen.html)),
                Err(why) => {
                    let delivered = run.borrow().delivered.join(", ");
                    say(&format!("RED {step}: {why}; the keys went: {delivered}"));
                    run.borrow_mut()
                        .failures
                        .push(format!("{step}: {why}; the keys went: {delivered}"));
                }
            }
            Phase::Acting
        }
    }
}
