//! The separate Wixen Mail window a link opens in, which is a process of its
//! own.
//!
//! #80's third place. The Reading tab offers three: the default browser, the
//! message view, and a separate Wixen Mail window. 11-11.1 built the first
//! two and left the third opening the browser with a line, because a
//! "separate window" inside the running process would not be separate in the
//! way the issue's third item is about. Every `WebView` in one process shares
//! one WebView2 environment under wxdragon 0.9.17, and nothing in either
//! crate can clear it or point a second control somewhere else: the data path
//! is decided once, from `wxStandardPaths::Get().GetUserLocalDataDir()`, and
//! wxdragon exposes no setter for it. So a page opened in a second frame
//! would sit in the message preview's cookie jar, which is the one thing that
//! route exists to prevent.
//!
//! A process of its own with an application name of its own is the isolation
//! this toolkit can reach. `wxStandardPathsBase::AppendAppInfo` appends
//! `wxTheApp->GetAppName()` to the local data folder, so setting the name
//! before the browser control is built decides where that process's profile
//! goes, and nothing else about the two processes is shared at all.
//!
//! # What a page process is not
//!
//! It is answered in `main` where `--help` and `--version` are, before the
//! data folder is prepared, before the log file is opened, before the
//! single-copy claim and before the handover. It opens no database, holds no
//! mutex, reads no credential, and is not the copy a later start hands its
//! `mailto:` link to. What it has is one frame, one browser control and the
//! announcements that window makes.
//!
//! # What this window cannot do, and why
//!
//! It cannot decide a navigation by its address. wxWidgets keeps a navigating
//! event's address in `wxWebViewEvent::GetURL` and a new-window request's in
//! the same place, and wxdragon 0.9.17 hands `WebViewEventData` only the
//! command event's string, which neither sets. That is 11-11.1's finding,
//! measured against a built control, and it applies here too. So the address
//! is checked at the two boundaries this program owns: the command line it
//! was started with, and the anchor the page's own listener posts. A
//! main-frame navigation the listener did not catch is allowed, because this
//! window exists to browse that page and its profile is its own; a popup a
//! script asks for is refused and said, because the event carries no address
//! for this window to load instead.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use wxdragon::event::WebViewEvents;
use wxdragon::event::webview_events::WebViewEventData;
use wxdragon::event::window_events::WindowEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

use crate::application::opening_links;
use crate::common::paths;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::html_renderer::HtmlRenderer;
use crate::presentation::page_links;
use crate::presentation::panes;

/// The flag that opens one page in a window of this program's own.
///
/// One spelling, in one place, read by the command line parser, quoted in
/// `--help`, and written by the route in `wx_app`. `scan_target::FLAG` is
/// here for the same reason and for the accident it was written after: two
/// spellings of one flag left every dialog scan walking the main window.
pub const FLAG: &str = "--show-page";

/// What a page process exits with when it was not given a page.
///
/// A `mailto:` link is not a page and the route never sends one here, so an
/// address this refuses arrived by hand or from something that is not this
/// program. Nothing opens, and the reason goes to the crash file, which is
/// where a start that must stop says why.
pub const NOT_A_PAGE: i32 = 2;

/// The address this window will load, or nothing.
///
/// Narrower than the sanitiser every link in a message goes through, which
/// also allows `mailto:` and `tel:`: those are for something other than this
/// program to answer, and a window that showed one would be a browser aimed
/// at an address bar it does not have. The parent sanitises before it spawns
/// and this sanitises again, because a process boundary is a boundary and
/// the thing on the other side of it may not have been this program.
pub fn may_be_followed(address: &str) -> Option<String> {
    let safe = HtmlRenderer::safe_external_url(address)?;
    let scheme = safe.to_ascii_lowercase();
    (scheme.starts_with("http://") || scheme.starts_with("https://")).then_some(safe)
}

