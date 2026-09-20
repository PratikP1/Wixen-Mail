//! Where a link in a message opens, and the decision from the setting and the
//! ask to the route.
//!
//! #80, Pratik's ask of 2026-09-18: a setting decides where a link opens, in
//! the default browser, in the message view, or in a separate Wixen Mail
//! window, and the link's context menu offers all three whatever the setting
//! says. The decision is one pure function here, [`route`], so the page's
//! activation, the three menu items and both surfaces that host a browser ask
//! the same question and get the same answer; the window carries the answer
//! out and nothing else decides.
//!
//! The default is the browser, on purpose. A page opened inside this program
//! runs in the browser profile the message preview uses, so a cookie it sets
//! is sent when a later message loads a picture from the same site, which the
//! browser's own profile would keep to itself; `docs/privacy.md` says so under
//! "Where a link opens". The separate window is 11-11.2's, a process of its
//! own with a profile of its own; until it lands the third choice is offered,
//! routed to the browser and said, never silent.
//!
//! An address that is not a page, `mailto:` or `tel:`, is the system's
//! whatever the setting or the item says: the message view cannot host a mail
//! address, and the browser would only hand it on.

/// Where a link opens, as the Reading tab's choice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Where {
    /// The default browser, which is the default here.
    #[default]
    DefaultBrowser,
    /// The view that held the message, with Backspace bringing the message back.
    MessageView,
    /// A separate Wixen Mail window, a process of its own (11-11.2).
    SeparateWindow,
}

impl Where {
    /// Every choice, in the order the Reading tab offers them.
    pub const ALL: [Where; 3] = [
        Where::DefaultBrowser,
        Where::MessageView,
        Where::SeparateWindow,
    ];

    /// How the setting stores itself.
    pub const fn stored(self) -> &'static str {
        match self {
            Where::DefaultBrowser => "browser",
            Where::MessageView => "message-view",
            Where::SeparateWindow => "separate-window",
        }
    }

    /// Read a stored setting; anything unknown is the browser, the default.
    ///
    /// A settings file written by a later version, or edited by hand, should
    /// not quietly open a stranger's page beside the sanitised mail.
    pub fn from_stored(stored: &str) -> Self {
        match stored.trim().to_ascii_lowercase().as_str() {
            "message-view" => Where::MessageView,
            "separate-window" => Where::SeparateWindow,
            _ => Where::DefaultBrowser,
        }
    }

    /// The words the Reading tab shows for the choice: where, and nothing
    /// about which is better, since the trade is real in both directions.
    pub const fn label(self) -> &'static str {
        match self {
            Where::DefaultBrowser => "In the default browser",
            Where::MessageView => "In the message view",
            Where::SeparateWindow => "In a separate Wixen Mail window",
        }
    }
}

/// Which entry of the offered list a stored choice selects.
pub fn offered_index(stored: &str) -> usize {
    let wanted = Where::from_stored(stored);
    Where::ALL
        .iter()
        .position(|choice| *choice == wanted)
        .unwrap_or(0)
}

/// How a link was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Activated in the page: a click, Enter, or a screen reader's Enter.
    Activated,
    /// The menu's Open in Message View.
    InMessageView,
    /// The menu's Open in Default Browser, or a Ctrl-click.
    InBrowser,
    /// The menu's Open in Separate Window, or a Shift-click.
    InSeparateWindow,
}

impl Asked {
    /// Every way a link is asked for, so a table over the decision covers them.
    pub const ALL: [Asked; 4] = [
        Asked::Activated,
        Asked::InMessageView,
        Asked::InBrowser,
        Asked::InSeparateWindow,
    ];
}

/// Where a link goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Handed to Windows, which opens the default browser.
    Browser,
    /// Loaded in the view that held the message.
    MessageView,
    /// Handed to the separate window's process.
    SeparateWindow,
    /// Handed to Windows for a mail address or a telephone number, which is
    /// not a page and opens whatever answers that kind of address.
    System,
}

impl From<Where> for Route {
    /// The place the setting names, as the route that reaches it.
    fn from(place: Where) -> Self {
        match place {
            Where::DefaultBrowser => Route::Browser,
            Where::MessageView => Route::MessageView,
            Where::SeparateWindow => Route::SeparateWindow,
        }
    }
}

