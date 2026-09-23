//! The log carries what a tester's report needs, at the level the default
//! names, and no log call in the tree spells a secret or a body.
//!
//! #71, 11-04. Pratik's decision of 2026-09-17: the log's default is debug
//! while the version carries alpha or beta, so a report comes with a log
//! that says what happened. A level alone buys nothing if the lines are not
//! written, and until 2026-09-18 four of the five things a report needs
//! were written at no level: each check's result per folder, each chunk of
//! the download and what the server answered, that the settings were saved,
//! and that an announcement was held back. The watch's end has been written
//! since 10-06.
//!
//! Read from the source rather than run, because reaching these lines needs
//! a mail server, a window and a running event loop, and the question is
//! whether a line is written and at which level, which is a property of the
//! text. Each reading is a function over that text with a companion that
//! plants the opposite and requires a complaint naming it.
//!
//! The last reading is a guard over every `tracing::` call under `src/`,
//! with test modules cut, and it is lexical: it reads the arguments of each
//! call for five identifiers, `body_plain`, `body_html`, `password`,
//! `access_token` and `refresh_token`, as interpolated values or bare
//! arguments, and not as words in the message's prose. What it proves is
//! that no call spells one of the five. What it cannot see, said plainly: a
//! body bound to another name, a subject, an error whose text carries a
//! body, or a value reached through a method it does not name all pass it.
//! The rule itself is `CLAUDE.md`'s, and it is kept by reading each new
//! line, which the readings above do one by one.

use std::fs;
use std::path::{Path, PathBuf};
use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_QUEUE: &str = "src/presentation/accessibility/announcements.rs";

/// The five identifiers no log call may name as a value.
const THE_SECRETS: [&str; 5] = [
    "body_plain",
    "body_html",
    "password",
    "access_token",
    "refresh_token",
];

/// Lines the guard found that are not yet fixed, each with its reason: the
/// file, the identifier, and why it is still there. Empty since the guard
/// landed, and held empty by the test below; a line added here is a finding
/// somebody chose to carry rather than a silence.
const THE_LINES_STILL_TO_FIX: [(&str, &str, &str); 0] = [];

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
    let end = rest.find(to).ok_or(format!(
        "{to:?} no longer follows {from:?}, so this reads nothing"
    ))?;
    Ok(&rest[..end])
}

/// The first `tracing::<level>!(` call in `text`: where it starts, the
/// level it was written at and its arguments; or nothing when there is none.
fn the_first_log_call(text: &str) -> Option<LogCall<'_>> {
    let mut earliest: Option<(usize, &str)> = None;
    for level in ["error", "warn", "info", "debug", "trace"] {
        let opens = format!("tracing::{level}!(");
        if let Some(at) = text.find(&opens)
            && earliest.is_none_or(|(so_far, _)| at < so_far)
        {
            earliest = Some((at, level));
        }
    }
    let (at, level) = earliest?;
    let opens = "tracing::".len() + level.len() + "!(".len();
    Some(LogCall {
        at,
        level,
        arguments: the_arguments_in(&text[at + opens..]),
    })
}

struct LogCall<'a> {
    at: usize,
    level: &'a str,
    arguments: &'a str,
}

impl LogCall<'_> {
    fn ends_at(&self) -> usize {
        self.at + "tracing::".len() + self.level.len() + "!(".len() + self.arguments.len()
    }

    fn spelled(&self) -> String {
        format!("tracing::{}!({})", self.level, self.arguments)
    }

    fn at_or_above_info(&self) -> bool {
        matches!(self.level, "info" | "warn" | "error")
    }

    fn names(&self, each: &[&str], which: &str) -> Result<(), String> {
        for named in each {
            if !self.arguments.contains(named) {
                return Err(format!("{which} does not name {named}"));
            }
        }
        Ok(())
    }
}