/// The window's title.
///
/// The host first, so a page cannot put its own words where the program's
/// name goes, and the program's name last, so a window in the task switcher
/// says whose it is (T-12-08).
pub fn the_windows_title(page_title: Option<&str>, address: &str) -> String {
    match page_title.map(str::trim).filter(|title| !title.is_empty()) {
        Some(title) => format!("{title} - Wixen Mail"),
        // Before the page has a title there is only the address, and the
        // host is the part of it worth hearing: the rest can be two hundred
        // characters of token. The same sentence the message view says when
        // it starts loading, so the two surfaces cannot come to name a page
        // differently.
        None => format!(
            "{} - Wixen Mail",
            opening_links::what_is_said_when_opening(address)
        ),
    }
}

/// What is said when Backspace or Alt+Left is pressed with nothing behind.
///
/// Said rather than left silent, because a key that does nothing is
/// indistinguishable from one that is broken.
pub const THIS_IS_THE_FIRST_PAGE: &str = "This is the first page";

/// What is said when a page's script asks for a second window.
///
/// It is not opened, and this window cannot open it instead: wxWidgets
/// carries a new-window request's address in `wxWebViewEvent::GetURL` and
/// wxdragon 0.9.17 hands the handler only the command event's string, which
/// that event never sets, so there is no address to load. A link with
/// `target="_blank"` never gets this far, because the page's own listener
/// takes the click and posts the address; what does is `window.open` from a
/// script.
pub const A_SECOND_WINDOW_WAS_NOT_OPENED: &str =
    "The page asked for another window, which was not opened.";

/// What this window shows once it is built.
pub enum What<'a> {
    /// A page at this address, which the browser fetches.
    APageAt(&'a str),
    /// A document already in hand, which the accessibility scan uses: the
    /// runner has no network, and a scan has to meet the same window every
    /// time it runs.
    ThisDocument { html: &'a str, as_if_from: &'a str },
}

/// The window, and the control its announcements are carried on.
pub struct ThePageWindow {
    /// The frame. Whoever built it shows it: a page process raises it, and
    /// the scan leaves it up for the walk.
    pub frame: Frame,
    /// The one-pixel static that carries announcements.
    ///
    /// Registered by a process whose only window this is. The scan builds
    /// this window inside an application that already has one registered and
    /// leaves this alone, because registering a second would point the main
    /// window's announcements at a control on a window nobody is looking at.
    pub live_region: isize,
}

impl ThePageWindow {
    /// Put the window in front of whoever asked for it.
    ///
    /// Un-minimised before it is shown, because a window this new is never
    /// minimised and `raise` on a minimised window leaves it minimised.
    /// Written here rather than at each call site so the three steps cannot
    /// come apart: `tests/wired.rs` holds every `frame.raise()` in the main
    /// window to the same shape, and found this one missing it.
    pub fn put_it_in_front(&self) {
        self.frame.iconize(false);
        self.frame.show(true);
        self.frame.raise();
    }
}