/// Whether an address is a page a browser control can show.
///
/// The two web schemes and nothing else: `mailto:` and `tel:`, which the
/// sanitiser also allows, open whatever answers them and not a page.
pub fn is_a_page(address: &str) -> bool {
    let lower = address.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// The route for a link, from the setting and the way it was asked for.
///
/// Total: every setting, every ask and every kind of address answers, and the
/// table in the tests has a case for each cell. An activation follows the
/// setting; a menu item, or the browser's own modifier, names its own place
/// whatever the setting says; an address that is not a page is the system's.
pub fn route(setting: Where, asked: Asked, address: &str) -> Route {
    if !is_a_page(address) {
        return Route::System;
    }
    match asked {
        Asked::Activated => Route::from(setting),
        Asked::InMessageView => Route::MessageView,
        Asked::InBrowser => Route::Browser,
        Asked::InSeparateWindow => Route::SeparateWindow,
    }
}

/// The label beside the Reading tab's choice.
pub const SETTING_LABEL: &str = "Open &links:";

/// The accessible name of the Reading tab's choice.
pub const SETTING_NAME: &str = "Open links";

/// What the Reading tab says under the choice, so the trade is on the screen.
pub const WHAT_EACH_CHOICE_COSTS: &str = "In the default browser, a page shares nothing with the message preview. In the message \
     view, the page loads where the message was and shares the preview's browser profile; \
     Backspace brings the message back. A separate Wixen Mail window arrives with the next \
     build and opens the browser until then.";

/// The status line when the separate window was asked for before it exists.
pub const SEPARATE_WINDOWS_ARRIVE_LATER: &str =
    "Separate windows arrive with the next build; opened in the browser";

/// What is said when the message view starts loading a page: the host, not
/// the whole address, which can be two hundred characters of token.
pub fn what_is_said_when_opening(address: &str) -> String {
    match host_of(address) {
        Some(host) => format!("Opening {host}"),
        None => "Opening the page".to_string(),
    }
}

/// The host of a web address, for saying where a page comes from.
fn host_of(address: &str) -> Option<&str> {
    let rest = address.trim().split_once("://")?.1;
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    (!host.is_empty()).then_some(host)
}

/// What is said when Backspace or Alt+Left brings the message back.
pub const BACK_TO_THE_MESSAGE: &str = "Back to the message";

/// What is said when a page in the message view will not load.
///
/// `kind` is the browser control's own code for what went wrong, as
/// wxWidgets numbers `wxWebViewNavigationError`, read from the event's int:
/// a reason in words, never the number.
pub fn could_not_be_opened(kind: Option<i32>) -> String {
    let reason = match kind {
        Some(0) => "the connection failed",
        Some(1) => "the site's certificate was not trusted",
        Some(2) => "the site asked for a sign-in",
        Some(3) => "the browser refused it as unsafe",
        Some(4) => "the page was not found",
        Some(5) => "the request was refused",
        Some(6) => "the load was cancelled",
        Some(_) => "the browser could not say why",
        None => "the browser gave no reason",
    };
    format!("The page could not be opened: {reason}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_links_open_in_the_browser_unless_somebody_says_otherwise() {
        // The browser is the default because a page opened inside the program
        // shares the preview's profile; the privacy page says so.
        assert_eq!(Where::default(), Where::DefaultBrowser);
        assert_eq!(Where::from_stored(""), Where::DefaultBrowser);
        assert_eq!(Where::from_stored("somewhere else"), Where::DefaultBrowser);
        assert_eq!(Where::ALL[0], Where::DefaultBrowser);
    }

    #[test]
    fn test_a_stored_choice_reads_back_as_itself() {
        for choice in Where::ALL {
            assert_eq!(Where::from_stored(choice.stored()), choice);
            assert_eq!(
                offered_index(choice.stored()),
                Where::ALL.iter().position(|c| *c == choice).unwrap()
            );
        }
        assert_eq!(Where::from_stored(" Message-View "), Where::MessageView);
        assert_eq!(Where::from_stored("SEPARATE-WINDOW"), Where::SeparateWindow);
    }

    #[test]
    fn test_each_choice_says_where_in_plain_words() {
        assert_eq!(Where::DefaultBrowser.label(), "In the default browser");
        assert_eq!(Where::MessageView.label(), "In the message view");
        assert_eq!(
            Where::SeparateWindow.label(),
            "In a separate Wixen Mail window"
        );
        for choice in Where::ALL {
            assert!(
                !choice.label().contains("ecommended"),
                "{choice:?} sells itself rather than saying where"
            );
        }
    }

    #[test]
    fn test_a_page_is_an_http_or_https_address_and_nothing_else_is() {
        assert!(is_a_page("https://example.com/where-it-went"));
        assert!(is_a_page("http://example.com/"));
        assert!(is_a_page("HTTPS://EXAMPLE.COM/"));
        for not_a_page in ["mailto:ada@example.org", "tel:+15550100", "", "example.com"] {
            assert!(!is_a_page(not_a_page), "{not_a_page:?} is not a page");
        }
    }

    #[test]
    fn test_an_activated_link_follows_the_setting() {
        let page = "https://example.com/where-it-went";
        assert_eq!(
            route(Where::DefaultBrowser, Asked::Activated, page),
            Route::Browser
        );
        assert_eq!(
            route(Where::MessageView, Asked::Activated, page),
            Route::MessageView
        );
        assert_eq!(
            route(Where::SeparateWindow, Asked::Activated, page),
            Route::SeparateWindow
        );
    }

    #[test]
    fn test_a_menu_item_names_its_own_place_whatever_the_setting_says() {
        let page = "https://example.com/where-it-went";
        for setting in Where::ALL {
            assert_eq!(
                route(setting, Asked::InMessageView, page),
                Route::MessageView,
                "under {setting:?}"
            );
            assert_eq!(
                route(setting, Asked::InBrowser, page),
                Route::Browser,
                "under {setting:?}"
            );
            assert_eq!(
                route(setting, Asked::InSeparateWindow, page),
                Route::SeparateWindow,
                "under {setting:?}"
            );
        }
    }

    #[test]
    fn test_a_mail_address_or_a_telephone_number_is_the_systems_whatever_was_asked() {
        // The message view cannot host a mail address, and the browser would
        // only hand it on: so it goes straight to whatever answers it.
        for setting in Where::ALL {
            for asked in Asked::ALL {
                for address in ["mailto:ada@example.org", "tel:+15550100"] {
                    assert_eq!(
                        route(setting, asked, address),
                        Route::System,
                        "{address} under {setting:?} asked {asked:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_every_cell_of_the_table_answers() {
        // Total: three settings, four asks, a page and a non-page, and no
        // cell reaches a fallback that was never chosen.
        let mut cells = 0;
        for setting in Where::ALL {
            for asked in Asked::ALL {
                for address in ["https://example.com/", "mailto:ada@example.org"] {
                    let answer = route(setting, asked, address);
                    let expected = if !is_a_page(address) {
                        Route::System
                    } else {
                        match asked {
                            Asked::Activated => Route::from(setting),
                            Asked::InMessageView => Route::MessageView,
                            Asked::InBrowser => Route::Browser,
                            Asked::InSeparateWindow => Route::SeparateWindow,
                        }
                    };
                    assert_eq!(answer, expected, "{setting:?} {asked:?} {address}");
                    cells += 1;
                }
            }
        }
        assert_eq!(cells, 24);
    }

    #[test]
    fn test_opening_says_the_pages_host_and_not_its_whole_address() {
        // The address can be two hundred characters of token; the host is
        // what a person wants to hear before the page arrives.
        assert_eq!(
            what_is_said_when_opening("https://example.com/where-it-went?token=abc"),
            "Opening example.com"
        );
        assert_eq!(
            what_is_said_when_opening("http://news.example.org"),
            "Opening news.example.org"
        );
        assert_eq!(what_is_said_when_opening("nonsense"), "Opening the page");
    }

    #[test]
    fn test_a_page_that_will_not_load_says_what_went_wrong_in_words() {
        // wxWidgets numbers the kinds; the person hears a reason, not a number.
        assert_eq!(
            could_not_be_opened(Some(0)),
            "The page could not be opened: the connection failed"
        );
        assert_eq!(
            could_not_be_opened(Some(1)),
            "The page could not be opened: the site's certificate was not trusted"
        );
        assert_eq!(
            could_not_be_opened(Some(4)),
            "The page could not be opened: the page was not found"
        );
        assert_eq!(
            could_not_be_opened(None),
            "The page could not be opened: the browser gave no reason"
        );
        for kind in 0..8 {
            let said = could_not_be_opened(Some(kind));
            assert!(said.starts_with("The page could not be opened: "), "{said}");
            assert!(!said.contains(char::is_numeric), "{said} says a number");
        }
    }

    #[test]
    fn test_the_words_on_the_reading_tab_name_the_trade_and_the_wait() {
        assert!(WHAT_EACH_CHOICE_COSTS.contains("profile"));
        assert!(WHAT_EACH_CHOICE_COSTS.contains("Backspace"));
        assert!(WHAT_EACH_CHOICE_COSTS.contains("next build"));
        assert!(SEPARATE_WINDOWS_ARRIVE_LATER.contains("browser"));
        assert_eq!(
            SETTING_LABEL.replace('&', "").trim_end_matches(':'),
            SETTING_NAME
        );
        assert_eq!(BACK_TO_THE_MESSAGE, "Back to the message");
    }
}