/// The text of a call's arguments, from just after its opening bracket to
/// the bracket that closes it, read outside string and character literals
/// so a bracket inside a message cannot end the call early.
fn the_arguments_in(after: &str) -> &str {
    let bytes = after.as_bytes();
    let mut depth = 1usize;
    let mut at = 0usize;
    while at < bytes.len() {
        match bytes[at] {
            b'"' => at = past_the_string(bytes, at),
            b'\'' => at = past_the_character(bytes, at),
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return &after[..at];
                }
            }
            _ => {}
        }
        at += 1;
    }
    after
}

/// The index of the quote that closes the string opening at `at`.
fn past_the_string(bytes: &[u8], at: usize) -> usize {
    let mut at = at + 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 1,
            b'"' => return at,
            _ => {}
        }
        at += 1;
    }
    bytes.len()
}

/// The index of the quote that closes a character literal opening at `at`,
/// or `at` itself when what follows is not one (a lifetime, say).
fn past_the_character(bytes: &[u8], at: usize) -> usize {
    match bytes.get(at + 1) {
        Some(b'\\') => bytes[at + 2..]
            .iter()
            .position(|b| *b == b'\'')
            .map_or(at, |close| at + 2 + close),
        Some(_) if bytes.get(at + 2) == Some(&b'\'') => at + 2,
        _ => at,
    }
}

/// The arguments with the prose of every string literal taken out, so that
/// only what is interpolated inside braces and what stands outside the
/// quotes remains: the values a call writes, and not the words around them.
fn the_values_in(arguments: &str) -> String {
    let mut values = String::new();
    let mut chars = arguments.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '"' {
            values.push(c);
            continue;
        }
        let mut in_braces = false;
        while let Some(inner) = chars.next() {
            match inner {
                '\\' => {
                    chars.next();
                }
                '"' => break,
                '{' if chars.peek() == Some(&'{') => {
                    chars.next();
                }
                '{' => {
                    in_braces = true;
                    values.push(' ');
                }
                '}' => in_braces = false,
                _ if in_braces => values.push(inner),
                _ => {}
            }
        }
    }
    values
}

/// The identifiers in a run of code, as Rust spells them.
fn identifiers_in(code: &str) -> Vec<&str> {
    code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty() && !word.starts_with(|c: char| c.is_ascii_digit()))
        .collect()
}

/// A log call naming one of the five, and where.
#[derive(Debug, PartialEq, Eq)]
struct Finding {
    file: String,
    line: usize,
    names: String,
}

/// Every log call in `source` that names one of the five secrets as a
/// value, with the line it starts on.
fn the_secrets_in(file: &str, source: &str) -> Vec<Finding> {
    let mut found = Vec::new();
    let mut from = 0usize;
    while let Some(call) = the_first_log_call(&source[from..]) {
        let line = source[..from + call.at].matches('\n').count() + 1;
        let values = the_values_in(call.arguments);
        let named = identifiers_in(&values);
        for secret in THE_SECRETS {
            if named.contains(&secret) {
                found.push(Finding {
                    file: file.to_string(),
                    line,
                    names: secret.to_string(),
                });
            }
        }
        from += call.ends_at();
    }
    found
}

fn every_source_file(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_source_file(&path, into);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            into.push(path);
        }
    }
}

// ── The readings, each a function over the text ─────────────────────────────