/// Build the window on a wx application that is already running.
///
/// Apart from `show` below, the accessibility scan calls this: the scan runs
/// inside the ordinary application, which has already entered its event
/// loop, so the frame and its wiring have to be reachable without one of
/// their own.
pub fn build(
    a11y: &Arc<Accessibility>,
    what: What<'_>,
    when_it_closes: Rc<dyn Fn()>,
) -> ThePageWindow {
    let address = match &what {
        What::APageAt(address) => (*address).to_string(),
        What::ThisDocument { as_if_from, .. } => (*as_if_from).to_string(),
    };
    let frame = Frame::builder()
        .with_title(&the_windows_title(None, &address))
        .with_size(Size::new(900, 700))
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // A one pixel static control that carries announcements, as the main
    // window has. A Win32 static reports its window text as its accessible
    // name, so setting the text and raising a live region change is a
    // complete announcement without moving focus. Built before the browser
    // and still not in front of it: a static text takes no focus, which is
    // what 12-01 measured the page window's first focusable child to be.
    let live_region = StaticText::builder(&frame)
        .with_label("")
        .with_pos(Point::new(0, 0))
        .with_size(Size::new(1, 1))
        .build();

    let page = WebView::builder(&frame)
        .with_backend(WebViewBackend::Edge)
        .build();
    set_accessible_name(&page, "Page");
    sizer.add(&page, 1, SizerFlag::Expand | SizerFlag::All, 0);
    // Not painted: the page's own document sets its colour, as it does in
    // the message preview and the formatted message window. Nothing of this
    // frame shows around a browser that fills it.
    page.enable_context_menu(false);
    page.enable_access_to_dev_tools(false);
    // The browser does not get this application's keys.
    page.enable_browser_accelerator_keys(false);
    // The one thing this window has that the other two do not: somewhere to
    // go back to, since a page here can be followed to another.
    page.enable_history(true);

    // Before the page is loaded, because the script is injected as each
    // document is created and one already loaded has missed it. The same
    // script both other surfaces run, so Escape and F6 leave, an anchor's
    // activation is taken from the browser and posted, and Backspace and
    // Alt+Left ask to go back.
    // It opens the script channel this window's handler below reads, so
    // nothing here opens a second one.
    crate::presentation::wx_app::wire_the_way_out(
        &page,
        THE_SURFACE,
        crate::presentation::wx_app::PageKeys::TheWayOut,
    );

    // Whether the title of the page now loading has been said. Cleared
    // before every load, so each page says its own once rather than on every
    // change a script makes to it.
    let title_said = Rc::new(Cell::new(false));

    page.on_title_changed({
        let a11y = a11y.clone();
        let title_said = title_said.clone();
        let address = address.clone();
        move |event: WebViewEventData| {
            let Some(title) = event.get_string().filter(|title| !title.trim().is_empty()) else {
                return;
            };
            frame.set_title(&the_windows_title(Some(&title), &address));
            if title_said.replace(true) {
                return;
            }
            let _ = a11y.announce(&title, Priority::Normal);
        }
    });

    page.on_error({
        let a11y = a11y.clone();
        move |event: WebViewEventData| {
            let said = opening_links::could_not_be_opened(event.get_int());
            tracing::warn!("{THE_SURFACE}: {said}");
            let _ = a11y.announce(&said, Priority::High);
            // Left up with the reason in the document, so somebody who
            // heard it once can read it again. There is nothing behind this
            // window to come back to, which is what the message view does
            // instead.
            page.set_page(&a_page_saying(&said), "about:blank");
        }
    });

    // Allowed, and this is the one decision in this window that reads
    // differently from the other two surfaces. A navigating event carries no
    // address here either, so nothing can be decided from one; and this
    // window exists to browse the page it was given, in a profile of its
    // own, so a redirect or a link the listener did not catch is the thing
    // it is for. The scheme is checked where an address can be read: the
    // command line, and the anchor the page's own listener posts.
    page.on_navigating({
        move |event: WebViewEventData| {
            tracing::debug!(
                "{THE_SURFACE}: navigating; main frame: {}",
                event.get_int() == Some(1)
            );
        }
    });

    page.on_new_window({
        let a11y = a11y.clone();
        move |event: WebViewEventData| {
            event.event.event.veto();
            tracing::warn!("{THE_SURFACE}: {A_SECOND_WINDOW_WAS_NOT_OPENED}");
            let _ = a11y.announce(A_SECOND_WINDOW_WAS_NOT_OPENED, Priority::Normal);
        }
    });

    page.on_script_message_received({
        let a11y = a11y.clone();
        let title_said = title_said.clone();
        move |event: WebViewEventData| {
            let Some(json) = event.get_string() else {
                return;
            };
            match page_links::what_the_page_posted(&json) {
                // Every link opens here, whatever modifier was held. This
                // window is one page at a time and has nowhere else to put
                // one; the modifiers mean something on the two surfaces that
                // do, where a link has a message to go back to.
                Some(page_links::Posted::Link { href, .. }) => {
                    match may_be_followed(&href) {
                        Some(safe) => {
                            title_said.set(false);
                            let _ = a11y.announce(
                                &opening_links::what_is_said_when_opening(&safe),
                                Priority::Normal,
                            );
                            page.load_url(&safe);
                        }
                        None => {
                            tracing::warn!("{THE_SURFACE}: refused to open {href}");
                            let _ = a11y
                                .announce(opening_links::THAT_LINK_WAS_NOT_OPENED, Priority::High);
                        }
                    }
                    return;
                }
                Some(page_links::Posted::Back) => {
                    if page.can_go_back() {
                        title_said.set(false);
                        page.go_back();
                    } else {
                        let _ = a11y.announce(THIS_IS_THE_FIRST_PAGE, Priority::Normal);
                    }
                    return;
                }
                None => {}
            }
            // Escape and F6. Closing is what leaving means here: there is
            // one window and nothing behind it, so the direction the page
            // sends is dropped, as it is in the formatted message window.
            if panes::leaving_which_way(&json).is_some() {
                frame.close(false);
            }
        }
    });

    frame.on_close({
        let when_it_closes = when_it_closes.clone();
        move |event| {
            if let WindowEventData::General(ref base) = event {
                base.veto();
            }
            // Hidden rather than destroyed, and then whoever built it
            // decides what closing means. A frame destroyed while its
            // WebView is still hosting an out of process browser takes the
            // application with it, which is what the formatted message
            // window found and does the same thing about.
            frame.show(false);
            when_it_closes();
        }
    });

    frame.set_sizer(sizer, true);

    match what {
        What::APageAt(address) => page.load_url(address),
        What::ThisDocument { html, as_if_from } => page.set_page(html, as_if_from),
    }

    ThePageWindow {
        frame,
        live_region: live_region.get_handle() as isize,
    }
}

