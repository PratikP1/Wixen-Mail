//! A signature follows the From account, and is chosen in two places that
//! show one answer (#43).
//!
//! The tester on 2026-09-15: "Allow signatures to be assigned by email
//! account. Default should apply if no signature is assigned to a particular
//! account." Pratik on 2026-09-23: the choice is made on the account's own
//! dialog and in the Signature Manager, one stored setting reachable from
//! both, so what either sets is what the other shows the next time it opens.
//!
//! **What is read.** A store built in the test with two accounts and three
//! signatures, one assigned to the first account and one the default. The
//! store's answer for each account; the Signature Manager's rows, built
//! through `wx_managers` and read off the live list; the account dialog's
//! choice, built and read back; each surface after the other has written;
//! the default set and cleared. Then the real composer: its page loaded the
//! way the composer loads it, the From account changed through the control
//! with the key a person presses, and the page's markup read back.
//!
//! **Companions.** Each reading is a check that can be handed a wrong state.
//! The companions hand it one, a swap over an edited signature, a manager
//! showing one account's signatures, and an account dialog that did not read
//! what the manager wrote, and are refused, so a reading that passes is one
//! that could have failed.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/event_times_move_in_blocks.rs` uses; the
//! composer's page is waited for on a timer, on the shape
//! `tests/a_marker_counts_at_the_start_of_any_line.rs` uses, since a page is
//! only there once the browser has made it. Runs under `WIXEN_NO_AUDIO` as
//! CI does, and on a temporary data directory in `WIXEN_MAIL_DATA`, never a
//! person's profile.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::Account;
use wixen_mail::data::message_cache::{MessageCache, Signature};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::editor_document;
use wixen_mail::presentation::managers::{
    save_what_the_signature_manager_returned, the_signature_managers_rows,
};
use wixen_mail::presentation::wx_account_manager::{
    SignatureChoices, build_account_edit_dialog, keep_the_signature_choice,
};
use wixen_mail::presentation::wx_compose::{self, SignatureFor};
use wixen_mail::presentation::wx_managers::{
    ManagedRow, SignatureAccount, SignatureEntry, build_sig_edit_dialog, build_signature_manager,
    offers_for, the_signature_as_edited,
};
use wxdragon::prelude::*;
use wxdragon::widgets::WebView;

type Harvest = BTreeMap<&'static str, String>;

const WORK: &str = "acct-work";
const HOME: &str = "acct-home";
/// An account nobody assigned anything to, for the default's readings.
const NOBODY: &str = "acct-nobody";

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const VK_UP: usize = 0x26;
const VK_DOWN: usize = 0x28;

const TICK_MS: i32 = 30;
/// Ticks before the run says it waited too long: a minute.
const GIVE_UP_AFTER_TICKS: u32 = 2000;
/// Ticks of nothing after a key, so what it started has finished.
const TICKS_TO_SETTLE: u32 = 6;

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
}

// ── The store ─────────────────────────────────────────────────────────────

fn the_accounts() -> Vec<SignatureAccount> {
    vec![
        SignatureAccount {
            id: WORK.to_string(),
            name: "Work".to_string(),
        },
        SignatureAccount {
            id: HOME.to_string(),
            name: "Home".to_string(),
        },
    ]
}

fn an_account(id: &str, name: &str) -> Account {
    Account {
        id: id.to_string(),
        name: name.to_string(),
        email: format!("{}@example.com", name.to_lowercase()),
        ..Account::default()
    }
}

fn a_signature(id: &str, account_id: &str, name: &str, text: &str) -> Signature {
    Signature {
        id: id.to_string(),
        account_id: account_id.to_string(),
        name: name.to_string(),
        content_plain: text.to_string(),
        content_html: None,
        is_default: false,
        created_at: "2026-09-24T00:00:00Z".to_string(),
    }
}

/// Two accounts and three signatures: Work's own, the default, and Brief,
/// which nobody uses yet. Two written while Work was active and one while
/// Home was, so a manager that listed one account's would show a part.
fn a_store(at: &std::path::Path) -> Result<MessageCache, String> {
    let cache = MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))?;
    for signature in [
        a_signature("sig-work", WORK, "Work signature", "Regards, Work"),
        a_signature("sig-home", HOME, "Home signature", "Cheers, Home"),
        a_signature("sig-brief", WORK, "Brief", "B."),
    ] {
        cache
            .create_signature(&signature)
            .map_err(|e| format!("{e}"))?;
    }
    cache
        .assign(WORK, Some("sig-work"))
        .map_err(|e| format!("{e}"))?;
    cache
        .set_the_default(Some("sig-home"))
        .map_err(|e| format!("{e}"))?;
    Ok(cache)
}

