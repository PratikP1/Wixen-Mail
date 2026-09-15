//! A window that holds a browser waits for the browser before it goes.
//!
//! `WebView::builder(..).build()` returns before WebView2 has made the
//! browser. The environment and then the controller arrive later, each as a
//! COM completion the event loop delivers, and the control reports
//! `on_created` (or `on_error`) when the last of them has. wxWidgets 3.3.2,
//! which wxdragon 0.9.17 builds, hands those completions a bare pointer to the
//! control and does not take it back when the control is destroyed (wxWidgets
//! #26491, fixed upstream on 2026-08-17 for 3.3.4, which is not released).
//! Destroy the window while a completion is still owed and it lands on freed
//! memory. Windows reports an exception escaping a callback as exit code
//! 0xc000041d and says nothing else: no panic, no message, no test name.
//!
//! Warm, the browser takes about a quarter of a second to arrive on the
//! machine this was written on (measured 2026-09-15 with `on_created`, three
//! runs, 246 to 273 ms). On GitHub's runner that day it took over three
//! seconds, and on any machine the first time after the WebView2 runtime
//! updates itself it takes seconds. Somebody who opens Compose by mistake and
//! presses Escape at once is inside that window. `tests/theme_reach.rs` tore
//! its windows down two seconds after building them and was inside it on
//! every runner run that day.
//!
//! So a window holding a browser is not destroyed; it is handed to
//! [`BrowserReady::destroy_when_ready`], hidden at once so the person sees it
//! go, and destroyed the moment the browser reports. A browser that never
//! reports leaves the hidden window alive. That is a leak, and the
//! alternative was the crash above.

use std::cell::RefCell;
use std::rc::Rc;
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::WebView;

/// Whether one browser has reported, and what is waiting for it to.
#[derive(Clone)]
pub struct BrowserReady {
    state: Rc<RefCell<State>>,
}

struct State {
    reported: bool,
}

impl BrowserReady {
    /// Watches `view` from the moment it is built. Bind this before the event
    /// loop turns, which every builder here does: a report that arrived before
    /// anything was listening would be a report nobody heard.
    ///
    /// `on_error` counts as a report because it is what a creation that failed
    /// sends, and a browser that failed to be made is one nothing is owed for.
    /// After the browser has reported, a navigation error arriving on the same
    /// event changes nothing.
    pub fn watch(view: &WebView) -> Self {
        let browser = Self::not_yet();
        view.on_created({
            let browser = browser.clone();
            move |_| browser.report()
        });
        view.on_error({
            let browser = browser.clone();
            move |_| browser.report()
        });
        browser
    }

    fn not_yet() -> Self {
        Self {
            state: Rc::new(RefCell::new(State { reported: true })),
        }
    }

    /// The browser has reported: created, or failed to be. Runs everything
    /// that was waiting, once.
    fn report(&self) {}

    /// Whether the browser has reported.
    pub fn is_ready(&self) -> bool {
        self.state.borrow().reported
    }

    /// Runs `then` now if the browser has reported, and the moment it does
    /// otherwise.
    pub fn when_ready(&self, then: impl FnOnce() + 'static) {
        then();
    }

    /// Hides `window` now and destroys it once the browser it holds has
    /// reported. In place of `window.destroy()` for any window holding a
    /// `WebView`; see the module comment for what destroying one early does.
    pub fn destroy_when_ready(&self, window: impl WxWidget + 'static) {
        window.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn counting(into: &Rc<Cell<u32>>) -> impl FnOnce() + 'static {
        let into = into.clone();
        move || into.set(into.get() + 1)
    }

    #[test]
    fn test_nothing_waiting_runs_before_the_browser_reports() {
        let browser = BrowserReady::not_yet();
        let ran = Rc::new(Cell::new(0));
        browser.when_ready(counting(&ran));
        assert_eq!(ran.get(), 0, "ran before the browser reported");
        assert!(!browser.is_ready(), "ready before the browser reported");
        browser.report();
        assert_eq!(ran.get(), 1, "did not run when the browser reported");
        assert!(browser.is_ready(), "not ready after the browser reported");
    }

    #[test]
    fn test_a_second_report_runs_nothing_twice() {
        let browser = BrowserReady::not_yet();
        let ran = Rc::new(Cell::new(0));
        browser.when_ready(counting(&ran));
        browser.report();
        browser.report();
        assert_eq!(ran.get(), 1);
    }

    #[test]
    fn test_once_the_browser_has_reported_a_new_wait_runs_at_once() {
        let browser = BrowserReady::not_yet();
        browser.report();
        let ran = Rc::new(Cell::new(0));
        browser.when_ready(counting(&ran));
        assert_eq!(ran.get(), 1);
    }

    #[test]
    fn test_a_wait_queued_by_something_that_was_waiting_runs_too() {
        let browser = BrowserReady::not_yet();
        let ran = Rc::new(Cell::new(0));
        browser.when_ready({
            let browser = browser.clone();
            let ran = ran.clone();
            move || browser.when_ready(counting(&ran))
        });
        browser.report();
        assert_eq!(
            ran.get(),
            1,
            "a wait queued while the queue was draining was lost"
        );
    }
}