/// Which surface this is, for the log, beside "preview" and "conversation
/// window".
const THE_SURFACE: &str = "separate window";

/// A document carrying one sentence, for a page that would not load.
fn a_page_saying(sentence: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><title>{sentence}</title></head>\
         <body><h1>{sentence}</h1></body></html>"
    )
}

/// Show one page in a window of this process's own, and answer with the exit
/// code the process should leave with.
///
/// The whole of a page process. Nothing above this has prepared a data
/// folder, opened a log file, claimed the single-copy marker or read a
/// credential, and nothing here does either.
pub fn show(address: &str) -> i32 {
    let Some(safe) = may_be_followed(address) else {
        crash_log(&format!(
            "{FLAG} was given {address:?}, which is not a page. Nothing was opened."
        ));
        return NOT_A_PAGE;
    };

    let ran = wxdragon::main(move |app| {
        // First, before any browser control exists. WebView2's profile
        // folder comes from `wxStandardPaths::GetUserLocalDataDir`, which
        // appends this name, and it is read when the environment is made,
        // which is when the first control is built. This is the whole
        // isolation a separate window gets (#80).
        app.set_app_name(&paths::page_profile_app_name());
        // And the name a person sees stays the program's, since the one
        // above is a pair of folders.
        app.set_app_display_name("Wixen Mail");

        let a11y = match Accessibility::new() {
            Ok(a11y) => Arc::new(a11y),
            Err(e) => {
                // Said in the log and carried on. A window that shows the
                // page without speaking is worth more than no window, and
                // the person asked for a page.
                tracing::error!("{THE_SURFACE}: no accessibility layer: {e}");
                return;
            }
        };
        let built = build(
            &a11y,
            What::APageAt(&safe),
            Rc::new(move || {
                if let Some(app) = wxdragon::get_app_instance() {
                    app.exit_main_loop();
                }
            }),
        );
        a11y.register_live_region(built.live_region);
        built.put_it_in_front();
    });

    match ran {
        Ok(()) => 0,
        Err(e) => {
            crash_log(&format!("{FLAG} could not open a window: {e}"));
            1
        }
    }
}