fn signing(cache: &MessageCache, account: &str) -> Result<String, String> {
    Ok(cache
        .signature_for_account(account)
        .map_err(|e| format!("{e}"))?
        .map_or_else(|| "none".to_string(), |signature| signature.name))
}

// ── The surfaces ──────────────────────────────────────────────────────────

fn the_rows(cache: &MessageCache) -> Result<Vec<SignatureEntry>, String> {
    the_signature_managers_rows(cache, &the_accounts()).map_err(|e| format!("{e}"))
}

/// The Signature Manager's list as it stands, one line per row: name,
/// default, used by.
fn the_manager_as_shown(frame: &Frame, cache: &MessageCache) -> Result<String, String> {
    let manager = build_signature_manager(frame, &the_rows(cache)?, None);
    let rows: Vec<String> = (0..manager.list.get_item_count() as i64)
        .map(|row| {
            (0..3)
                .map(|column| manager.list.get_item_text(row, column))
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect();
    manager.dialog.destroy();
    Ok(rows.join("\n"))
}

/// The account dialog's signature choice for one account: what it opens on,
/// and its first entry.
fn the_account_dialog(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
    account: &Account,
) -> Result<(String, String), String> {
    let choices = SignatureChoices::read(cache, Some(&account.id)).map_err(|e| format!("{e}"))?;
    let dialog = build_account_edit_dialog(frame, Some(account), a11y, None, &choices);
    let opens_on = dialog
        .signature_choice
        .get_string_selection()
        .unwrap_or_default();
    let first = dialog.signature_choice.get_string(0).unwrap_or_default();
    dialog.dialog.destroy();
    Ok((opens_on, first))
}

/// A signature's editor opened on `name`, changed by `change`, and OK
/// pressed: read back, settled among the rows as the manager settles it, and
/// saved through the manager's save.
fn edit_in_the_manager(
    frame: &Frame,
    cache: &MessageCache,
    name: &str,
    change: impl Fn(&wixen_mail::presentation::wx_managers::SigEditWidgets),
) -> Result<(), String> {
    let accounts = the_accounts();
    let opened = the_rows(cache)?;
    let at = opened
        .iter()
        .position(|row| row.name == name)
        .ok_or_else(|| format!("no row named {name}"))?;
    let offers = offers_for(&accounts, Some(&opened[at]), &opened);
    let editor = build_sig_edit_dialog(frame, Some(&opened[at]), &offers, None);
    change(&editor);
    let edited = the_signature_as_edited(&editor, Some(&opened[at]));
    editor.dialog.destroy();
    let mut returned = opened.clone();
    returned[at] = edited;
    SignatureEntry::settle(&mut returned, at);
    let failures =
        save_what_the_signature_manager_returned(cache, WORK, &opened, returned, &accounts);
    match failures.is_empty() {
        true => Ok(()),
        false => Err(failures.join("; ")),
    }
}

/// Whether a signature's editor opens with an account's box ticked.
fn the_editor_ticks(
    frame: &Frame,
    cache: &MessageCache,
    name: &str,
    account: &str,
) -> Result<String, String> {
    let accounts = the_accounts();
    let rows = the_rows(cache)?;
    let row = rows
        .iter()
        .find(|row| row.name == name)
        .ok_or_else(|| format!("no row named {name}"))?;
    let editor = build_sig_edit_dialog(
        frame,
        Some(row),
        &offers_for(&accounts, Some(row), &rows),
        None,
    );
    let ticked = editor
        .account_boxes
        .iter()
        .find(|(offered, _)| offered.id == account)
        .map_or_else(
            || "no box".to_string(),
            |(_, check)| check.get_value().to_string(),
        );
    editor.dialog.destroy();
    Ok(ticked)
}

/// A reading, or what stopped it being taken. A step that cannot be taken
/// fails the readings that depend on it and not the rest of the session.
fn or_why(result: Result<String, String>) -> String {
    result.unwrap_or_else(|why| format!("(not read: {why})"))
}

/// Work's own dialog set to Brief and saved.
fn choose_brief_on_works_dialog(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
    work: &Account,
) -> Result<(), String> {
    let choices = SignatureChoices::read(cache, Some(WORK)).map_err(|e| format!("{e}"))?;
    let dialog = build_account_edit_dialog(frame, Some(work), a11y, None, &choices);
    let brief = (0..dialog.signature_choice.get_count())
        .find(|at| dialog.signature_choice.get_string(*at).as_deref() == Some("Brief"));
    let kept = match brief {
        Some(brief) => {
            dialog.signature_choice.set_selection(brief);
            keep_the_signature_choice(cache, WORK, &dialog, &choices).map_err(|e| format!("{e}"))
        }
        None => Err("Work's dialog does not offer Brief".to_string()),
    };
    dialog.dialog.destroy();
    kept
}

fn read_the_surfaces(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
    into: &mut Harvest,
) {
    let work = an_account(WORK, "Work");
    let home = an_account(HOME, "Home");
    let dialog = |account: &Account| the_account_dialog(frame, a11y, cache, account);

    into.insert("the store: Work signs with", or_why(signing(cache, WORK)));
    into.insert("the store: Home signs with", or_why(signing(cache, HOME)));
    into.insert(
        "the manager, opened",
        or_why(the_manager_as_shown(frame, cache)),
    );
    into.insert(
        "Work's dialog, opened on",
        or_why(dialog(&work).map(|(opens_on, _)| opens_on)),
    );
    into.insert(
        "Work's dialog, first entry",
        or_why(dialog(&work).map(|(_, first)| first)),
    );

    // The manager writes, the account dialog reads: Brief given to Home in
    // Brief's editor.
    let given = edit_in_the_manager(frame, cache, "Brief", |editor| {
        for (account, check) in &editor.account_boxes {
            if account.id == HOME {
                check.set_value(true);
            }
        }
    });
    into.insert(
        "Home's dialog, after Brief's editor gave it Home",
        or_why(given.and_then(|()| dialog(&home).map(|(opens_on, _)| opens_on))),
    );

    // The account dialog writes, the manager reads: Work set to Brief on
    // Work's own dialog and saved.
    let chosen = choose_brief_on_works_dialog(frame, a11y, cache, &work);
    into.insert(
        "the manager, after Work's dialog chose Brief",
        or_why(
            chosen
                .clone()
                .and_then(|()| the_manager_as_shown(frame, cache)),
        ),
    );
    into.insert(
        "Brief's editor, Work's box, after Work's dialog chose Brief",
        or_why(chosen.and_then(|()| the_editor_ticks(frame, cache, "Brief", WORK))),
    );

    // The default, set and cleared in Brief's editor.
    let set = edit_in_the_manager(frame, cache, "Brief", |editor| {
        editor.def_check.set_value(true)
    });
    into.insert(
        "Work's dialog, first entry, with Brief the default",
        or_why(
            set.clone()
                .and_then(|()| dialog(&work).map(|(_, first)| first)),
        ),
    );
    into.insert(
        "the store: nobody's, with Brief the default",
        or_why(set.and_then(|()| signing(cache, NOBODY))),
    );
    let cleared = edit_in_the_manager(frame, cache, "Brief", |editor| {
        editor.def_check.set_value(false)
    });
    into.insert(
        "Work's dialog, first entry, with no default",
        or_why(
            cleared
                .clone()
                .and_then(|()| dialog(&work).map(|(_, first)| first)),
        ),
    );
    into.insert(
        "the store: nobody's, with no default",
        or_why(cleared.and_then(|()| signing(cache, NOBODY))),
    );
}

// ── The composer ──────────────────────────────────────────────────────────

/// A reply's body the way the composer quotes one written as a page.
fn a_reply() -> MessageBody {
    MessageBody::Html("<p><br></p><p>--- Original Message ---</p><p>Their words</p>".to_string())
}

/// One thing the composer's run does.
enum Act {
    /// Load a message into the page the way the composer does, signed with
    /// this text, and wait for the page.
    Open(MessageBody, &'static str),
    /// A script run in the page, standing for somebody's typing.
    Type(&'static str),
    /// The body's markup, read into the harvest under this name.
    Read(&'static str),
    /// A key pressed on the From account, then ticks of nothing.
    Press(usize),
}

fn the_composers_acts() -> Vec<Act> {
    vec![
        // A new message from Work, and From changed to Home.
        Act::Open(MessageBody::Plain(String::new()), "Regards, Work"),
        Act::Read("a new message, opened"),
        Act::Press(VK_DOWN),
        Act::Read("a new message, after From changed to Home"),
        // A new message from Home whose signature somebody typed into, and
        // From changed back to Work.
        Act::Open(MessageBody::Plain(String::new()), "Cheers, Home"),
        Act::Type("b.innerHTML = b.innerHTML.replace('Cheers, Home', 'Cheers, Home team');"),
        Act::Read("an edited message, before From changed"),
        Act::Press(VK_UP),
        Act::Read("an edited message, after From changed to Work"),
        // A reply from Work, and From changed to Home.
        Act::Open(a_reply(), "Regards, Work"),
        Act::Press(VK_DOWN),
        Act::Read("a reply, after From changed to Home"),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WaitingForBrowser,
    WaitingForPage,
    Settling(u32),
    Acting,
    Done,
}

struct Run {
    acts: Vec<Act>,
    next: usize,
    phase: Phase,
    ticks: u32,
    busy: bool,
    harvest: Harvest,
    failure: Option<String>,
}

fn unquoted(answer: Option<String>) -> String {
    answer
        .map(|text| editor_document::plain_from_editor(&text))
        .unwrap_or_else(|| "(run_script answered None)".to_string())
}

fn the_page_is_up(body_editor: &WebView) -> bool {
    unquoted(body_editor.run_script(
        "(typeof window.wixenRules === 'object' && !window.__signature_run) ? 'up' : 'not yet'",
    )) == "up"
}

fn press(choice: &Choice, key: usize) {
    let hwnd = choice.get_handle() as isize;
    // SAFETY: a live window on this thread; the messages carry a key.
    unsafe {
        SendMessageW(hwnd, WM_KEYDOWN, key, 0x0100_0001);
        SendMessageW(hwnd, WM_KEYUP, key, 0xC100_0001_u32 as i32 as isize);
    }
}

fn one_act(run: &Rc<RefCell<Run>>, body_editor: &WebView, choice: &Choice) -> Phase {
    let index = run.borrow().next;
    if index >= run.borrow().acts.len() {
        return Phase::Done;
    }
    run.borrow_mut().next = index + 1;
    let act = std::mem::replace(&mut run.borrow_mut().acts[index], Act::Read("spent"));
    match act {
        Act::Open(body, signature) => {
            let signed = wx_compose::with_signature(&body, signature);
            body_editor.set_page(&editor_document::editor_document(&signed, "en", false), "");
            Phase::WaitingForPage
        }
        Act::Type(script) => {
            let _ = body_editor.run_script(&format!(
                "(function () {{ var b = document.getElementById('wixen-body'); {script} return 'typed'; }})()"
            ));
            Phase::Settling(TICKS_TO_SETTLE)
        }
        Act::Read(name) => {
            let html = unquoted(body_editor.run_script(&editor_document::read_body_script()));
            run.borrow_mut().harvest.insert(name, html);
            Phase::Acting
        }
        Act::Press(key) => {
            press(choice, key);
            Phase::Settling(TICKS_TO_SETTLE)
        }
    }
}

// ── The session ───────────────────────────────────────────────────────────

fn take_the_harvest() -> Result<Harvest, String> {
    let data = tempfile::tempdir().map_err(|e| format!("a data directory: {e}"))?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read either yet.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let store_at = tempfile::tempdir().map_err(|e| format!("a store directory: {e}"))?;
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        let store_at = store_at.path().to_path_buf();
        wxdragon::main(move |app| {
            let finish = {
                let outcome = outcome.clone();
                move |taken: Result<Harvest, String>| {
                    if let Ok(mut slot) = outcome.lock() {
                        *slot = Some(taken);
                    }
                    wxdragon::call_after(Box::new(move || {
                        app.exit_main_loop();
                    }));
                }
            };
            let frame = Frame::builder()
                .with_title("A signature follows the From account, the reading")
                .build();
            let a11y = match Accessibility::new() {
                Ok(a11y) => Arc::new(a11y),
                Err(why) => return finish(Err(format!("accessibility: {why:?}"))),
            };
            let mut harvest = Harvest::new();
            match a_store(&store_at) {
                Ok(cache) => read_the_surfaces(&frame, &a11y, &cache, &mut harvest),
                Err(why) => return finish(Err(why)),
            }

            let widgets = wx_compose::build_compose_dialog(
                &frame,
                "Compose New Message",
                &[
                    "work@example.com".to_string(),
                    "home@example.com".to_string(),
                ],
                0,
                None,
            );
            let dialog = widgets.dialog;
            let body_editor = widgets.body_editor;
            let choice = widgets.account_choice;
            let browser = widgets.browser.clone();
            wx_compose::follow_the_from_account(
                choice,
                body_editor,
                vec![
                    SignatureFor {
                        name: "Work signature".to_string(),
                        text: "Regards, Work".to_string(),
                    },
                    SignatureFor {
                        name: "Home signature".to_string(),
                        text: "Cheers, Home".to_string(),
                    },
                ],
                0,
                a11y.clone(),
            );
            dialog.show(true);

            let run = Rc::new(RefCell::new(Run {
                acts: the_composers_acts(),
                next: 0,
                phase: Phase::WaitingForBrowser,
                ticks: 0,
                busy: false,
                harvest,
                failure: None,
            }));
            let ticker = Rc::new(Timer::new(&dialog));
            ticker.on_tick({
                let run = run.clone();
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
                            run.failure = Some("the composer's run waited too long".to_string());
                            run.phase = Phase::Done;
                            run.next = run.acts.len();
                        }
                        run.phase
                    };
                    let next = match phase {
                        Phase::WaitingForBrowser if browser.is_ready() => Phase::Acting,
                        Phase::WaitingForPage if the_page_is_up(&body_editor) => {
                            let _ = body_editor.run_script("window.__signature_run = 1; 'marked'");
                            Phase::Settling(TICKS_TO_SETTLE)
                        }
                        Phase::Settling(left) if left > 1 => Phase::Settling(left - 1),
                        Phase::Settling(_) => Phase::Acting,
                        Phase::Acting => one_act(&run, &body_editor, &choice),
                        waiting => waiting,
                    };
                    let finished = next == Phase::Done && phase != Phase::Done;
                    {
                        let mut run = run.borrow_mut();
                        run.phase = next;
                        run.busy = false;
                    }
                    if finished {
                        ticker.stop();
                        let mut run = run.borrow_mut();
                        let taken = match run.failure.take() {
                            Some(why) => Err(why),
                            None => Ok(std::mem::take(&mut run.harvest)),
                        };
                        browser.destroy_when_ready(dialog);
                        finish(taken);
                    }
                }
            });
            ticker.start(TICK_MS, false);
            // The tick closure holds the other handle; the timer must outlive
            // on_init.
            std::mem::forget(ticker);
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

fn reading(name: &str) -> &'static str {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    let harvest = match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    };
    harvest
        .get(name)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("nothing was read for {name:?}"))
}

// ── The checks, each of which a companion hands a wrong state ─────────────

fn every_signature_is_listed(manager: &str) -> Result<(), String> {
    let names: Vec<&str> = manager
        .lines()
        .map(|row| row.split(" | ").next().unwrap_or_default())
        .collect();
    match names == ["Brief", "Home signature", "Work signature"] {
        true => Ok(()),
        false => Err(format!("the manager listed {names:?}")),
    }
}

fn the_row(manager: &str, name: &str) -> String {
    manager
        .lines()
        .find(|row| row.starts_with(&format!("{name} | ")))
        .unwrap_or_default()
        .to_string()
}

fn the_dialog_opens_on(reading: &str, wanted: &str) -> Result<(), String> {
    match reading == wanted {
        true => Ok(()),
        false => Err(format!(
            "the account dialog opened on {reading:?}, not {wanted:?}"
        )),
    }
}

fn the_block_was_left_alone(before: &str, after: &str) -> Result<(), String> {
    match before == after && after.contains("Cheers, Home team") {
        true => Ok(()),
        false => Err(format!(
            "the edited message changed:\nbefore {before}\nafter  {after}"
        )),
    }
}

// ── The store ─────────────────────────────────────────────────────────────

#[test]
fn test_an_account_with_a_signature_assigned_signs_with_it() {
    assert_eq!(reading("the store: Work signs with"), "Work signature");
}

#[test]
fn test_an_account_with_none_assigned_signs_with_the_default() {
    assert_eq!(reading("the store: Home signs with"), "Home signature");
}

// ── The Signature Manager ─────────────────────────────────────────────────

#[test]
fn test_the_manager_lists_every_signature_whichever_account_wrote_it() {
    every_signature_is_listed(reading("the manager, opened")).unwrap();
}

#[test]
fn test_the_manager_says_who_uses_each_signature() {
    let manager = reading("the manager, opened");
    assert_eq!(
        the_row(manager, "Work signature"),
        "Work signature |  | Work"
    );
    assert_eq!(
        the_row(manager, "Home signature"),
        "Home signature | ★ | everyone else"
    );
    assert_eq!(the_row(manager, "Brief"), "Brief |  | ");
}

#[test]
fn test_companion_a_manager_showing_one_accounts_signatures_is_refused() {
    let one_account = "Brief |  | \nWork signature |  | Work";
    assert!(every_signature_is_listed(one_account).is_err());
}

// ── The account's own dialog ──────────────────────────────────────────────

#[test]
fn test_the_account_dialog_opens_on_the_signature_the_account_uses() {
    the_dialog_opens_on(reading("Work's dialog, opened on"), "Work signature").unwrap();
}

#[test]
fn test_the_account_dialogs_first_entry_names_the_default() {
    assert_eq!(
        reading("Work's dialog, first entry"),
        "Use the default: Home signature"
    );
}

// ── Each surface shows what the other chose ──────────────────────────────

#[test]
fn test_an_account_ticked_in_a_signatures_editor_is_what_its_dialog_shows() {
    the_dialog_opens_on(
        reading("Home's dialog, after Brief's editor gave it Home"),
        "Brief",
    )
    .unwrap();
}

#[test]
fn test_companion_an_assignment_the_account_dialog_does_not_read_is_refused() {
    // A manager that kept its own copy of the assignment: the dialog would
    // open on the default, as though nothing had been chosen.
    assert!(the_dialog_opens_on("Use the default: Home signature", "Brief").is_err());
}

#[test]
fn test_a_signature_chosen_on_the_account_dialog_is_what_the_manager_shows() {
    assert_eq!(
        the_row(
            reading("the manager, after Work's dialog chose Brief"),
            "Brief"
        ),
        "Brief |  | Work, Home"
    );
    assert_eq!(
        reading("Brief's editor, Work's box, after Work's dialog chose Brief"),
        "true"
    );
}

// ── The default, set and cleared in the manager ──────────────────────────

#[test]
fn test_the_default_set_in_the_manager_is_named_on_the_account_dialog() {
    assert_eq!(
        reading("Work's dialog, first entry, with Brief the default"),
        "Use the default: Brief"
    );
    assert_eq!(
        reading("the store: nobody's, with Brief the default"),
        "Brief"
    );
}

#[test]
fn test_the_default_cleared_in_the_manager_leaves_an_unassigned_account_with_none() {
    assert_eq!(
        reading("Work's dialog, first entry, with no default"),
        "Use the default (none is set)"
    );
    assert_eq!(reading("the store: nobody's, with no default"), "none");
}

// ── The composer ──────────────────────────────────────────────────────────

#[test]
fn test_a_new_messages_signature_follows_the_from_account() {
    let opened = reading("a new message, opened");
    assert!(opened.contains("Regards, Work"), "opened as {opened}");
    let after = reading("a new message, after From changed to Home");
    assert!(
        after.contains("Cheers, Home") && !after.contains("Regards, Work"),
        "after the change: {after}"
    );
}

#[test]
fn test_a_signature_somebody_typed_into_stays_when_the_from_account_changes() {
    the_block_was_left_alone(
        reading("an edited message, before From changed"),
        reading("an edited message, after From changed to Work"),
    )
    .unwrap();
}

#[test]
fn test_companion_a_swap_over_an_edited_signature_is_refused() {
    let before = "<br><br>-- <br>Cheers, Home team";
    let swapped = "<br><br>-- <br>Regards, Work";
    assert!(the_block_was_left_alone(before, swapped).is_err());
}

#[test]
fn test_a_replys_signature_follows_the_from_account_above_the_quote() {
    let after = reading("a reply, after From changed to Home");
    let signature = after.find("Cheers, Home");
    let quote = after.find("--- Original Message ---");
    assert!(
        matches!((signature, quote), (Some(s), Some(q)) if s < q)
            && !after.contains("Regards, Work")
            && after.contains("Their words"),
        "the reply after the change: {after}"
    );
}
