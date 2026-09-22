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
    let _ = address;
    None
}

/// The window's title.
///
/// The host first, so a page cannot put its own words where the program's
/// name goes, and the program's name last, so a window in the task switcher
/// says whose it is (T-12-08).
pub fn the_windows_title(page_title: Option<&str>, address: &str) -> String {
    let _ = (page_title, address);
    String::new()
}

/// Show one page in a window of this process's own, and answer with the exit
/// code the process should leave with.
pub fn show(address: &str) -> i32 {
    let _ = address;
    0
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
        // An address with no host is still a window with a name.
        assert_eq!(
            the_windows_title(None, "https://"),
            "Opening a page - Wixen Mail"
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

    #[test]
    fn test_the_profile_name_is_set_before_the_browser_is_built() {
        // The whole reason this is a process. The application name decides
        // where WebView2 puts the profile, and it is read when the
        // environment is made, which is when the first control is built. Set
        // afterwards it would name a folder nothing uses, and the page would
        // share the message preview's cookies, which is what #80's third item
        // is about.
        //
        // Read from the half a release build compiles, so this test's own
        // mention of either name cannot satisfy it.
        let ships = what_ships(include_str!("page_window.rs"));
        let names_the_profile = ships
            .find("set_app_name")
            .expect("the page process names its profile");
        let builds_the_browser = ships
            .find("WebView::builder")
            .expect("the page process builds a browser");

        assert!(
            names_the_profile < builds_the_browser,
            "the profile is named after the browser is built, so the browser did not get it"
        );
    }
}
