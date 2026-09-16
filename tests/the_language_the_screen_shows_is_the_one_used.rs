//! The spelling language the settings screen shows is the one the checker
//! will use.
//!
//! The screen and the checker read one stored value, and for a year they
//! read it differently: the screen looked for a row whose tag was spelled
//! exactly as stored and fell back to row 0, and the checker took the first
//! of the family Windows listed. Both landed on English (Caribbean) for a
//! stored bare "en" on a machine set to English (United States), because
//! Windows lists the Caribbean first (#21). Every profile written before
//! 2026-09-03 holds that bare "en".
//!
//! Built rather than read from the source, for the reason
//! `tests/every_event_has_a_control.rs` gives: which row a `Choice` ends up
//! selecting is a property of the running window. The real General tab is
//! built with a stored value, and the tag of the selected row is read back
//! through `read_settings`, which is what pressing OK writes, so the claim
//! is about what a person would see and what they would keep.
//!
//! One `#[test]` function calling `wxdragon::main` once, for the reason every
//! target under `tests/` that builds a window records: wxWidgets supports one
//! application per process. Three dialogs are built inside that one call,
//! one per stored value, and each is destroyed before the next is built, the
//! way `tests/checkbox_labels.rs` builds one item form per kind.

use std::sync::{Arc, Mutex};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_settings;
use wixen_mail::service::spellcheck::{available_languages, language_to_use, system_language};
use wxdragon::prelude::*;

/// One stored value the screen showed wrongly: what was stored, what the
/// screen would have kept, and what it should have.
type Wrong = Vec<(String, String, String)>;

/// The tag the screen keeps for a stored value, read the way OK reads it.
fn what_the_screen_keeps_for(frame: &Frame, a11y: &Arc<Accessibility>, stored: &str) -> String {
    let config = AppConfig {
        language: stored.to_string(),
        ..AppConfig::default()
    };
    let widgets = wx_settings::build_settings_dialog(frame, &config, &[], false, a11y);
    let kept = wx_settings::read_settings(&widgets, &config).language;
    widgets.dialog.destroy();
    kept
}

#[test]
fn test_the_language_the_screen_shows_is_the_one_the_checker_uses() {
    // `Accessibility::new` builds an audio player, and on a machine where a
    // sound device opens and then does not work the first write faults. See
    // `tests/every_event_has_a_control.rs` for why this is not a skip.
    if std::env::var_os("WIXEN_NO_AUDIO").is_none() && cfg!(not(target_os = "windows")) {
        unsafe { std::env::set_var("WIXEN_NO_AUDIO", "1") };
    }

    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));
            let offered = available_languages();
            let machine = system_language();

            // A bare "en", which is what every profile written before
            // 2026-09-03 stores. The screen keeps what the checker resolves
            // it to: this machine's own region on a machine set to English,
            // and on a runner with no spell checking, the built-in "en".
            let resolved = language_to_use("en", machine.as_deref(), &offered)
                .unwrap_or_else(|| "en".to_string());
            let kept = what_the_screen_keeps_for(&frame, &a11y, "en");
            if kept != resolved {
                wrong.push(("en".to_string(), kept, resolved));
            }

            // A tag somebody chose is kept exactly as chosen, whether or not
            // this machine can check it.
            let kept = what_the_screen_keeps_for(&frame, &a11y, "en-AU");
            if kept != "en-AU" {
                wrong.push(("en-AU".to_string(), kept, "en-AU".to_string()));
            }

            // A tag nothing offers is kept as stored rather than turned into
            // whatever sits in row 0.
            let kept = what_the_screen_keeps_for(&frame, &a11y, "zz-ZZ");
            if kept != "zz-ZZ" {
                wrong.push(("zz-ZZ".to_string(), kept, "zz-ZZ".to_string()));
            }

            drop(wrong);
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let wrong = wrong.lock().unwrap();
    assert!(
        wrong.is_empty(),
        "the settings screen shows a spelling language other than the one the \
         checker uses:\n{}",
        wrong
            .iter()
            .map(|(stored, kept, wanted)| format!(
                "  stored {stored:?}: the screen would keep {kept:?}, the checker uses {wanted:?}"
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
