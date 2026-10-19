//! An unreadable settings file is kept and said (ledger 889, 13.2-31.9).
//!
//! Until 13.2-31.9 a settings file a disk fault or a hand edit had made
//! unreadable was said only in the log, and OK in Settings wrote the built-in
//! settings over it. Start-up now keeps it under a name of its own, puts
//! settings that allow no change in its place, and says so once after the
//! window is up and the first-run question has been answered; Settings
//! refuses rather than opening on built-in settings its OK would save.
//!
//! The keeping is run here through the real path, `AppPaths::resolve`, with
//! the profile pointed at a folder of its own. The start-up wiring and the
//! Settings refusal are read as text over `what_ships` of the main window's
//! source, each reading with a companion that plants the fault and is caught.
//!
//! **This never reaches the settings of whoever runs it.** `WIXEN_MAIL_DATA`
//! is set once, before anything in this process resolves a path, and only the
//! one case that keeps anything reads it.

use std::path::Path;
use std::sync::OnceLock;
use wixen_mail::application::unreadable_settings::{keep_at_start_up, what_start_up_says};
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::config::{Fate, SettingsFile};

/// The profile of this process, set once before anything resolves a path.
fn the_profile() -> &'static Path {
    static PROFILE: OnceLock<tempfile::TempDir> = OnceLock::new();
    PROFILE
        .get_or_init(|| {
            let profile = tempfile::tempdir().expect("a profile of its own");
            // SAFETY: set once, inside the `OnceLock`, before any thread of
            // this process reads it; the one case that resolves a path comes
            // through here first.
            unsafe { std::env::set_var("WIXEN_MAIL_DATA", profile.path()) };
            profile
        })
        .path()
}

#[test]
fn test_start_up_keeps_a_broken_settings_file_through_the_real_path_and_once() {
    let config = the_profile().join("config");
    std::fs::create_dir_all(&config).expect("the settings folder");
    std::fs::write(config.join("app_config.json"), "{\"theme\": ").expect("broken settings");

    let kept = keep_at_start_up().expect("the keeping to run");

    assert_eq!(kept.folder, config);
    assert_eq!(kept.unreadable.len(), 1, "{kept:#?}");
    assert_eq!(kept.unreadable[0].which, SettingsFile::TheSettings);
    let Fate::KeptAs(name) = &kept.unreadable[0].fate else {
        panic!("the broken settings were not kept: {kept:#?}");
    };
    assert_eq!(
        std::fs::read_to_string(config.join(name)).expect("the kept copy"),
        "{\"theme\": "
    );
    let said = what_start_up_says(&kept).expect("a sentence for what was kept");
    assert!(said.contains(name.as_str()), "{said}");
    assert!(said.contains(&config.display().to_string()), "{said}");
    assert!(
        keep_at_start_up().expect("a second start").is_empty(),
        "a second start kept the fresh settings again"
    );
}

// ── The window, read as text ───────────────────────────────────────────────

const THE_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_QUESTION: &str = "ask_about_the_alpha_once(&frame, &a11y);";
const THE_SENTENCE: &str = "what_start_up_says(";

fn the_shipping_window() -> String {
    let whole = std::fs::read_to_string(THE_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// One top-level item's text, from its opening line to the brace that closes
/// it at the left margin, which is where rustfmt puts it.
fn body_of<'a>(source: &'a str, opening: &str) -> Option<&'a str> {
    let start = source.find(opening)?;
    let rest = &source[start..];
    rest.find("\n}\n").map(|end| &rest[..end])
}

/// Lines of text that are code rather than comments.
fn code_of(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// What is wrong with how start-up says what it kept.
fn what_is_wrong_with_the_start_up_sentence(source: &str) -> Vec<String> {
    let code = code_of(source);
    let calls: Vec<usize> = code.match_indices(THE_SENTENCE).map(|(at, _)| at).collect();
    let [call] = calls[..] else {
        return vec![format!(
            "the sentence is said {} times, not once",
            calls.len()
        )];
    };
    let Some(question) = code.find(THE_QUESTION) else {
        return vec!["start-up never asks the first-run question".to_string()];
    };
    let mut wrong = Vec::new();
    if call < question {
        wrong.push("the sentence is said before the first-run question".to_string());
    } else if code[question..call].contains('}') {
        wrong.push(
            "the sentence is not in the block that skips the first-run question in a scan run"
                .to_string(),
        );
    }
    let after = &code[call..];
    let three_lines = after
        .match_indices('\n')
        .nth(2)
        .map_or(after.len(), |(at, _)| at);
    if !after[..three_lines].contains("send_status(") {
        wrong.push("the sentence is not said through send_status".to_string());
    }
    wrong
}

/// What is wrong with Settings when the stored settings cannot be read.
fn what_is_wrong_with_settings(source: &str) -> Vec<String> {
    let Some(settings) = body_of(source, "fn handle_settings(").map(code_of) else {
        return vec!["handle_settings is not in the main window".to_string()];
    };
    let mut wrong = Vec::new();
    if !settings.contains("ConfigManager::load_stored()") {
        wrong.push("Settings does not read the stored settings through load_stored".to_string());
    }
    if settings.contains("ConfigManager::new()") {
        wrong.push("Settings builds settings of its own that its OK would save".to_string());
    }
    if !settings.contains("settings_will_not_open(") {
        wrong.push("Settings never says why it did not open".to_string());
    }
    wrong
}

/// The sentence as a planted copy says it, on one line.
fn the_sentence_said() -> String {
    format!(
        "if let Some(said) = {THE_SENTENCE}&kept) {{ send_status(&scan_tx, &scan_rt, &said); }}"
    )
}

#[test]
fn test_start_up_says_what_it_kept_once_after_the_first_run_question() {
    let found = what_is_wrong_with_the_start_up_sentence(&the_shipping_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_start_up_reading_sees_the_sentence_said_before_the_question() {
    let planted = the_shipping_window()
        .replacen(THE_SENTENCE, "said_by_nobody(", 1)
        .replacen(
            THE_QUESTION,
            &format!("{}\n{THE_QUESTION}", the_sentence_said()),
            1,
        );

    assert_eq!(
        what_is_wrong_with_the_start_up_sentence(&planted),
        ["the sentence is said before the first-run question"]
    );
}

#[test]
fn test_the_start_up_reading_sees_the_sentence_said_twice() {
    let planted = the_shipping_window().replacen(
        THE_QUESTION,
        &format!("{THE_QUESTION} {}", the_sentence_said()),
        1,
    );

    assert_eq!(
        what_is_wrong_with_the_start_up_sentence(&planted),
        ["the sentence is said 2 times, not once"]
    );
}

#[test]
fn test_settings_refuses_rather_than_opening_on_settings_of_its_own() {
    let found = what_is_wrong_with_settings(&the_shipping_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_settings_reading_sees_settings_opened_on_their_own_again() {
    let shipping = the_shipping_window();
    let body = body_of(&shipping, "fn handle_settings(").expect("handle_settings");
    let planted = shipping.replacen(
        body,
        &body.replacen(
            "ConfigManager::load_stored()",
            "ConfigManager::new().and_then(|mut m| { let _ = m.load(); Ok(m) })",
            1,
        ),
        1,
    );

    let found = what_is_wrong_with_settings(&planted);

    assert!(
        found.contains(&"Settings builds settings of its own that its OK would save".to_string()),
        "{found:#?}"
    );
}
