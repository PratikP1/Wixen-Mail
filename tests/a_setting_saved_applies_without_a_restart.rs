//! A setting saved in Settings applies without a restart, and every setting
//! that cannot says so where it is set.
//!
//! #91, 11-11.1.1. The tester on 2026-09-20 on `1.0.0-alpha.1` at `4a09bfc2`,
//! confirmed after a restart: "Mark as read after" did nothing however he set
//! it. The main timer asked the rule with a value read once, where the window
//! was built (`let marks_read = stored_config...`), and captured by the
//! timer's closure; saving Settings wrote the file and nothing the timer held,
//! so the wait he chose governed the next start and never the next tick. The
//! working day already travelled the other way: a field in the window's
//! state, written where the window is built and again by the Settings-saved
//! arm through an update, and read from the state by whatever acts on it.
//! Mark as read after joins it, and so do the date settings, which the row,
//! cell and read-aloud closures captured the same way while four other sites
//! read them afresh, so a date style chosen in Settings showed in the
//! calendar heading and not in the rows.
//!
//! Read from the source rather than run, because the sites are closures
//! inside a window with a running event loop and a timer, and what a reading
//! can hold is the shape: that the timer's function takes no setting and reads
//! the state, that the Settings-saved arm sends the update after the save
//! succeeded, that the update's arm writes the state, and that nothing in the
//! startup block binds the setting into a local a closure could capture. Each
//! reading is a function over the text with a companion that hands it the
//! opposite and requires a complaint. What no reading can see, said plainly:
//! that the tester changes the wait, presses Enter on an unread message, and
//! hears the count move after the new wait without a restart; that is his ear
//! and is on the ledger.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

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

/// Where the startup block that reads the stored settings starts, and the
/// first thing after it: the paint callback's registration, which is what
/// the block's own comment says the settings are read once for.
const THE_STARTUP_BLOCK: (&str, &str) = ("let stored_config = ", "let callback_registered");

/// Where the Settings-saved arm starts and where the arm after it starts.
const THE_SETTINGS_SAVED_ARM: (&str, &str) = (
    "wx_settings::SettingsResult::Updated(new_config) => {",
    "wx_settings::SettingsResult::Cancelled",
);

/// The update the Settings-saved arm sends for the wait, and what the arm
/// builds it from. The second is matched with the whitespace taken out and
/// ends before the closing bracket, because where rustfmt breaks a call and
/// whether it leaves a trailing comma are the formatter's choices and not
/// the code's.
const THE_MARK_READ_UPDATE: &str = "UIUpdate::MarkReadAfterChanged(";
const THE_MARK_READ_SETTING: &str = "MarkRead::from_setting(&new_config.mark_read_after";

/// The text with every run of whitespace removed, so a call reads the same
/// however the formatter broke it.
fn without_whitespace(text: &str) -> String {
    text.split_whitespace().collect()
}

/// The write into state the startup block makes, so the field holds the
/// stored answer from the first tick.
const THE_MARK_READ_WRITE_AT_STARTUP: &str = "lock_state(&state).marks_read = stored_config";

/// The save that has to succeed before any update goes out (T-11-109).
const THE_SAVE: &str = "mgr.save()";

/// The arm of the update handler for one update: from its pattern to the
/// next arm's, inside the handler's body.
fn the_arm_for<'a>(handler: &'a str, update: &str) -> Result<&'a str, String> {
    let at = handler.find(update).ok_or(format!(
        "{update} has no arm in handle_update, so a save sends an update nothing answers"
    ))?;
    let rest = &handler[at + update.len()..];
    let end = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
    Ok(&rest[..end])
}