/// Write a line to the crash file, which is how a start that must stop says
/// why.
///
/// Its own copy rather than `main`'s, because `main`'s is a binary's private
/// function and this is the library half of the same start.
fn crash_log(message: &str) {
    let folder = crate::common::logging::default_log_dir();
    let _ = std::fs::create_dir_all(&folder);
    let line = format!(
        "[{}] {message}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(folder.join("crash.log"))
        .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::what_ships::what_ships;

    #[test]
    fn test_only_a_page_may_be_followed() {
        // The two schemes a window can show. `mailto:` and `tel:` pass the
        // sanitiser every link in a message goes through, because something
        // else on this computer answers them; neither is a page, and this
        // window is only ever a page.
        assert_eq!(
            may_be_followed("https://example.com/where-it-went"),
            Some("https://example.com/where-it-went".to_string())
        );
        assert_eq!(
            may_be_followed("http://example.com/"),
            Some("http://example.com/".to_string())
        );
        for refused in [
            "mailto:somebody@example.com",
            "tel:+15550100",
            "javascript:alert(1)",
            "file:///C:/Windows/win.ini",
            "not-a-page",
            "",
            "   ",
            "https://somebody@example.com/",
        ] {
            assert_eq!(may_be_followed(refused), None, "{refused} was allowed");
        }
    }

    #[test]
    fn test_the_title_says_the_host_before_the_page_has_one_and_the_page_after() {
        // Before the page arrives there is only the address, and the host is
        // the part of it worth hearing: the rest can be two hundred
        // characters of token.
        assert_eq!(
            the_windows_title(None, "https://example.com/where-it-went?t=1"),
            "Opening example.com - Wixen Mail"
        );
        // And once it has one, the page's title, with this program's name
        // still last so the window says whose it is.
        assert_eq!(
            the_windows_title(Some("Example Domain"), "https://example.com/"),
            "Example Domain - Wixen Mail"
        );
        // A page that titles itself with spaces gets the address back rather
        // than a window called nothing.
        assert_eq!(
            the_windows_title(Some("   "), "https://example.com/"),
            "Opening example.com - Wixen Mail"
        );
        // An address with no host is still a window with a name, and the
        // sentence is the one the message view already says for the same
        // case, so the two surfaces cannot come to name a page differently.
        assert_eq!(
            the_windows_title(None, "https://"),
            "Opening the page - Wixen Mail"
        );
    }

    #[test]
    fn test_an_address_that_is_not_a_page_opens_no_window() {
        // `show` is the whole process: anything it refuses is refused before
        // `wxdragon::main` is called at all, so nothing is on screen and
        // nothing has been claimed. Asserted on the exit code rather than on
        // a window, since a test cannot run two event loops.
        for refused in ["mailto:somebody@example.com", "not-a-page", ""] {
            assert_eq!(
                show(refused),
                NOT_A_PAGE,
                "{refused} did not stop the start"
            );
        }
    }

    /// The text of one function in this module, as a release build sees it.
    ///
    /// From its signature to the first line that closes at the left margin,
    /// which is how every item in this file is written.
    fn the_body_of(ships: &str, signature: &str) -> String {
        let from = ships
            .find(signature)
            .unwrap_or_else(|| panic!("this module should define {signature}"));
        let rest = &ships[from..];
        let to = rest.find("\n}").map(|at| at + 2).unwrap_or(rest.len());
        rest[..to].to_string()
    }

    #[test]
    fn test_the_profile_name_is_set_before_the_browser_is_built() {
        // The whole reason this is a process. The application name decides
        // where WebView2 puts the profile, and it is read when the
        // environment is made, which is when the first control is built. Set
        // afterwards it would name a folder nothing uses, and the page would
        // share the message preview's cookies, which is what #80's third item
        // is about.
        //
        // Read as the order of two calls in `show`, not as the order of two
        // names in the file. The first draft of this read the file and was
        // red on a tree that is right: `build` is defined above `show` and
        // called from inside it, so the browser is written first and made
        // second. What matters is which runs first, and in a file that is
        // one function calling another that is the order inside the caller.
        //
        // Read from the half a release build compiles, so this test's own
        // mention of either name cannot satisfy it.
        let ships = what_ships(include_str!("page_window.rs"));
        let starting = the_body_of(&ships, "pub fn show(");
        let names_the_profile = starting
            .find("set_app_name")
            .expect("a page process names its profile");
        let builds_the_window = starting
            .find("build(")
            .expect("a page process builds its window");

        assert!(
            names_the_profile < builds_the_window,
            "the profile is named after the window is built, so the browser did not get it"
        );
        // And the window is where the browser is made, which is the other
        // half of the same claim and the half a caller cannot see.
        assert!(
            the_body_of(&ships, "pub fn build(").contains("WebView::builder"),
            "the browser is not built in `build`, so the order above is about the wrong pair"
        );
    }
}
