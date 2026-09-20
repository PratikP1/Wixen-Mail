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
//! startup block binds the setting into a local a closure could capture. And
//! an audit: every setting that block reads is held to a list with a
//! disposition each, following a save through a named update or call, or
//! next-start by nature with its control saying so, or the window's own, so
//! a third setting cannot be captured quietly. Each reading is a function
//! over the text with a companion that hands it the opposite and requires a
//! complaint. What no reading can see, said plainly:
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

// ── The date settings follow a save the same way ────────────────────────────
//
// The four date settings were captured at the same place into the row
// callback, the PIM cells and every read-aloud closure, while four other
// sites read them afresh, so a date style chosen in Settings showed in the
// calendar heading and not in the rows. They travel as the wait does: a
// state field, an update the Settings-saved arm sends, the closures reading
// the state under the lock they already take, and the six lists repainted
// once when the update arrives, never per row.

const THE_SETTINGS_SCREEN: &str = "src/presentation/wx_settings.rs";

/// The update the Settings-saved arm sends for the dates, and what it
/// builds it from, matched without whitespace like the wait's.
const THE_DATES_UPDATE: &str = "UIUpdate::DateSettingsChanged(";
const THE_DATES_SETTING: &str = "date_settings_from(&new_config)";

/// The write into state the startup block makes for the dates.
const THE_DATES_WRITE_AT_STARTUP: &str = "lock_state(&state).dates = stored_config";

/// Where the mail list's paint callback starts and ends, and where a PIM
/// list's does: the closures that used to hold the captured dates.
const THE_ROW_CALLBACK: (&str, &str) = (
    "msg_list.set_virtual_text_callback({",
    "if !callback_registered",
);
const THE_CELL_CALLBACK: (&str, &str) = (
    "list.set_virtual_text_callback(move |row, column| {",
    "if !registered",
);

/// The six lists the dates are painted into, every one repainted when the
/// update arrives.
const THE_SIX_LISTS: [&str; 6] = [
    "msg_list",
    "contact_list",
    "cal_event_list",
    "reminder_list",
    "task_list",
    "note_list",
];

/// The local the dates used to be captured into. Named nowhere in the file
/// any more, so no closure can quietly hold a copy; `date_settings_from`,
/// the one mapping from the stored settings, is a different name.
fn names_the_captured_dates(app: &str) -> bool {
    let local = "date_settings";
    app.match_indices(local)
        .any(|(at, _)| !app[at + local.len()..].starts_with("_from"))
}

/// The rows and the cells read the dates from the state under the lock the
/// callback already takes, and nothing in the file holds the old capture.
fn the_rows_read_the_dates_from_state(app: &str) -> Result<(), String> {
    if names_the_captured_dates(app) {
        return Err(
            "the file still names date_settings, the local the row, cell and read-aloud closures \
             captured at startup, so a date style saved in Settings shows in the rows only after \
             a restart; that is #91's shape on the dates"
                .to_string(),
        );
    }
    let block = between(app, THE_STARTUP_BLOCK.0, THE_STARTUP_BLOCK.1)?;
    if !block.contains(THE_DATES_WRITE_AT_STARTUP) {
        return Err(
            "the startup block never writes the dates into the state, so the rows are painted \
             with the defaults until Settings is saved"
                .to_string(),
        );
    }
    let rows = between(app, THE_ROW_CALLBACK.0, THE_ROW_CALLBACK.1)?;
    if !rows.contains("state.dates") {
        return Err(
            "the mail list's paint callback never reads the dates from the state, so the rows \
             keep whatever they were built with"
                .to_string(),
        );
    }
    let cells = between(app, THE_CELL_CALLBACK.0, THE_CELL_CALLBACK.1)?;
    if !cells.contains("s.dates") {
        return Err(
            "the PIM lists' paint callback never reads the dates from the state, so their cells \
             keep whatever they were built with"
                .to_string(),
        );
    }
    Ok(())
}

