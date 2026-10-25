//! The message window's automatic save ticks on a timer of its own.
//!
//! The window has a second timer, `later`, whose handler runs whatever work is
//! waiting: a spelling check, a look-up of a name, Send Later's question. At
//! wxDragon 0.9.17 a timer event reaches every handler bound on the timer's
//! owner, so a save timer owned by the window would run `later`'s handler at
//! each tick and start that work early; at 0.9.23 a timer reaches only the
//! handler bound to its own id, so a save timer with no handler would save
//! nothing. `wx_compose::save_on_a_timer_of_its_own` is what the composer
//! hands its save to, and this runs it.
//!
//! A dialog stands in for the window, with a child panel standing in for the
//! message body, and a timer on the dialog stands in for `later`: never
//! started, with a handler that would take what is waiting. The save is asked
//! for every 150 ms, an interval only a test can ask for, since the setting
//! starts at one minute. A timer owned by another window ends the run after a
//! second and a half and writes down what was wrong.
//!
//! One `#[test]` function calling `wxdragon::main` once, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per
//! process. Nothing is shown, so nothing reaches the screen of whoever runs it.

#![cfg(windows)]

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wixen_mail::presentation::wx_compose::save_on_a_timer_of_its_own;
use wxdragon::prelude::*;

const SAVE_EVERY: Duration = Duration::from_millis(150);
const RUN_FOR_MS: i32 = 1_500;
/// Ten are due in the run; three is the fewest that shows it repeats on its
/// own, with room for a slow machine.
const FEWEST_SAVES: u32 = 3;
const WHAT_WAS_WAITING: &str = "a spelling check";

/// What the run hands back: what went wrong, or why it could not run.
type Outcome = Arc<Mutex<Option<Result<Vec<String>, String>>>>;

/// Ends the event loop from a closure that captures nothing, so nothing
/// owned by this thread is sent to another.
fn end_the_loop() {
    wxdragon::call_after(Box::new(|| {
        if let Some(app) = wxdragon::get_app_instance() {
            app.exit_main_loop();
        }
    }));
}

/// What the run saw.
#[derive(Debug, Default)]
struct Seen {
    saves: Cell<u32>,
    stand_in_ticks: Cell<u32>,
    waiting: Cell<Option<&'static str>>,
}

impl Seen {
    /// Every property the run should have shown that it did not.
    fn what_was_wrong(&self) -> Vec<String> {
        let mut wrong = Vec::new();
        if self.saves.get() < FEWEST_SAVES {
            wrong.push(format!(
                "the save ran {} times in {RUN_FOR_MS} ms at every {} ms, and at least \
                 {FEWEST_SAVES} were owed",
                self.saves.get(),
                SAVE_EVERY.as_millis()
            ));
        }
        if self.stand_in_ticks.get() != 0 {
            wrong.push(format!(
                "the window's other timer, never started, heard {} ticks",
                self.stand_in_ticks.get()
            ));
        }
        if self.waiting.get() != Some(WHAT_WAS_WAITING) {
            wrong.push(format!(
                "what was waiting on the other timer was taken: {:?} at the end",
                self.waiting.get()
            ));
        }
        wrong
    }
}

#[test]
fn test_the_automatic_save_ticks_on_its_own_and_reaches_nothing_else() {
    let outcome = Outcome::default();
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |_| {
            let keeper = Frame::builder().with_title("The end timer's owner").build();
            let dialog = Dialog::builder(&keeper, "The message window, standing in").build();
            let body = Panel::builder(&dialog).build();
            let seen = Rc::new(Seen {
                waiting: Cell::new(Some(WHAT_WAS_WAITING)),
                ..Seen::default()
            });

            let later = Timer::new(&dialog);
            later.on_tick({
                let seen = Rc::clone(&seen);
                move |_| {
                    seen.stand_in_ticks.set(seen.stand_in_ticks.get() + 1);
                    seen.waiting.take();
                }
            });

            let save = save_on_a_timer_of_its_own(&body, SAVE_EVERY, {
                let seen = Rc::clone(&seen);
                move || seen.saves.set(seen.saves.get() + 1)
            });

            let end = Timer::new(&keeper);
            end.on_tick({
                let outcome = outcome.clone();
                move |_| {
                    println!(
                        "saves: {}, the other timer's ticks: {}",
                        seen.saves.get(),
                        seen.stand_in_ticks.get()
                    );
                    if let Ok(mut slot) = outcome.lock()
                        && slot.is_none()
                    {
                        *slot = Some(Ok(seen.what_was_wrong()));
                    }
                    end_the_loop();
                }
            });
            if !end.start(RUN_FOR_MS, true) {
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(Err("the end timer refused to start".into()));
                }
                return end_the_loop();
            }
            std::mem::forget(later);
            std::mem::forget(save);
            std::mem::forget(end);
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let outcome = outcome.lock().expect("the outcome").take();
    match outcome {
        Some(Ok(wrong)) => assert!(
            wrong.is_empty(),
            "the automatic save did not tick on its own:\n  {}",
            wrong.join("\n  ")
        ),
        Some(Err(why)) => panic!("the run could not be made: {why}"),
        None => panic!("the run said nothing"),
    }
}