/// Each folder of a check is written at info, naming the account, the
/// folder and the sentence the check said about it.
fn each_folder_of_a_check_is_written_at_info(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn spawn_mail_sync(")?;
    let arm = between(&body, "Ok(result) => {", "Err(e) => problems.push(")?;
    // The log line carries the check's own sentence, asked for by name, so
    // the log and the step say the same words.
    let the_sentence = "what_the_folder_sync_did(&result)";
    let call = the_first_log_call(arm).ok_or(
        "the check's folder arm writes nothing to the log, so a report cannot say what each \
         folder's check found",
    )?;
    if call.level != "info" {
        return Err(format!(
            "the check's folder line is written at {}, where the default under a release does \
             not reach",
            call.level
        ));
    }
    call.names(
        &["account.name", "folder.name", the_sentence],
        "the check's folder line",
    )
}

/// Each chunk of the download, headers and text, is written at debug,
/// naming the account and how far it has got.
fn each_chunk_of_the_download_is_written_at_debug(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn start_the_download(")?;
    let headers = between(&body, "Ok(done) => {", "Err(e) => {")?;
    let call = the_first_log_call(headers)
        .ok_or("a chunk of headers landing writes nothing to the log")?;
    if call.level != "debug" {
        return Err(format!(
            "a chunk of headers is written at {}, where hundreds of chunks per account would \
             fill an info log",
            call.level
        ));
    }
    call.names(
        &[
            "account.name",
            "folder.name",
            "done.held",
            "done.total_on_server",
        ],
        "the headers chunk line",
    )?;
    let text = between(
        &body,
        "Ending::WentThroughTheWholeList => {",
        "Ending::ReadingWasTurnedOff(",
    )?;
    let call =
        the_first_log_call(text).ok_or("a chunk of text landing writes nothing to the log")?;
    if call.level != "debug" {
        return Err(format!(
            "a chunk of text is written at {}, not debug",
            call.level
        ));
    }
    call.names(
        &["account.name", "text.fetched", "to_fetch"],
        "the text chunk line",
    )
}

/// What the server answered when it stopped is written at info or above,
/// in this program's clause for the failure's kind, for a refused chunk of
/// headers and for the text download; and the wait before the next try.
fn what_the_server_answered_is_written_at_info_or_above(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn start_the_download(")?;
    let refused = between(&body, "Ok(done) => {", "break 'accounts;")?;
    let refused = between(
        refused,
        "Err(e) => {",
        "ended = WhyTheRunEnded::AServerStopped;",
    )?;
    let call = the_first_log_call(refused)
        .ok_or("a refused chunk of headers writes nothing to the log")?;
    if !call.at_or_above_info() {
        return Err(format!(
            "a refused chunk of headers is written at {}, below info",
            call.level
        ));
    }
    call.names(
        &["account.name", "folder.name", "as_a_clause()"],
        "the refused chunk line",
    )?;
    let stopped = between(
        &body,
        "Ending::TheServerStoppedAnswering {",
        "ended = WhyTheRunEnded::AServerStopped;",
    )?;
    let call = the_first_log_call(stopped)
        .ok_or("the server stopping the text download writes nothing to the log")?;
    if !call.at_or_above_info() {
        return Err(format!(
            "the text download's stop is written at {}, below info",
            call.level
        ));
    }
    call.names(
        &["account.name", "as_a_clause()", "after"],
        "the text download's stop line",
    )?;
    let wait = between(
        &body,
        "if let Some((wait, failures)) = waiting {",
        "say(UIUpdate::Progress(",
    )?;
    let call = the_first_log_call(wait).ok_or("the wait before the next try is written nowhere")?;
    if !call.at_or_above_info() {
        return Err(format!("the wait is written at {}, below info", call.level));
    }
    call.names(&["wait.as_secs()", "failures"], "the wait line")
}

/// The settings save is written at info, naming the log level and the
/// while-fetching level that were written, and no other setting.
fn the_settings_save_is_written_at_info(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn handle_settings(")?;
    let after_the_send = between(
        &body,
        "send_status(tx, rt, \"Settings saved.\");",
        "\n            }",
    )?;
    let call = the_first_log_call(after_the_send).ok_or(
        "the settings save writes nothing to the log, so a report cannot say when a setting \
         changed",
    )?;
    if call.level != "info" {
        return Err(format!(
            "the settings save is written at {}, not info",
            call.level
        ));
    }
    call.names(
        &["log_level", "announce_while_fetching"],
        "the settings save line",
    )?;
    // A setting's name has an underscore in it; the two named are the only
    // ones a report needs, and the settings are where a person's choices
    // live, so nothing else from them is written.
    let values = the_values_in(call.arguments);
    let other: Vec<&str> = identifiers_in(&values)
        .into_iter()
        .filter(|word| {
            word.contains('_')
                && !["log_level", "announce_while_fetching", "app_config"].contains(word)
        })
        .collect();
    if !other.is_empty() {
        return Err(format!(
            "the settings save line names {other:?}, which is more of the settings than a report \
             needs"
        ));
    }
    Ok(())
}

/// What the queue holds back from speech is written: content muted at
/// info by its length, a line dropped for capacity at info with its topic,
/// a repeat at debug with its topic; and the words never.
fn what_is_held_back_from_speech_is_written(queue: &str) -> Result<(), String> {
    let muted = between(
        queue,
        "if announcement.kind == Kind::Content && self.is_muted() {",
        "return Ok(());",
    )?;
    let call = the_first_log_call(muted).ok_or(
        "content held back because content is muted is written nowhere, so a report of silence \
         cannot be checked",
    )?;
    if call.level != "info" {
        return Err(format!(
            "content held back is written at {}, not info",
            call.level
        ));
    }
    never_the_words(&call, "content held back")?;
    call.names(&[".len()"], "content held back")?;

    let repeat = between(
        queue,
        ".any(|p| p.announcement.text == announcement.text)",
        "return Ok(());",
    )?;
    let call = the_first_log_call(repeat).ok_or("a repeat dropped is written nowhere")?;
    if call.level != "debug" {
        return Err(format!(
            "a repeat dropped is written at {}, not debug",
            call.level
        ));
    }
    call.names(&["topic"], "a repeat dropped")?;
    never_the_words(&call, "a repeat dropped")?;

    let capacity = between(
        queue,
        "if state.pending.len() >= CAPACITY {",
        "let sequence = state.sequence;",
    )?;
    let call =
        the_first_log_call(capacity).ok_or("a line dropped for capacity is written nowhere")?;
    if call.level != "info" {
        return Err(format!(
            "a line dropped for capacity is written at {}, not info",
            call.level
        ));
    }
    call.names(&["topic"], "a line dropped for capacity")?;
    never_the_words(&call, "a line dropped for capacity")
}

/// The words of an announcement are never written by these lines: `.text`
/// may appear only as `.text.len()`.
fn never_the_words(call: &LogCall<'_>, which: &str) -> Result<(), String> {
    if call.arguments.replace(".text.len()", "").contains(".text") {
        return Err(format!(
            "{which} writes the words themselves, which may be message text"
        ));
    }
    Ok(())
}

/// No log call under `src/`, outside test modules, names one of the five
/// secrets as a value; the exception table lists what is still to fix.
fn no_log_call_spells_a_secret() -> Result<(), String> {
    let mut files = Vec::new();
    every_source_file(Path::new("src"), &mut files);
    files.sort();
    if files.len() < 100 {
        return Err(format!(
            "only {} source files were found under src, so this read almost nothing",
            files.len()
        ));
    }
    let mut findings = Vec::new();
    for path in files {
        let file = path.to_string_lossy().replace('\\', "/");
        findings.extend(the_secrets_in(&file, &shipped(&file)));
    }
    let carried = |finding: &Finding| {
        THE_LINES_STILL_TO_FIX
            .iter()
            .any(|(file, names, _)| *file == finding.file && *names == finding.names)
    };
    let new: Vec<String> = findings
        .iter()
        .filter(|finding| !carried(finding))
        .map(|finding| format!("{}:{} names {}", finding.file, finding.line, finding.names))
        .collect();
    if !new.is_empty() {
        return Err(format!(
            "these log calls name a secret or a body as a value, which the log must never \
             carry:\n  {}",
            new.join("\n  ")
        ));
    }
    for (file, names, why) in THE_LINES_STILL_TO_FIX {
        if !findings
            .iter()
            .any(|finding| finding.file == file && finding.names == names)
        {
            return Err(format!(
                "{file} no longer names {names} ({why}), so its row in the exception table is \
                 stale and should go"
            ));
        }
    }
    Ok(())
}

// ── The tests ───────────────────────────────────────────────────────────────

#[test]
fn test_each_folder_of_a_check_is_written_at_info() {
    each_folder_of_a_check_is_written_at_info(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_each_chunk_of_the_download_is_written_at_debug() {
    each_chunk_of_the_download_is_written_at_debug(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_what_the_server_answered_is_written_at_info_or_above() {
    what_the_server_answered_is_written_at_info_or_above(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_settings_save_is_written_at_info() {
    the_settings_save_is_written_at_info(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_what_is_held_back_from_speech_is_written_and_never_the_words() {
    what_is_held_back_from_speech_is_written(&shipped(THE_QUEUE))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_no_log_call_in_the_tree_spells_a_secret_or_a_body() {
    no_log_call_spells_a_secret().unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions, each planting the opposite ──────────────────────────────

/// The real text with one substring replaced, or a panic saying the anchor
/// is gone, so a companion cannot pass by planting nothing.
fn with(text: &str, from: &str, to: &str) -> String {
    assert!(
        text.contains(from),
        "the companion's anchor is not in the text any more, so it plants nothing: {from}"
    );
    text.replacen(from, to, 1)
}

/// The first log call inside the segment of `text` between two anchors,
/// spelled as it stands, so a companion can take it out or move its level.
fn the_call_between(text: &str, from: &str, to: &str) -> (String, String) {
    let segment = between(text, from, to).unwrap_or_else(|why| panic!("{why}"));
    let call = the_first_log_call(segment).expect("the segment holds a log call");
    (call.spelled(), call.arguments.to_string())
}

#[test]
fn test_the_reading_complains_when_the_folder_line_is_missing_or_at_debug() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "fn spawn_mail_sync(").expect("the check");
    let (call, arguments) = the_call_between(&body, "Ok(result) => {", "Err(e) => problems.push(");
    let why = each_folder_of_a_check_is_written_at_info(&with(&app, &call, ""))
        .expect_err("a check with no folder line was passed over");
    assert!(why.contains("writes nothing"), "{why}");
    let why = each_folder_of_a_check_is_written_at_info(&with(
        &app,
        &call,
        &format!("tracing::debug!({arguments})"),
    ))
    .expect_err("a folder line at debug was passed over");
    assert!(why.contains("written at debug"), "{why}");
}

#[test]
fn test_the_reading_complains_when_a_chunk_line_is_missing_or_at_info() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "fn start_the_download(").expect("the download");
    let (call, arguments) = the_call_between(&body, "Ok(done) => {", "Err(e) => {");
    let why = each_chunk_of_the_download_is_written_at_debug(&with(&app, &call, ""))
        .expect_err("a download with no headers chunk line was passed over");
    assert!(why.contains("writes nothing"), "{why}");
    let why = each_chunk_of_the_download_is_written_at_debug(&with(
        &app,
        &call,
        &format!("tracing::info!({arguments})"),
    ))
    .expect_err("a chunk line at info was passed over");
    assert!(why.contains("written at info"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_text_downloads_stop_is_not_written() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "fn start_the_download(").expect("the download");
    let (call, _) = the_call_between(
        &body,
        "Ending::TheServerStoppedAnswering {",
        "ended = WhyTheRunEnded::AServerStopped;",
    );
    let why = what_the_server_answered_is_written_at_info_or_above(&with(&app, &call, ""))
        .expect_err("a text download stopping in silence was passed over");
    assert!(why.contains("writes nothing"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_settings_save_line_is_missing_or_says_more() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "fn handle_settings(").expect("the handler");
    let (call, arguments) = the_call_between(
        &body,
        "send_status(tx, rt, \"Settings saved.\");",
        "\n            }",
    );
    let why = the_settings_save_is_written_at_info(&with(&app, &call, ""))
        .expect_err("a save with no line was passed over");
    assert!(why.contains("writes nothing"), "{why}");
    let saying_more = format!(
        "tracing::info!({}, mgr.app_config().default_account_id)",
        arguments.trim_end()
    );
    let why = the_settings_save_is_written_at_info(&with(&app, &call, &saying_more))
        .expect_err("a save line naming another setting was passed over");
    assert!(why.contains("more of the settings"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_muted_line_is_missing_or_writes_the_words() {
    let queue = shipped(THE_QUEUE);
    let (call, _) = the_call_between(
        &queue,
        "if announcement.kind == Kind::Content && self.is_muted() {",
        "return Ok(());",
    );
    let why = what_is_held_back_from_speech_is_written(&with(&queue, &call, ""))
        .expect_err("a queue that says nothing when it mutes was passed over");
    assert!(why.contains("written nowhere"), "{why}");
    let why = what_is_held_back_from_speech_is_written(&with(
        &queue,
        &call,
        "tracing::info!(\"Held back: {}\", announcement.text)",
    ))
    .expect_err("a muted line writing the words was passed over");
    assert!(why.contains("writes the words"), "{why}");
}

#[test]
fn test_the_guard_reads_values_and_not_the_prose_around_them() {
    // "password" as a word in the message is not a value; the same word as
    // an argument, an interpolation, a field or a nested call is.
    let prose = "tracing::warn!(\"Could not read the saved password for {account_id}: {e}\");\n";
    assert!(
        the_secrets_in("x.rs", prose).is_empty(),
        "prose was read as a value"
    );
    let bare = "fn f() {\n    tracing::info!(\"{}\", password);\n}\n";
    assert_eq!(
        the_secrets_in("x.rs", bare),
        vec![Finding {
            file: "x.rs".into(),
            line: 2,
            names: "password".into()
        }]
    );
    let inline = "tracing::debug!(\"the body: {body_plain}\");\n";
    assert_eq!(the_secrets_in("x.rs", inline)[0].names, "body_plain");
    let field = "tracing::info!(\"{}\", account.password);\n";
    assert_eq!(the_secrets_in("x.rs", field)[0].names, "password");
    let structured = "tracing::warn!(token = %access_token, \"refreshing\");\n";
    assert_eq!(the_secrets_in("x.rs", structured)[0].names, "access_token");
    let nested = "tracing::info!(\"{}\", mask(\")\", refresh_token));\n";
    assert_eq!(the_secrets_in("x.rs", nested)[0].names, "refresh_token");
    let escaped = "tracing::info!(\"{{password}} {}\", n);\n";
    assert!(
        the_secrets_in("x.rs", escaped).is_empty(),
        "an escaped brace was read as an interpolation"
    );
    let longer = "tracing::info!(\"{}\", password_was_checked);\n";
    assert!(
        the_secrets_in("x.rs", longer).is_empty(),
        "a longer identifier was read as the secret"
    );
    let two = "tracing::info!(\"{}\", a);\ntracing::info!(\"{body_html}\");\n";
    assert_eq!(the_secrets_in("x.rs", two)[0].line, 2);
}

#[test]
fn test_the_guard_does_not_read_test_modules() {
    let only_in_a_test = "fn f() {}\n#[cfg(test)]\nmod tests {\n    fn g() {\n        \
                          tracing::info!(\"{}\", password);\n    }\n}\n";
    assert!(the_secrets_in("x.rs", &what_ships(only_in_a_test)).is_empty());
    assert_eq!(
        the_secrets_in("x.rs", only_in_a_test).len(),
        1,
        "the raw text does hold one"
    );
}
