//! Work the main window's tick hands on, to run after the tick has returned.
//!
//! The main window polls every 50 ms, and that one handler drains every update
//! the workers send, raises what is due and puts the question about folders a
//! server stopped listing. Seven of the things it does open a modal window: the
//! Due now window, the folders question, the two questions about an update, the
//! question before a counted rule runs, the question about a key in an
//! attachment and the question about mail a block caught. Opened from inside
//! the handler, each window's loop ran the next tick inside the one that opened
//! it.
//!
//! At wxDragon 0.9.17 that works by accident: the event trampoline calls a
//! handler again while an outer call of it is still running. From 0.9.23 every
//! handler sits behind a guard that skips a second call while the first runs,
//! with a warning, so a tick that opened a window would stop the tick until the
//! window closed: no mail announced, no sync or send result handled, no further
//! reminder, for as long as somebody left a reminder on screen.
//!
//! So the tick decides, says what it says, and hands the window and whatever
//! follows its answer to [`after_the_tick`]. The work waits in a queue on this
//! thread and runs from the application's idle handler, through
//! `wxdragon::call_after`, which a modal loop keeps calling. The tick is a
//! different handler, so it runs beside an open window and never inside itself.
//!
//! The queue is per thread and the closure handed to `call_after` captures
//! nothing, because `call_after` wants a `Send` closure and the tick's state is
//! `Rc` and `RefCell`. One piece of work runs at a time: a drain that finds
//! another one running returns at once, and the running one takes what arrived
//! once the work in hand returns. So a second window waits for the first to
//! close rather than opening over it.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

/// A piece of work handed on by the tick.
type Work = Box<dyn FnOnce()>;

thread_local! {
    /// What is waiting to run, oldest first.
    static WAITING: RefCell<VecDeque<Work>> = const { RefCell::new(VecDeque::new()) };
    /// Whether a drain has been handed to `call_after` and has not started.
    static A_DRAIN_IS_QUEUED: Cell<bool> = const { Cell::new(false) };
    /// Whether a drain is running a piece of work right now.
    static A_DRAIN_IS_RUNNING: Cell<bool> = const { Cell::new(false) };
}

/// Run `work` after the handler that is running now has returned, after any
/// work handed on before it, and never while another piece is running.
pub fn after_the_tick(work: impl FnOnce() + 'static) {
    WAITING.with(|waiting| waiting.borrow_mut().push_back(Box::new(work)));
    queue_a_drain();
}

/// Hand a drain to the event loop unless one is already waiting there.
fn queue_a_drain() {
    if !A_DRAIN_IS_QUEUED.replace(true) {
        wxdragon::call_after(Box::new(drain));
    }
}

/// Run what is waiting, one piece at a time, until nothing is.
///
/// Returns at once when another drain is running further up this thread's
/// stack, which is the case while a piece of work holds a modal window open
/// and its loop calls the idle handler again. The running drain takes what
/// arrived when the window closes.
fn drain() {
    A_DRAIN_IS_QUEUED.set(false);
    if A_DRAIN_IS_RUNNING.replace(true) {
        return;
    }
    let _running = Running;
    while let Some(work) = the_next_piece() {
        work();
    }
}

/// The oldest waiting piece, with the queue's borrow released before it runs,
/// so the work can hand on more work.
fn the_next_piece() -> Option<Work> {
    WAITING.with(|waiting| waiting.borrow_mut().pop_front())
}

/// Says a drain is running, and stops saying so however the drain ends.
///
/// Without it a piece of work that panicked would leave the flag set, and
/// every drain after would return at once: nothing handed on would ever run
/// again, and nothing would say why.
struct Running;

impl Drop for Running {
    fn drop(&mut self) {
        A_DRAIN_IS_RUNNING.set(false);
        // Only after a panic is anything still waiting here, since the loop
        // runs until the queue is empty. What is left gets a drain of its own.
        if WAITING.with(|waiting| !waiting.borrow().is_empty()) {
            queue_a_drain();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// What ran, in order.
    fn a_log() -> Rc<RefCell<Vec<&'static str>>> {
        Rc::new(RefCell::new(Vec::new()))
    }

    fn noting(log: &Rc<RefCell<Vec<&'static str>>>, what: &'static str) -> impl FnOnce() + 'static {
        let log = Rc::clone(log);
        move || log.borrow_mut().push(what)
    }

    #[test]
    fn test_work_handed_on_does_not_run_until_the_event_loop_drains_it() {
        let log = a_log();
        after_the_tick(noting(&log, "first"));
        after_the_tick(noting(&log, "second"));
        assert!(
            log.borrow().is_empty(),
            "work ran inside the handler that handed it on: {:?}",
            log.borrow()
        );
        drain();
        assert_eq!(*log.borrow(), ["first", "second"]);
    }

    #[test]
    fn test_work_handed_on_while_a_piece_runs_waits_for_that_piece_to_end() {
        // The modal case: the running piece holds a window open, its loop
        // calls the idle handler, and that drain must not run the next window.
        let log = a_log();
        let inner = Rc::clone(&log);
        after_the_tick(move || {
            inner.borrow_mut().push("window opens");
            after_the_tick(noting(&inner, "second window"));
            drain();
            inner.borrow_mut().push("window closes");
        });
        drain();
        assert_eq!(
            *log.borrow(),
            ["window opens", "window closes", "second window"]
        );
    }

    #[test]
    fn test_a_piece_that_panics_does_not_stop_the_queue() {
        let log = a_log();
        after_the_tick(|| panic!("a piece of work failed"));
        after_the_tick(noting(&log, "after the panic"));
        let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(drain));
        assert!(ran.is_err(), "the panicking piece did not panic");
        drain();
        assert_eq!(*log.borrow(), ["after the panic"]);
    }
}