/// Saving Settings sends the dates as an update, built from the saved
/// configuration through the one mapping, after the save succeeded.
fn saving_settings_sends_the_dates(app: &str) -> Result<(), String> {
    let arm = between(app, THE_SETTINGS_SAVED_ARM.0, THE_SETTINGS_SAVED_ARM.1)?;
    let Some(sent_at) = arm.find(THE_DATES_UPDATE) else {
        return Err(
            "the Settings-saved arm never sends DateSettingsChanged, so a date style saved in \
             Settings reaches the calendar heading and never the rows"
                .to_string(),
        );
    };
    if !without_whitespace(arm).contains(THE_DATES_SETTING) {
        return Err(
            "the Settings-saved arm sends DateSettingsChanged from something other than \
             date_settings_from(&new_config), the one mapping every other reader uses"
                .to_string(),
        );
    }
    let saved_at = arm
        .find(THE_SAVE)
        .ok_or("the Settings-saved arm never saves, so this reads nothing".to_string())?;
    if sent_at < saved_at {
        return Err(
            "the Settings-saved arm sends DateSettingsChanged before the save, so a save that \
             fails still changes the running window"
                .to_string(),
        );
    }
    Ok(())
}

/// The update's arm writes the state and repaints every list once, so the
/// next paint reads the new dates and no row reads configuration.
fn the_update_writes_the_dates_and_repaints(app: &str) -> Result<(), String> {
    let handler = body_of(app, "fn handle_update(")?;
    let arm = the_arm_for(&handler, THE_DATES_UPDATE)?;
    if !arm.contains(".dates = ") {
        return Err(
            "the DateSettingsChanged arm never writes the dates into the state, so the update \
             arrives and the rows go on with the old ones"
                .to_string(),
        );
    }
    if !arm.contains(".refresh(") {
        return Err(
            "the DateSettingsChanged arm repaints nothing, so the new dates show only when \
             something else makes a list paint"
                .to_string(),
        );
    }
    if let Some(list) = THE_SIX_LISTS.iter().find(|list| !arm.contains(*list)) {
        return Err(format!(
            "the DateSettingsChanged arm never repaints {list}, so that list shows the old dates \
             until something else repaints it"
        ));
    }
    if arm.contains("load_stored") {
        return Err(
            "the DateSettingsChanged arm reads the stored file, where the update it was handed \
             already carries the answer"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_the_rows_and_the_cells_read_the_dates_from_the_state() {
    the_rows_read_the_dates_from_state(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_saving_settings_sends_the_dates_after_the_save() {
    saving_settings_sends_the_dates(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_update_writes_the_dates_and_repaints_every_list_once() {
    the_update_writes_the_dates_and_repaints(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The startup block, the two callbacks, the Settings-saved arm and the
/// update's arm as they should be for the dates, in a snippet.
fn a_window_whose_dates_are_as_they_should_be() -> String {
    format!(
        "{}\n    .map(|mgr| mgr.app_config().clone());\n\
         lock_state(&state).dates = stored_config.as_ref().map(date_settings_from).unwrap_or_default();\n\
         {}\n\
         {}\n    let Ok(state) = state.lock() else {{ return placeholder; }};\n    text_for(listed, &columns, row, column, state.dates, now)\n}});\n{}\n\
         {}\n    let Ok(s) = state.lock() else {{ return placeholder; }};\n    event_cell(e, column, s.dates, now, s.working_day)\n}});\n{}\n\
         fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {{\n    match update {{\n        UIUpdate::Other => {{}}\n        UIUpdate::DateSettingsChanged(dates) => {{\n            lock_state(state).dates = *dates;\n            for list in [msg_list, &pim.contact_list, &pim.cal_event_list, &pim.reminder_list, &pim.task_list, &pim.note_list] {{\n                list.refresh(true, None);\n            }}\n        }}\n        UIUpdate::Another => {{}}\n    }}\n}}\n\
         {}\n    let dates = date_settings_from(&new_config);\n    if let Err(e) = mgr.save() {{\n    }} else {{\n        let _ = tx.try_send(UIUpdate::DateSettingsChanged(dates));\n    }}\n{}\n",
        THE_STARTUP_BLOCK.0,
        THE_STARTUP_BLOCK.1,
        THE_ROW_CALLBACK.0,
        THE_ROW_CALLBACK.1,
        THE_CELL_CALLBACK.0,
        THE_CELL_CALLBACK.1,
        THE_SETTINGS_SAVED_ARM.0,
        THE_SETTINGS_SAVED_ARM.1,
    )
}

#[test]
fn test_the_date_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_whose_dates_are_as_they_should_be();
    the_rows_read_the_dates_from_state(&app).unwrap_or_else(|why| panic!("{why}"));
    saving_settings_sends_the_dates(&app).unwrap_or_else(|why| panic!("{why}"));
    the_update_writes_the_dates_and_repaints(&app).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_date_readings_complain_when_the_capture_is_back_or_a_list_is_missed() {
    // The local back, captured by the row callback as it was until 11-11.1.1.
    let app = a_window_whose_dates_are_as_they_should_be().replacen(
        "text_for(listed, &columns, row, column, state.dates, now)",
        "text_for(listed, &columns, row, column, date_settings, now)",
        1,
    );
    let why = the_rows_read_the_dates_from_state(&app)
        .expect_err("a row callback holding the captured dates was passed over");
    assert!(why.contains("still names date_settings"), "{why}");

    // The cells reading a copy of their own rather than the state.
    let app = a_window_whose_dates_are_as_they_should_be().replacen(
        "event_cell(e, column, s.dates, now, s.working_day)",
        "event_cell(e, column, DateSettings::default(), now, s.working_day)",
        1,
    );
    let why = the_rows_read_the_dates_from_state(&app)
        .expect_err("a cell callback reading no dates from the state was passed over");
    assert!(
        why.contains("PIM lists' paint callback never reads"),
        "{why}"
    );

    // The arm sending nothing.
    let app = a_window_whose_dates_are_as_they_should_be().replacen(
        "        let _ = tx.try_send(UIUpdate::DateSettingsChanged(dates));\n",
        "",
        1,
    );
    let why =
        saving_settings_sends_the_dates(&app).expect_err("an arm sending nothing was passed over");
    assert!(why.contains("never sends DateSettingsChanged"), "{why}");

    // The update's arm forgetting one list.
    let app = a_window_whose_dates_are_as_they_should_be().replacen(", &pim.note_list]", "]", 1);
    let why = the_update_writes_the_dates_and_repaints(&app)
        .expect_err("an arm repainting five lists of six was passed over");
    assert!(why.contains("never repaints note_list"), "{why}");

    // The update's arm reading the file instead of what it was handed.
    let app = a_window_whose_dates_are_as_they_should_be().replacen(
        "lock_state(state).dates = *dates;",
        "lock_state(state).dates = ConfigManager::load_stored().map(|m| date_settings_from(m.app_config())).unwrap_or_default();",
        1,
    );
    let why = the_update_writes_the_dates_and_repaints(&app)
        .expect_err("an arm reading the stored file was passed over");
    assert!(why.contains("reads the stored file"), "{why}");
}

// ── The audit: every setting read at startup has a disposition ──────────────
//
// The startup block reads the stored settings once, for the reason its own
// comment gives: the paint callback runs for every visible cell and must not
// touch configuration. Every setting it reads is therefore one a save could
// leave behind, and this holds the block to a list with a disposition for
// each: it follows a save through a named update or a named call in the
// Settings-saved arm, or it is next-start by nature and the control that
// offers it says so, or no settings control offers it at all. A setting
// read there with no row fails with a sentence saying what to do; a row
// naming a setting the block no longer reads fails too, so the list cannot
// outlive the code.

/// What a setting the startup block reads is allowed to be.
enum Disposition {
    /// The Settings-saved arm applies it to the running window: the text
    /// the arm must contain, an update it sends or a call it makes.
    FollowsASave(&'static str),
    /// It applies at the next start by nature, and the control that offers
    /// it says so: the builder of its tab, the control's accessible name,
    /// and the sentence's name, which must follow the control in the builder.
    NextStartAndSaidSo {
        tab: &'static str,
        control: &'static str,
        sentence: &'static str,
    },
    /// The window's own: written by a function in the window that also
    /// applies it on screen, and the part of it a listing needs read on use
    /// by another, so a save has nothing to send. Both functions must name
    /// the setting.
    OwnedByTheWindow {
        written_by: &'static str,
        read_on_use_by: &'static str,
    },
}

use Disposition::{FollowsASave, NextStartAndSaidSo, OwnedByTheWindow};

/// Every setting the startup block reads, by the field it reads from the
/// stored configuration, or by the one mapping function it hands the
/// configuration to, with what each is allowed to be.
const THE_STARTUP_CAPTURES: &[(&str, Disposition)] = &[
    (
        "working_day_starts",
        FollowsASave("UIUpdate::WorkingDayChanged("),
    ),
    (
        "working_day_ends",
        FollowsASave("UIUpdate::WorkingDayChanged("),
    ),
    (
        "default_reminder_minutes",
        FollowsASave("UIUpdate::DefaultEventAlertLeadChanged("),
    ),
    (
        "calendar_view",
        FollowsASave("UIUpdate::CalendarViewChanged("),
    ),
    (
        "mark_read_after",
        FollowsASave("UIUpdate::MarkReadAfterChanged("),
    ),
    // `date_style`, `date_order`, `date_wording` and `clock_hours`, read
    // through the one mapping so a list column and an opened message
    // cannot disagree about the order of the day and the month.
    (
        "date_settings_from",
        FollowsASave("UIUpdate::DateSettingsChanged("),
    ),
    // The saved column layout: the column chooser writes it on F8 and
    // applies the columns in the same window, and the sort it carries, the
    // second level of which the Reading tab's Then by control writes, is
    // read from the stored copy at every listing.
    (
        "message_columns",
        OwnedByTheWindow {
            written_by: "fn persist_column_layout(",
            read_on_use_by: "fn the_sort_as(",
        },
    ),
    (
        "default_sort_order",
        NextStartAndSaidSo {
            tab: "fn build_reading_tab(",
            control: "\"Default sort order\"",
            sentence: "WHERE_THE_DEFAULT_SORT_ORDER_APPLIES",
        },
    ),
    (
        "feedback_channels",
        FollowsASave("a11y.set_feedback_settings("),
    ),
    (
        "announce_while_fetching",
        FollowsASave("a11y.set_how_much_to_say("),
    ),
    ("sound_scheme_id", FollowsASave("a11y.set_sound_scheme(")),
];

fn is_identifier(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The identifier starting at `at`, which may be empty.
fn identifier_at(text: &str, at: usize) -> &str {
    let len = text.as_bytes()[at..]
        .iter()
        .take_while(|byte| is_identifier(**byte))
        .count();
    &text[at..at + len]
}

/// The names a block binds the configuration to: `cfg` in `|cfg|` and in
/// `if let Some(cfg) = stored_config...`.
fn bound_names(block: &str) -> Vec<String> {
    let mut names = Vec::new();
    for (opener, closer) in [("|", '|'), ("Some(", ')')] {
        for (at, _) in block.match_indices(opener) {
            let start = at + opener.len();
            let name = identifier_at(block, start);
            if !name.is_empty() && block[start + name.len()..].starts_with(closer) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Every setting the block reads: a field read through a closure's
/// parameter (`cfg.mark_read_after`, but not a method such as
/// `cfg.as_ref()`), or a function the configuration is handed whole
/// (`.map(date_settings_from)`). Comment lines are left out.
fn what_the_startup_block_reads(block: &str) -> Vec<String> {
    let code: String = block
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| format!("{line}\n"))
        .collect();
    let mut read = Vec::new();
    for parameter in bound_names(&code) {
        let dotted = format!("{parameter}.");
        for (at, _) in code.match_indices(&dotted) {
            let preceded_by_identifier = at > 0 && is_identifier(code.as_bytes()[at - 1]);
            if preceded_by_identifier {
                continue;
            }
            let field = identifier_at(&code, at + dotted.len());
            let is_a_method = code[at + dotted.len() + field.len()..].starts_with('(');
            if !field.is_empty() && !is_a_method {
                read.push(field.to_string());
            }
        }
    }
    for (at, _) in code.match_indices(".map(") {
        let function = identifier_at(&code, at + ".map(".len());
        if !function.is_empty() && code[at + ".map(".len() + function.len()..].starts_with(')') {
            read.push(function.to_string());
        }
    }
    read.sort();
    read.dedup();
    read
}

/// The audit: what the block reads is exactly what the list names, and
/// every disposition is true of the tree.
fn every_startup_capture_has_a_disposition(app: &str, settings_screen: &str) -> Result<(), String> {
    let block = between(app, THE_STARTUP_BLOCK.0, THE_STARTUP_BLOCK.1)?;
    let read = what_the_startup_block_reads(block);
    if read.is_empty() {
        return Err("the startup block reads no setting at all, so this reads nothing".to_string());
    }
    for name in &read {
        if !THE_STARTUP_CAPTURES
            .iter()
            .any(|(listed, _)| listed == name)
        {
            return Err(format!(
                "a setting captured at startup with no disposition: {name}; either send an update \
                 on save or say on its control that it takes effect at the next start"
            ));
        }
    }
    for (listed, _) in THE_STARTUP_CAPTURES {
        if !read.iter().any(|name| name == listed) {
            return Err(format!(
                "the audit names {listed}, which the startup block no longer reads; drop the row"
            ));
        }
    }
    let arm = between(app, THE_SETTINGS_SAVED_ARM.0, THE_SETTINGS_SAVED_ARM.1)?;
    for (name, disposition) in THE_STARTUP_CAPTURES {
        match disposition {
            FollowsASave(applied_by) => {
                if !arm.contains(applied_by) {
                    return Err(format!(
                        "{name} is listed as following a save through {applied_by}, and the \
                         Settings-saved arm does not do that; that is the capture #91 met"
                    ));
                }
            }
            NextStartAndSaidSo {
                tab,
                control,
                sentence,
            } => {
                let builder = body_of(settings_screen, tab)?;
                let Some(control_at) = builder.find(control) else {
                    return Err(format!(
                        "{name} is listed as said on its control, and {tab} has no control named \
                         {control}"
                    ));
                };
                if !builder[control_at..].contains(sentence) {
                    return Err(format!(
                        "{name} is listed as taking effect at the next start, and nothing under \
                         {control} says so with {sentence}"
                    ));
                }
            }
            OwnedByTheWindow {
                written_by,
                read_on_use_by,
            } => {
                for (function, part) in [
                    (written_by, "writes it"),
                    (read_on_use_by, "reads it on use"),
                ] {
                    if !body_of(app, function)?.contains(name) {
                        return Err(format!(
                            "{name} is listed as the window's own, and {function}, which {part}, \
                             no longer names it; give it a disposition that is true"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// The log level is not read by the window at all: it is set up once in
/// main, with no way to change it while the program runs, so it is
/// next-start by nature and its control says so.
fn the_log_level_says_it_waits_for_the_next_start(settings_screen: &str) -> Result<(), String> {
    let builder = body_of(settings_screen, "fn build_advanced_tab(")?;
    let Some(control_at) = builder.find("\"Log level\"") else {
        return Err(
            "the Advanced tab has no control named Log level, so this reads nothing".to_string(),
        );
    };
    if !builder[control_at..].contains("TAKES_EFFECT_AT_THE_NEXT_START") {
        return Err(
            "nothing under Log level says the level takes effect at the next start, so a person \
             changing it is left to find out"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_every_setting_read_at_startup_has_a_disposition_the_tree_honours() {
    every_startup_capture_has_a_disposition(
        &shipped(THE_MAIN_WINDOW),
        &shipped(THE_SETTINGS_SCREEN),
    )
    .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_log_level_says_it_takes_effect_at_the_next_start() {
    the_log_level_says_it_waits_for_the_next_start(&shipped(THE_SETTINGS_SCREEN))
        .unwrap_or_else(|why| panic!("{why}"));
}

/// A startup block reading every listed setting the way the real one does,
/// a Settings-saved arm applying each, and a settings screen saying what it
/// must, in a snippet.
fn a_window_whose_captures_are_all_listed() -> String {
    format!(
        "{}\n    .map(|mgr| mgr.app_config().clone());\n\
         lock_state(&state).dates = stored_config.as_ref().map(date_settings_from).unwrap_or_default();\n\
         lock_state(&state).working_day = stored_config.as_ref().map(|cfg| WorkingDay::from_setting(cfg.working_day_starts, cfg.working_day_ends)).unwrap_or_default();\n\
         if let Some(cfg) = stored_config.as_ref() {{ lock_state(&state).default_event_alert_lead = i64::from(cfg.default_reminder_minutes); }}\n\
         let opens_on = stored_config.as_ref().map_or(CalendarView::default(), |cfg| CalendarView::from_stored(&cfg.calendar_view));\n\
         lock_state(&state).marks_read = stored_config.as_ref().map(|cfg| MarkRead::from_setting(&cfg.mark_read_after)).unwrap_or_default();\n\
         let mut starting_layout = stored_config.as_ref().map(|c| c.message_columns.as_str()).filter(|stored| !stored.is_empty()).and_then(ColumnLayout::from_stored).unwrap_or_else(defaults);\n\
         if let Some(asked_for) = stored_config.as_ref().and_then(|c| Sort::from_setting(&c.default_sort_order)) {{ starting_layout.sort = asked_for; }}\n\
         if let Some(stored) = stored_config.as_ref().map(|c| c.feedback_channels.as_str()).filter(|s| !s.is_empty()) {{ a11y.set_feedback_settings(FeedbackSettings::from_stored(stored)); }}\n\
         a11y.set_how_much_to_say(HowMuchToSay::from_stored(stored_config.as_ref().map_or(\"\", |c| c.announce_while_fetching.as_str())));\n\
         if let Some(id) = stored_config.as_ref().map(|c| c.sound_scheme_id.as_str()) {{ a11y.set_sound_scheme(SoundScheme::resolve(id)); }}\n\
         {}\n\
         {}\n    a11y.set_feedback_settings(x);\n    a11y.set_how_much_to_say(y);\n    a11y.set_sound_scheme(z);\n    if let Err(e) = mgr.save() {{\n    }} else {{\n        let _ = tx.try_send(UIUpdate::WorkingDayChanged(working_day));\n        let _ = tx.try_send(UIUpdate::DefaultEventAlertLeadChanged(lead));\n        let _ = tx.try_send(UIUpdate::CalendarViewChanged(opens_on));\n        let _ = tx.try_send(UIUpdate::MarkReadAfterChanged(wait));\n        let _ = tx.try_send(UIUpdate::DateSettingsChanged(dates));\n    }}\n{}\n\
         fn the_sort_as(showing: Showing) -> Option<String> {{\n    load_stored().map(|mgr| mgr.app_config().message_columns.clone())\n}}\n\
         fn persist_column_layout(layout: &ColumnLayout) {{\n    mgr.app_config_mut().message_columns = layout.to_stored();\n}}\n",
        THE_STARTUP_BLOCK.0,
        THE_STARTUP_BLOCK.1,
        THE_SETTINGS_SAVED_ARM.0,
        THE_SETTINGS_SAVED_ARM.1,
    )
}

/// A settings screen whose Reading tab says where the default sort order
/// applies and whose Advanced tab says the log level waits, in a snippet.
fn a_settings_screen_that_says_what_it_must() -> String {
    "fn build_reading_tab(panel: &Panel, config: &AppConfig) -> ReadingTabControls {\n    \
     set_accessible_name(&sort_choice, \"Default sort order\");\n    \
     let sort_note = a_sentence_under(panel, WHERE_THE_DEFAULT_SORT_ORDER_APPLIES);\n\
     }\n\
     fn build_advanced_tab(panel: &Panel, config: &AppConfig) -> AdvancedTabControls {\n    \
     set_accessible_name(&log_choice, \"Log level\");\n    \
     let log_note = a_sentence_under(panel, TAKES_EFFECT_AT_THE_NEXT_START);\n\
     }\n"
    .to_string()
}

#[test]
fn test_the_audit_passes_a_window_whose_captures_are_all_listed() {
    let app = a_window_whose_captures_are_all_listed();
    let block = between(&app, THE_STARTUP_BLOCK.0, THE_STARTUP_BLOCK.1)
        .unwrap_or_else(|why| panic!("{why}"));
    let read = what_the_startup_block_reads(block);
    let mut listed: Vec<&str> = THE_STARTUP_CAPTURES.iter().map(|(name, _)| *name).collect();
    listed.sort();
    assert_eq!(
        read, listed,
        "the snippet reads what the list names, and the reader finds it all"
    );
    every_startup_capture_has_a_disposition(&app, &a_settings_screen_that_says_what_it_must())
        .unwrap_or_else(|why| panic!("{why}"));
    the_log_level_says_it_waits_for_the_next_start(&a_settings_screen_that_says_what_it_must())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_audit_complains_about_a_capture_with_no_disposition() {
    // A new setting read where the window is built, the way #91's was.
    let app = a_window_whose_captures_are_all_listed().replacen(
        "lock_state(&state).marks_read =",
        "let snippet_length = stored_config.as_ref().map(|cfg| cfg.snippet_length).unwrap_or(80);\n\
         lock_state(&state).marks_read =",
        1,
    );
    let why =
        every_startup_capture_has_a_disposition(&app, &a_settings_screen_that_says_what_it_must())
            .expect_err("a setting captured at startup with no row was passed over");
    assert!(
        why.contains("a setting captured at startup with no disposition: snippet_length"),
        "{why}"
    );
    assert!(why.contains("either send an update on save"), "{why}");

    // The same through a method-free read in a `map_or`, so the reader is
    // not fooled by the shape of the closure.
    let app = a_window_whose_captures_are_all_listed().replacen(
        "lock_state(&state).marks_read =",
        "let theme = stored_config.as_ref().map_or(\"\", |c| c.theme.as_str());\n\
         lock_state(&state).marks_read =",
        1,
    );
    let why =
        every_startup_capture_has_a_disposition(&app, &a_settings_screen_that_says_what_it_must())
            .expect_err("a setting read through map_or with no row was passed over");
    assert!(why.contains("no disposition: theme"), "{why}");
}

#[test]
fn test_the_audit_complains_when_a_disposition_is_not_true_of_the_tree() {
    // A row for a setting the block no longer reads.
    let app = a_window_whose_captures_are_all_listed().replacen(
        "if let Some(id) = stored_config.as_ref().map(|c| c.sound_scheme_id.as_str()) { a11y.set_sound_scheme(SoundScheme::resolve(id)); }\n",
        "",
        1,
    );
    let why =
        every_startup_capture_has_a_disposition(&app, &a_settings_screen_that_says_what_it_must())
            .expect_err("a row naming a setting the block no longer reads was passed over");
    assert!(why.contains("the audit names sound_scheme_id"), "{why}");

    // A setting listed as following a save that the arm never sends: #91.
    let app = a_window_whose_captures_are_all_listed().replacen(
        "        let _ = tx.try_send(UIUpdate::MarkReadAfterChanged(wait));\n",
        "",
        1,
    );
    let why =
        every_startup_capture_has_a_disposition(&app, &a_settings_screen_that_says_what_it_must())
            .expect_err("a save that does not send a listed update was passed over");
    assert!(
        why.contains("mark_read_after is listed as following a save"),
        "{why}"
    );

    // A next-start setting whose control says nothing.
    let screen = a_settings_screen_that_says_what_it_must().replacen(
        "    let sort_note = a_sentence_under(panel, WHERE_THE_DEFAULT_SORT_ORDER_APPLIES);\n",
        "",
        1,
    );
    let why =
        every_startup_capture_has_a_disposition(&a_window_whose_captures_are_all_listed(), &screen)
            .expect_err("a next-start setting with no sentence under its control was passed over");
    assert!(
        why.contains("nothing under \"Default sort order\" says so"),
        "{why}"
    );

    // A setting listed as the window's own that the window no longer reads
    // on use: the listing would keep the sort it started with.
    let app = a_window_whose_captures_are_all_listed().replacen(
        "    load_stored().map(|mgr| mgr.app_config().message_columns.clone())\n",
        "    None\n",
        1,
    );
    let why =
        every_startup_capture_has_a_disposition(&app, &a_settings_screen_that_says_what_it_must())
            .expect_err("a setting listed as read on use that nothing reads was passed over");
    assert!(
        why.contains("fn the_sort_as(, which reads it on use, no longer names it"),
        "{why}"
    );

    // The log level's sentence gone.
    let screen = a_settings_screen_that_says_what_it_must().replacen(
        "    let log_note = a_sentence_under(panel, TAKES_EFFECT_AT_THE_NEXT_START);\n",
        "",
        1,
    );
    let why = the_log_level_says_it_waits_for_the_next_start(&screen)
        .expect_err("a Log level control with no sentence was passed over");
    assert!(why.contains("nothing under Log level says"), "{why}");
}

#[test]
fn test_the_reader_finds_a_field_and_not_a_method_and_not_a_longer_name() {
    // `cfg.as_str()` is a method, `stored_config.beta` is a longer name that
    // ends in a bound one, a comment is not code, a name bound by `Some(..)`
    // counts as a closure's does, and `.map(a_function)` is a function
    // handed the whole configuration.
    let block = "let stored_config = load();\n\
                 // a comment naming cfg.not_a_setting\n\
                 let a = stored_config.as_ref().map(|cfg| cfg.alpha.as_str());\n\
                 let b = stored_config.as_ref().map(read_them);\n\
                 let c = stored_config.beta;\n\
                 if let Some(cfg) = stored_config.as_ref() { use_it(cfg.gamma); }\n\
                 let d = stored_config.as_ref().filter(|g| !g.is_empty());\n";
    assert_eq!(
        what_the_startup_block_reads(block),
        vec![
            "alpha".to_string(),
            "gamma".to_string(),
            "read_them".to_string()
        ]
    );
}