/// The timer's function takes no setting: it reads the wait from the state
/// under the lock it already takes, so the state is the one place the
/// setting lives while the program runs.
fn the_timer_reads_the_state(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn mark_what_was_read(")?;
    let head = body.split('{').next().unwrap_or_default();
    if head.contains("MarkRead") || head.contains("marks_read") {
        return Err(
            "mark_what_was_read takes the wait as a parameter, so whoever calls it hands it a \
             value read once, and a save in Settings cannot reach it; that is #91"
                .to_string(),
        );
    }
    if !body.contains("s.marks_read") {
        return Err(
            "mark_what_was_read never reads marks_read from the state, so the wait the timer \
             asks the rule with is not the one a save writes"
                .to_string(),
        );
    }
    Ok(())
}

/// Saving Settings sends the wait to the window as an update, built from the
/// saved configuration, after the save succeeded and not before.
fn saving_settings_sends_the_wait(app: &str) -> Result<(), String> {
    let arm = between(app, THE_SETTINGS_SAVED_ARM.0, THE_SETTINGS_SAVED_ARM.1)?;
    let Some(sent_at) = arm.find(THE_MARK_READ_UPDATE) else {
        return Err(
            "the Settings-saved arm never sends MarkReadAfterChanged, so saving a new wait \
             writes the file and nothing the timer reads; that is #91"
                .to_string(),
        );
    };
    if !without_whitespace(arm).contains(THE_MARK_READ_SETTING) {
        return Err(
            "the Settings-saved arm sends MarkReadAfterChanged from something other than the \
             saved mark_read_after, read through MarkRead::from_setting"
                .to_string(),
        );
    }
    let saved_at = arm
        .find(THE_SAVE)
        .ok_or("the Settings-saved arm never saves, so this reads nothing".to_string())?;
    if sent_at < saved_at {
        return Err(
            "the Settings-saved arm sends MarkReadAfterChanged before the save, so a save that \
             fails still changes the running window"
                .to_string(),
        );
    }
    Ok(())
}

/// The update's arm writes the state field the timer reads.
fn the_update_writes_the_state(app: &str) -> Result<(), String> {
    let handler = body_of(app, "fn handle_update(")?;
    let arm = the_arm_for(&handler, THE_MARK_READ_UPDATE)?;
    if !arm.contains(".marks_read = ") {
        return Err(
            "the MarkReadAfterChanged arm never writes marks_read into the state, so the update \
             arrives and the timer goes on asking with the old wait"
                .to_string(),
        );
    }
    Ok(())
}

/// The startup block writes the stored wait into the state and binds it into
/// no local: a local is what a closure captures, and a captured value is one
/// a save cannot reach.
fn nothing_captures_the_wait_at_startup(app: &str) -> Result<(), String> {
    let block = between(app, THE_STARTUP_BLOCK.0, THE_STARTUP_BLOCK.1)?;
    if let Some(line) = block
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("let ") && line.contains("marks_read"))
    {
        return Err(format!(
            "the startup block binds the wait into a local ({line}), which a closure captures \
             and a save cannot reach; that is the capture #91 met"
        ));
    }
    if !block.contains(THE_MARK_READ_WRITE_AT_STARTUP) {
        return Err(
            "the startup block never writes marks_read into the state, so the first tick asks \
             the rule with the default and not the stored wait"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_the_timer_reads_the_wait_from_the_state_and_takes_none() {
    the_timer_reads_the_state(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_saving_settings_sends_the_wait_after_the_save() {
    saving_settings_sends_the_wait(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_update_writes_the_wait_into_the_state() {
    the_update_writes_the_state(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_nothing_captures_the_wait_where_the_window_is_built() {
    nothing_captures_the_wait_at_startup(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── Companions: each reading complains when handed the opposite ─────────────
//
// Each hands its reading a snippet shaped like the site, with the fault
// planted, rather than editing the real file's text: the readings' anchors
// exist only once the sites are built, and a companion that edits the real
// text is red for the wrong reason until then.

/// The timer's function, the Settings-saved arm, the update's arm and the
/// startup block as they should be, in a snippet.
fn a_window_as_it_should_be() -> String {
    format!(
        "{}\n    .map(|mgr| mgr.app_config().clone());\n\
         lock_state(&state).marks_read = stored_config.as_ref().map(read_it).unwrap_or_default();\n\
         {}\n\
         fn mark_what_was_read(app: AppHandles<'_>) {{\n    let mut s = lock_state(state);\n    let mark = whether_to_mark_read(began, selected_unread, now, s.marks_read);\n}}\n\
         fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {{\n    match update {{\n        UIUpdate::Other => {{}}\n        UIUpdate::MarkReadAfterChanged(wait) => {{\n            lock_state(state).marks_read = *wait;\n        }}\n        UIUpdate::Another => {{}}\n    }}\n}}\n\
         {}\n    let wait = MarkRead::from_setting(&new_config.mark_read_after);\n    if let Err(e) = mgr.save() {{\n    }} else {{\n        let _ = tx.try_send(UIUpdate::MarkReadAfterChanged(wait));\n    }}\n{}\n",
        THE_STARTUP_BLOCK.0,
        THE_STARTUP_BLOCK.1,
        THE_SETTINGS_SAVED_ARM.0,
        THE_SETTINGS_SAVED_ARM.1,
    )
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_as_it_should_be();
    the_timer_reads_the_state(&app).unwrap_or_else(|why| panic!("{why}"));
    saving_settings_sends_the_wait(&app).unwrap_or_else(|why| panic!("{why}"));
    the_update_writes_the_state(&app).unwrap_or_else(|why| panic!("{why}"));
    nothing_captures_the_wait_at_startup(&app).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_the_timer_takes_the_wait_or_reads_it_from_nowhere() {
    // The parameter back, as it was until 11-11.1.1.
    let app = a_window_as_it_should_be().replacen(
        "fn mark_what_was_read(app: AppHandles<'_>) {",
        "fn mark_what_was_read(app: AppHandles<'_>, marks_read: MarkRead) {",
        1,
    );
    let why = the_timer_reads_the_state(&app).expect_err("a timer taking the wait was passed over");
    assert!(why.contains("takes the wait as a parameter"), "{why}");

    // No parameter, and no read of the state either: the default every tick.
    let app = a_window_as_it_should_be().replacen(
        "whether_to_mark_read(began, selected_unread, now, s.marks_read)",
        "whether_to_mark_read(began, selected_unread, now, MarkRead::default())",
        1,
    );
    let why = the_timer_reads_the_state(&app)
        .expect_err("a timer reading the wait from nowhere was passed over");
    assert!(
        why.contains("never reads marks_read from the state"),
        "{why}"
    );
}

#[test]
fn test_the_reading_complains_when_the_save_sends_nothing_or_sends_it_before_saving() {
    let app = a_window_as_it_should_be().replacen(
        "        let _ = tx.try_send(UIUpdate::MarkReadAfterChanged(wait));\n",
        "",
        1,
    );
    let why =
        saving_settings_sends_the_wait(&app).expect_err("an arm sending nothing was passed over");
    assert!(why.contains("never sends MarkReadAfterChanged"), "{why}");

    let app = a_window_as_it_should_be().replacen(
        "    if let Err(e) = mgr.save() {\n    } else {\n        let _ = tx.try_send(UIUpdate::MarkReadAfterChanged(wait));\n    }\n",
        "    let _ = tx.try_send(UIUpdate::MarkReadAfterChanged(wait));\n    if let Err(e) = mgr.save() {\n    }\n",
        1,
    );
    let why = saving_settings_sends_the_wait(&app)
        .expect_err("an arm sending before the save was passed over");
    assert!(why.contains("before the save"), "{why}");

    let app = a_window_as_it_should_be().replacen(
        "MarkRead::from_setting(&new_config.mark_read_after)",
        "MarkRead::default()",
        1,
    );
    let why = saving_settings_sends_the_wait(&app)
        .expect_err("an arm sending something other than the saved wait was passed over");
    assert!(why.contains("something other than the saved"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_update_has_no_arm_or_the_arm_writes_nothing() {
    let app = a_window_as_it_should_be().replacen(
        "            lock_state(state).marks_read = *wait;\n",
        "            let _ = wait;\n",
        1,
    );
    let why =
        the_update_writes_the_state(&app).expect_err("an arm writing nothing was passed over");
    assert!(
        why.contains("never writes marks_read into the state"),
        "{why}"
    );

    let app = a_window_as_it_should_be().replacen(
        "        UIUpdate::MarkReadAfterChanged(wait) => {\n            lock_state(state).marks_read = *wait;\n        }\n",
        "",
        1,
    );
    let why = the_update_writes_the_state(&app).expect_err("a missing arm was passed over");
    assert!(why.contains("has no arm in handle_update"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_startup_block_captures_the_wait() {
    // The binding as it was until 11-11.1.1: a local the timer's closure took.
    let app = a_window_as_it_should_be().replacen(
        "lock_state(&state).marks_read = stored_config.as_ref().map(read_it).unwrap_or_default();",
        "lock_state(&state).marks_read = stored_config.as_ref().map(read_it).unwrap_or_default();\n\
         let marks_read = stored_config.as_ref().map(read_it).unwrap_or_default();",
        1,
    );
    let why = nothing_captures_the_wait_at_startup(&app)
        .expect_err("a startup block binding the wait into a local was passed over");
    assert!(why.contains("binds the wait into a local"), "{why}");

    // Nothing written into the state at all: the first tick asks with the default.
    let app = a_window_as_it_should_be().replacen(
        "lock_state(&state).marks_read = stored_config.as_ref().map(read_it).unwrap_or_default();",
        "",
        1,
    );
    let why = nothing_captures_the_wait_at_startup(&app)
        .expect_err("a startup block writing nothing into the state was passed over");
    assert!(
        why.contains("never writes marks_read into the state"),
        "{why}"
    );
}
