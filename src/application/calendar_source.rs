//! Adding a calendar by the address it lives at.
//!
//! Two kinds. A calendar server holds a calendar that can be read and, one day,
//! written; a feed is a file somebody publishes that this application re-reads
//! and never writes to. Both are added the same way and are told apart by the
//! word written into the calendar's row.
//!
//! Everything a screen would have to decide is here rather than in the window,
//! because a window needs a display and a running application and nothing about
//! it can be tested. What somebody typed becomes an address, an address is
//! usable or it is not, a server's answer becomes a list somebody can choose
//! from, and a failure becomes a sentence. The window is left with the widgets.
//!
//! # Nothing here changes anything at a provider
//!
//! Discovery asks a server what it has. It goes out through
//! [`CalDavClient::new`], which holds the refusing client, so a request that
//! changed something would be stopped before the network even if somebody
//! later moved discovery onto a writing method. The two things this does write
//! are on this computer: a row in the calendars table, and the sign-in in the
//! credential store.

use crate::common::Error;
use crate::data::message_cache::{CalendarContainer, MessageCache};
use crate::service::caldav::{CalDavCalendar, CalDavClient, sign_in};

/// The word written into a calendar's row for one held on a calendar server.
///
/// The sync loop and the uninstall both look for this exact word, and nothing
/// in the application wrote it until this screen existed, which is why both
/// were unreachable. A test below reads their source and fails if either stops
/// looking for it.
pub const ON_A_SERVER: &str = "caldav";

/// The word written into a calendar's row for one read from a published feed.
pub const FROM_A_FEED: &str = "subscription";

/// What the window says about itself, where somebody adding a calendar reads it.
///
/// Not in a report and not in a log line. This is the first way in the product
/// to reach a calendar server at all, nobody has pointed it at a real one, and
/// a change made to one of these calendars is not sent anywhere yet. Somebody
/// deciding whether to trust it with their calendar should be told all three
/// before they do, not after.
pub const NOT_TRIED_FOR_REAL: &str = "This is experimental. Nothing here has been tried against \
                                      a real calendar server yet, so expect problems. Changes \
                                      you make to a calendar added this way stay on this \
                                      computer: they are not sent back to the server.";

/// What a calendar this program added for somebody is coloured.
///
/// The same blue every other calendar made here gets. Colour is decoration:
/// nothing is told apart by it alone, and a colour picker in this dialog is a
/// control most people would tab past.
const A_CALENDAR_NOBODY_CHOSE_A_COLOUR_FOR: &str = "#4285F4";

/// What a feed with no name given is called.
const A_FEED_NOBODY_NAMED: &str = "Calendar feed";

/// The two kinds of address this screen takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A calendar server, which is signed in to.
    Server,
    /// A published feed, which is only ever read.
    Feed,
}

/// Why an address cannot be used, in a form that becomes a sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unusable {
    /// Nothing was typed.
    Nothing,
    /// It could not be read as an address at all.
    NotAnAddress,
    /// It names something that is not the web.
    NotTheWeb,
    /// It carries a user name, and maybe a password, inside the address.
    CarriesASignIn,
    /// It holds a space, a tab or a character that cannot be in an address.
    HasStrayCharacters,
    /// A sign-in would go over the wire in clear.
    NotEncrypted,
}

impl Unusable {
    /// Every reason, so a test can walk them and none can be left without
    /// words.
    pub const ALL: [Unusable; 6] = [
        Unusable::Nothing,
        Unusable::NotAnAddress,
        Unusable::NotTheWeb,
        Unusable::CarriesASignIn,
        Unusable::HasStrayCharacters,
        Unusable::NotEncrypted,
    ];

    /// What somebody is told: what happened, then what to do next.
    pub const fn said(self) -> &'static str {
        match self {
            Unusable::Nothing => {
                "Type the address of the calendar. It usually starts with www or with a name \
                 like cal followed by a dot."
            }
            Unusable::NotAnAddress => {
                "That is not an address a calendar can be read from. Check it for a typing \
                 mistake and try again."
            }
            Unusable::NotTheWeb => {
                "A calendar can only be added from the web. Paste the address from your \
                 calendar provider's own page."
            }
            Unusable::CarriesASignIn => {
                "That address has a user name written into it. Take the part before the at \
                 sign out, and type the user name and password in the boxes below."
            }
            Unusable::HasStrayCharacters => {
                "That address has a space or a line break in it. It was probably picked up \
                 when the address was copied, so take it out and try again."
            }
            Unusable::NotEncrypted => {
                "That address is not a secure one, so the password would travel where anyone \
                 on the network could read it. Ask whoever looks after the server for a \
                 secure address, or add the calendar as a feed instead, which needs no \
                 password."
            }
        }
    }
}

/// What somebody typed, turned into an address that can be asked, or a reason
/// it cannot.
///
/// This is the boundary. The string was typed by a person, or pasted from
/// wherever they found it, and the next thing that happens to it is a network
/// request, so it is checked here and nowhere else has to wonder.
pub fn address_for(typed: &str, source: Source) -> std::result::Result<String, Unusable> {
    let typed = typed.trim();
    if typed.is_empty() {
        return Err(Unusable::Nothing);
    }
    // Before parsing rather than after. The address parser strips a tab and a
    // line break silently, so what would be stored is not what was checked.
    if typed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(Unusable::HasStrayCharacters);
    }

    // Every provider's subscribe button hands out webcal, which nothing can
    // fetch. It means "this address, over the web", so that is what it becomes.
    let written = match typed.strip_prefix("webcal://") {
        Some(rest) => format!("https://{rest}"),
        // No scheme typed at all. The secure one, because a person typing a
        // bare server name is not asking for their password in clear.
        None if !typed.contains("://") => format!("https://{typed}"),
        None => typed.to_string(),
    };

    let address = url::Url::parse(&written).map_err(|_| Unusable::NotAnAddress)?;
    if !matches!(address.scheme(), "http" | "https") {
        return Err(Unusable::NotTheWeb);
    }
    if !address.username().is_empty() || address.password().is_some() {
        return Err(Unusable::CarriesASignIn);
    }
    if source == Source::Server && address.scheme() == "http" && !is_on_this_computer(&address) {
        return Err(Unusable::NotEncrypted);
    }
    Ok(address.to_string())
}

/// Whether an address names this computer, where the bytes never leave the
/// machine.
///
/// The same rule a browser uses to call a page secure without encryption, and
/// it is what lets a test exercise the real path against a loopback server
/// rather than a stand-in.
fn is_on_this_computer(address: &url::Url) -> bool {
    match address.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(name)) => {
            name.eq_ignore_ascii_case("localhost")
                || name.to_ascii_lowercase().ends_with(".localhost")
        }
        None => false,
    }
}

/// One line somebody can choose from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// What is read out. Distinguishable from its neighbours, always.
    pub name: String,
    /// The address the server itself gave, kept as it came.
    pub address: String,
}

/// What a server's answer becomes: the calendars it really has, each with
/// something a person can tell apart from the one next to it.
///
/// A picker read aloud is only its lines. Two calendars called Work, which one
/// account holding one of somebody's own and one shared to them ordinarily has,
/// are two identical lines and nobody knows which they chose. So a name shared
/// with another line has the calendar's own address added to it.
pub fn offered(found: &[CalDavCalendar]) -> Vec<Offer> {
    let named: Vec<Offer> = found
        .iter()
        .filter(|calendar| !calendar.url.trim().is_empty())
        .map(|calendar| Offer {
            name: name_for(calendar),
            address: calendar.url.clone(),
        })
        .collect();

    named
        .iter()
        .map(|offer| {
            let shared = named
                .iter()
                .filter(|other| other.name == offer.name)
                .count()
                > 1;
            Offer {
                name: if shared {
                    format!("{} at {}", offer.name, offer.address)
                } else {
                    offer.name.clone()
                },
                address: offer.address.clone(),
            }
        })
        .collect()
}

/// What one calendar is called before its neighbours are considered.
///
/// A server that named it wins. One it did not is given the last part of its
/// own address, which is what tells two unnamed calendars apart, because the
/// reader gives every one of them the same word.
fn name_for(calendar: &CalDavCalendar) -> String {
    let given = calendar.display_name.trim();
    if !given.is_empty() && given != "Untitled" {
        return given.to_string();
    }
    let last_part = calendar
        .url
        .trim_end_matches('/')
        .rsplit('/')
        .find(|part| !part.is_empty());
    match last_part {
        Some(part) => format!("Calendar at {part}"),
        None => "Calendar".to_string(),
    }
}

/// What somebody is told when a calendar server could not be added.
///
/// One sentence per case, each saying what happened and what to do next. Never
/// the words underneath: a transport failure read back at somebody tells them
/// nothing they can act on and quite a lot they did not ask about.
pub fn what_went_wrong(failure: &Error) -> String {
    match failure {
        Error::Api { status: 401, .. } | Error::Api { status: 403, .. } => {
            "The calendar server did not accept that user name and password. Check both and \
             try again."
        }
        Error::Api { status: 404, .. } => {
            "There is no calendar at that address. Check the address with whoever gave it to \
             you and try again."
        }
        Error::Api { .. } => {
            "The calendar server could not answer just now. Try again in a few minutes, and \
             if it keeps happening ask whoever looks after the server."
        }
        Error::Network(_) => {
            "That address could not be reached. Check the address and your connection, then \
             try again."
        }
        _ => {
            "The calendar could not be added. Check the address and the sign-in details, then \
             try again."
        }
    }
    .to_string()
}

/// What somebody is told when a server answered and had nothing to offer.
const NOTHING_OFFERED: &str = "That server has no calendars on it for this sign-in. Check the \
                               address, or sign in as somebody a calendar is shared with.";

/// What somebody is told when a calendar would be filed where nothing visits.
const NO_ACCOUNT_TO_FILE_IT_UNDER: &str = "Set up an account first. A calendar is kept with an \
                                           account, and only an account's calendars are \
                                           brought up to date.";

/// The row a hand-added calendar becomes.
///
/// The id has no server's name in it and no colon, which is the rule the
/// calendars table's own notes set out: a server's list names a calendar
/// `google:` or `ms:`, everything made here is left alone by reconciliation. An
/// id shaped like a server's would be a calendar deleted out from under
/// somebody the first time the calendars are reconciled.
pub fn row_for(account_id: &str, source: Source, name: &str, address: &str) -> CalendarContainer {
    let stamp = chrono::Utc::now().to_rfc3339();
    let name = match name.trim() {
        "" => A_FEED_NOBODY_NAMED.to_string(),
        given => given.to_string(),
    };
    CalendarContainer {
        id: an_id_no_server_would_have_given(),
        account_id: account_id.to_string(),
        name,
        color: A_CALENDAR_NOBODY_CHOSE_A_COLOUR_FOR.to_string(),
        source_provider: Some(
            match source {
                Source::Server => ON_A_SERVER,
                Source::Feed => FROM_A_FEED,
            }
            .to_string(),
        ),
        caldav_url: (source == Source::Server).then(|| address.to_string()),
        subscription_url: (source == Source::Feed).then(|| address.to_string()),
        // Never the default. That is where every unfiled event lands, and
        // moving it moves somebody's whole calendar.
        is_default: false,
        is_visible: true,
        // A feed is a file somebody else publishes. Nothing here can change it,
        // and saying so is what stops a later write path trying.
        is_read_only: source == Source::Feed,
        display_order: 0,
        etag: None,
        ctag: None,
        sync_token: None,
        refresh_interval_minutes: None,
        created_at: stamp.clone(),
        updated_at: stamp,
    }
}

/// An identifier for a calendar this program made.
///
/// Time based rather than random, so calendars added in one session sort in the
/// order they were made. No colon in it, which is what says a server did not
/// name this calendar.
fn an_id_no_server_would_have_given() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    format!("calendar-{nanos}")
}

/// Add a calendar held on a calendar server: ask what it has, let somebody
/// choose, keep the sign-in where passwords go, and write the row.
///
/// `choosing` is handed the calendars the server offered and answers with the
/// one somebody picked, or nothing if they changed their mind. It is a closure
/// so the window can supply a picker and this can be run without one.
///
/// `Ok(None)` is somebody changing their mind, which is not a failure. `Err` is
/// one sentence saying what happened and what to do next.
pub async fn add_from_a_server(
    cache: &MessageCache,
    account_id: &str,
    typed: &str,
    user_name: &str,
    password: &str,
    choosing: impl FnOnce(&[Offer]) -> Option<usize>,
) -> std::result::Result<Option<CalendarContainer>, String> {
    if account_id == crate::application::new_item::LOCAL_ACCOUNT_ID {
        return Err(NO_ACCOUNT_TO_FILE_IT_UNDER.to_string());
    }
    // Before anything is sent. A refusal after the request is a password
    // already on the wire.
    let address = address_for(typed, Source::Server).map_err(|why| why.said().to_string())?;

    // The refusing client, not the one built from what an account is allowed.
    // Asking a server what it has changes nothing, and building it this way
    // means a later edit that moved discovery onto a changing method would be
    // stopped here rather than at somebody's calendar.
    let found = CalDavClient::new()
        .discover_calendars(&address, user_name, password)
        .await
        .map_err(|failure| what_went_wrong(&failure))?;

    let offers = offered(&found);
    if offers.is_empty() {
        return Err(NOTHING_OFFERED.to_string());
    }
    let Some(chosen) = choosing(&offers).and_then(|which| offers.get(which)) else {
        return Ok(None);
    };

    let row = row_for(account_id, Source::Server, &chosen.name, &chosen.address);
    // The sign-in first, so a calendar never exists with no way to reach it.
    // If the row cannot be written the sign-in is taken back out again. The
    // credential store under test cannot be made to fail, so that second half
    // is reasoned rather than covered by a test.
    sign_in::store(&row.id, user_name, password)
        .map_err(|_| "The sign-in could not be saved on this computer. Try again.".to_string())?;
    if let Err(failure) = cache.save_calendar(&row) {
        let _ = sign_in::forget(&row.id);
        tracing::error!("A calendar added by address could not be saved: {failure}");
        return Err("The calendar could not be saved on this computer. Try again.".to_string());
    }
    Ok(Some(row))
}

/// Add a calendar read from a published feed.
///
/// Nothing is asked of the server here. A feed is fetched whole by the refresh,
/// and asking for it twice would download somebody's whole year to find out
/// whether the address works.
pub fn add_from_a_feed(
    cache: &MessageCache,
    account_id: &str,
    typed: &str,
    name: &str,
) -> std::result::Result<CalendarContainer, String> {
    if account_id == crate::application::new_item::LOCAL_ACCOUNT_ID {
        return Err(NO_ACCOUNT_TO_FILE_IT_UNDER.to_string());
    }
    let address = address_for(typed, Source::Feed).map_err(|why| why.said().to_string())?;

    let row = row_for(account_id, Source::Feed, name, &address);
    if let Err(failure) = cache.save_calendar(&row) {
        tracing::error!("A feed added by address could not be saved: {failure}");
        return Err("The calendar could not be saved on this computer. Try again.".to_string());
    }
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::MessageCache;
    use crate::service::caldav::{CalDavCalendar, sign_in};

    // ── Turning what somebody typed into an address ──────────────────────

    #[test]
    fn test_an_address_with_no_scheme_is_read_as_a_secure_one() {
        assert_eq!(
            address_for("cal.example.com/dav/", Source::Server),
            Ok("https://cal.example.com/dav/".to_string())
        );
    }

    #[test]
    fn test_a_calendar_link_written_the_way_providers_write_them_becomes_one_that_can_be_fetched() {
        // Every provider's "subscribe" button hands out webcal://. Nothing can
        // fetch that, and it is the single most likely thing to be pasted here.
        assert_eq!(
            address_for("webcal://cal.example.com/holidays.ics", Source::Feed),
            Ok("https://cal.example.com/holidays.ics".to_string())
        );
    }

    #[test]
    fn test_a_sign_in_is_never_sent_over_an_unencrypted_connection() {
        // This screen is the first thing in the product that asks for a
        // password. Sent to an http address it goes over the wire in clear.
        assert_eq!(
            address_for("http://cal.example.com/dav/", Source::Server),
            Err(Unusable::NotEncrypted)
        );
    }

    #[test]
    fn test_a_server_on_this_computer_may_be_reached_without_encryption() {
        // The bytes never leave the machine, which is the rule browsers use
        // for a secure context, and it is what lets the tests below exercise
        // the real path rather than a stand-in.
        for typed in [
            "http://127.0.0.1:8008/dav/",
            "http://localhost:8008/dav/",
            "http://[::1]:8008/dav/",
        ] {
            assert!(
                address_for(typed, Source::Server).is_ok(),
                "{typed} was refused"
            );
        }
    }

    #[test]
    fn test_a_feed_may_be_unencrypted_because_it_carries_no_sign_in() {
        // An ordinary holiday feed. There is no password to lose, and refusing
        // these would refuse most of what this half of the screen is for.
        assert_eq!(
            address_for("http://example.com/holidays.ics", Source::Feed),
            Ok("http://example.com/holidays.ics".to_string())
        );
    }

    #[test]
    fn test_an_address_that_hides_a_user_name_inside_it_is_refused() {
        // A password written into the address is a password in a log line and
        // in the database's address column. It has somewhere of its own here.
        assert_eq!(
            address_for("https://sam:secret@cal.example.com/dav/", Source::Server),
            Err(Unusable::CarriesASignIn)
        );
    }

    #[test]
    fn test_an_address_that_names_no_server_is_refused() {
        // There is no separate reason for this. A web address with nothing
        // where the server goes cannot be read as an address at all, so it is
        // refused as one, and a reason nothing could ever produce would be a
        // sentence nobody hears.
        assert_eq!(
            address_for("https://", Source::Server),
            Err(Unusable::NotAnAddress)
        );
    }

    #[test]
    fn test_a_scheme_that_is_not_the_web_is_refused() {
        for typed in ["file:///c:/calendar.ics", "ftp://example.com/c.ics"] {
            assert_eq!(
                address_for(typed, Source::Feed),
                Err(Unusable::NotTheWeb),
                "{typed}"
            );
        }
        // Written with no slashes, so there is no scheme to read and it is
        // refused as an address rather than as a scheme. Either way it never
        // reaches the network.
        assert!(address_for("javascript:alert(1)", Source::Feed).is_err());
    }

    #[test]
    fn test_a_blank_address_is_refused_with_a_sentence_saying_what_to_type() {
        assert_eq!(address_for("   ", Source::Server), Err(Unusable::Nothing));
        assert!(Unusable::Nothing.said().contains("address"));
    }

    #[test]
    fn test_an_address_with_control_characters_in_it_is_refused() {
        // A pasted address can carry a newline or a tab. The address parser
        // strips those quietly, so what gets stored is not what was checked.
        assert_eq!(
            address_for("https://cal.example.com/\ndav/", Source::Server),
            Err(Unusable::HasStrayCharacters)
        );
        assert_eq!(
            address_for("https://cal.example.com/da v/", Source::Server),
            Err(Unusable::HasStrayCharacters)
        );
    }

    #[test]
    fn test_every_refusal_says_what_happened_and_what_to_do() {
        for refusal in Unusable::ALL {
            let said = refusal.said();
            assert!(!said.is_empty(), "{refusal:?} says nothing");
            assert!(said.ends_with('.'), "{refusal:?}: {said}");
            for jargon in ["URL", "URI", "scheme", "http", "parse", "invalid"] {
                assert!(
                    !said.contains(jargon),
                    "{refusal:?} says {jargon} at somebody: {said}"
                );
            }
        }
    }

    #[test]
    fn test_reading_an_address_never_panics_whatever_was_typed() {
        // Typed by a person and then used to make a network request, so this
        // is the boundary. Deterministic, so a failure comes back from a seed.
        for seed in 0..5000u64 {
            let typed = fuzz_typing(seed);
            let _ = address_for(&typed, Source::Server);
            let _ = address_for(&typed, Source::Feed);
        }
    }

    /// Something a person could plausibly paste, and a lot they could not.
    fn fuzz_typing(seed: u64) -> String {
        let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as usize
        };
        let pieces = [
            "https://",
            "webcal://",
            "http://",
            ":",
            "//",
            "/",
            "@",
            "?",
            "#",
            "..",
            "%",
            "cal.example.com",
            "sam",
            "секрет",
            "日本",
            "\u{0}",
            "\n",
            "\t",
            " ",
            "[",
            "]",
            "\u{FFFD}",
            "8008",
            ".ics",
        ];
        let count = next() % 12;
        (0..count).map(|_| pieces[next() % pieces.len()]).collect()
    }

    #[test]
    fn test_the_screen_says_it_is_experimental_where_somebody_will_read_it() {
        // A warning that lives in a report is a warning nobody gets. This one
        // is on the window itself, and it has to say what has not been tried
        // and what does not happen yet, not only that it is early.
        assert!(
            NOT_TRIED_FOR_REAL.contains("experimental"),
            "{NOT_TRIED_FOR_REAL}"
        );
        assert!(
            NOT_TRIED_FOR_REAL.contains("real calendar server"),
            "{NOT_TRIED_FOR_REAL}"
        );
        assert!(
            NOT_TRIED_FOR_REAL.contains("stay on this computer"),
            "{NOT_TRIED_FOR_REAL}"
        );
        // Read aloud, so a wrapped literal that lost its continuations is
        // runs of silence in the middle of the sentence.
        assert!(!NOT_TRIED_FOR_REAL.contains("  "), "{NOT_TRIED_FOR_REAL}");
    }

    // ── What a discovered list becomes ───────────────────────────────────

    fn discovered(name: &str, url: &str) -> CalDavCalendar {
        CalDavCalendar {
            url: url.to_string(),
            display_name: name.to_string(),
            color: None,
            ctag: None,
            description: None,
        }
    }

    #[test]
    fn test_a_calendar_keeps_the_name_the_server_gave_it() {
        let offers = offered(&[discovered("Work", "https://cal.example.com/dav/work/")]);

        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].name, "Work");
    }

    #[test]
    fn test_the_address_offered_is_the_one_the_server_gave() {
        // Never rebuilt from what was typed. The server's own address is the
        // one that answers, and a calendar can live somewhere other than under
        // the address somebody typed to find it.
        let offers = offered(&[discovered("Work", "https://elsewhere.example.com/c/17/")]);

        assert_eq!(offers[0].address, "https://elsewhere.example.com/c/17/");
    }

    #[test]
    fn test_two_calendars_with_the_same_name_are_still_told_apart() {
        // One account holding two calendars called Work is ordinary: one of
        // somebody's own and one shared to them. A picker that speaks the same
        // words for both is a picker where nobody knows which they chose, and
        // the only thing a screen reader says is the line.
        let offers = offered(&[
            discovered("Work", "https://cal.example.com/dav/sam/work/"),
            discovered("Work", "https://cal.example.com/dav/team/work/"),
        ]);

        assert_eq!(offers.len(), 2);
        assert_ne!(offers[0].name, offers[1].name, "{offers:?}");
    }

    #[test]
    fn test_two_calendars_the_server_did_not_name_are_still_told_apart() {
        // The reader gives an unnamed calendar the word "Untitled", so two of
        // them arrive identical before anything here has a chance to help.
        let offers = offered(&[
            discovered("Untitled", "https://cal.example.com/dav/one/"),
            discovered("Untitled", "https://cal.example.com/dav/two/"),
        ]);

        assert_eq!(offers.len(), 2);
        assert_ne!(offers[0].name, offers[1].name, "{offers:?}");
        assert!(offers[0].name.contains("one"), "{offers:?}");
    }

    #[test]
    fn test_a_calendar_with_no_address_is_not_offered() {
        // Nothing can be synced from an empty address, and a line somebody can
        // pick that then does nothing is worse than a line that is not there.
        assert!(offered(&[discovered("Work", "")]).is_empty());
    }

    // ── What to say when it fails ────────────────────────────────────────

    fn api(status: u16) -> crate::common::Error {
        crate::common::Error::Api {
            status,
            provider: "calendar server".to_string(),
            message: format!("PROPFIND returned {status}"),
        }
    }

    #[test]
    fn test_a_sign_in_the_server_would_not_accept_says_so_and_not_that_the_network_failed() {
        for status in [401, 403] {
            let said = what_went_wrong(&api(status));
            assert!(said.contains("user name"), "{status}: {said}");
            assert!(said.contains("password"), "{status}: {said}");
        }
    }

    #[test]
    fn test_nothing_at_that_address_says_to_check_the_address() {
        let said = what_went_wrong(&api(404));

        assert!(said.contains("no calendar at that address"), "{said}");
    }

    #[test]
    fn test_a_server_that_could_not_be_reached_is_told_apart_from_one_that_answered() {
        let said = what_went_wrong(&crate::common::Error::Network("dns".to_string()));

        assert!(said.contains("could not be reached"), "{said}");
        assert!(!said.contains("user name"), "{said}");
    }

    #[test]
    fn test_any_other_answer_still_gets_a_sentence() {
        let said = what_went_wrong(&api(500));

        assert!(!said.is_empty());
        assert!(said.ends_with('.'), "{said}");
        // And an answer nobody wrote an arm for is still a sentence.
        let other = what_went_wrong(&crate::common::Error::Other("boom".to_string()));
        assert!(other.ends_with('.'), "{other}");
    }

    #[test]
    fn test_no_message_reads_a_transport_error_back_at_somebody() {
        let all = [
            what_went_wrong(&api(401)),
            what_went_wrong(&api(403)),
            what_went_wrong(&api(404)),
            what_went_wrong(&api(500)),
            what_went_wrong(&crate::common::Error::Network("dns".to_string())),
            what_went_wrong(&crate::common::Error::Other("boom".to_string())),
        ];

        for said in all {
            for machinery in ["PROPFIND", "reqwest", "CalDAV", "Error", "dns", "boom"] {
                assert!(!said.contains(machinery), "{machinery} in: {said}");
            }
        }
    }

    // ── The row a hand-added calendar becomes ────────────────────────────

    #[test]
    fn test_a_calendar_added_by_hand_is_not_given_an_identifier_a_provider_would_have_given_it() {
        // The rule the calendars unit wrote down: a server's own list names a
        // calendar google: or ms:, everything made here has no server's name
        // in it and no colon. A reconciliation removes what the server does
        // not have, so an id shaped like a server's is a calendar deleted out
        // from under somebody.
        let row = row_for(
            "acc-1",
            Source::Server,
            "Work",
            "https://cal.example.com/dav/work/",
        );

        assert!(!row.id.contains(':'), "{}", row.id);
        assert!(!row.id.starts_with("google"), "{}", row.id);
        assert!(!row.id.starts_with("ms"), "{}", row.id);
    }

    #[test]
    fn test_two_calendars_added_one_after_the_other_do_not_share_an_identifier() {
        let one = row_for("acc-1", Source::Feed, "A", "https://example.com/a.ics");
        let two = row_for("acc-1", Source::Feed, "B", "https://example.com/b.ics");

        assert_ne!(one.id, two.id);
    }

    #[test]
    fn test_a_calendar_on_a_server_is_filed_as_one_the_sync_will_visit() {
        let row = row_for(
            "acc-1",
            Source::Server,
            "Work",
            "https://cal.example.com/dav/work/",
        );

        assert_eq!(row.source_provider.as_deref(), Some(ON_A_SERVER));
        assert_eq!(
            row.caldav_url.as_deref(),
            Some("https://cal.example.com/dav/work/")
        );
        assert_eq!(row.subscription_url, None);
        assert!(!row.is_read_only);
    }

    #[test]
    fn test_a_feed_is_filed_as_one_the_refresh_will_visit_and_nobody_can_write_to() {
        let row = row_for(
            "acc-1",
            Source::Feed,
            "Holidays",
            "https://example.com/holidays.ics",
        );

        assert_eq!(row.source_provider.as_deref(), Some(FROM_A_FEED));
        assert_eq!(
            row.subscription_url.as_deref(),
            Some("https://example.com/holidays.ics")
        );
        assert_eq!(row.caldav_url, None);
        assert!(row.is_read_only, "a feed cannot be written to");
    }

    #[test]
    fn test_a_new_calendar_is_never_the_account_default_and_can_be_seen() {
        let row = row_for(
            "acc-1",
            Source::Feed,
            "Holidays",
            "https://example.com/h.ics",
        );

        // The default is where every unfiled event lands, and moving it would
        // move somebody's whole calendar.
        assert!(!row.is_default);
        assert!(
            row.is_visible,
            "a calendar nobody can see is a calendar nobody added"
        );
    }

    #[test]
    fn test_both_stamps_are_filled_and_agree() {
        // The upsert writes whatever is here straight through, so a blank
        // stamp becomes a row with no date on it and nothing says so.
        let row = row_for(
            "acc-1",
            Source::Feed,
            "Holidays",
            "https://example.com/h.ics",
        );

        assert!(!row.created_at.is_empty());
        assert_eq!(row.created_at, row.updated_at);
        assert!(
            chrono::DateTime::parse_from_rfc3339(&row.created_at).is_ok(),
            "{}",
            row.created_at
        );
    }

    #[test]
    fn test_a_calendar_added_by_hand_keeps_the_name_it_was_given() {
        let row = row_for(
            "acc-1",
            Source::Feed,
            "Term dates",
            "https://example.com/t.ics",
        );

        assert_eq!(row.name, "Term dates");
        assert_eq!(row.account_id, "acc-1");
        assert!(
            !row.color.is_empty(),
            "a row with no colour is a blank swatch"
        );
    }

    #[test]
    fn test_the_words_written_here_are_the_words_the_sync_and_the_uninstall_look_for() {
        // The whole point of this unit. Both filters were unreachable because
        // nothing in the application ever wrote either word, and renaming one
        // here without the other would put them back that way in silence.
        for (path, word) in [
            ("src/presentation/wx_app.rs", "calendar_source::ON_A_SERVER"),
            ("src/presentation/wx_app.rs", "calendar_source::FROM_A_FEED"),
            ("src/application/forget.rs", "calendar_source::ON_A_SERVER"),
        ] {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(
                source.contains(word),
                "{path} does not look for {word}, so what this screen writes is never visited"
            );
        }
        // And the words themselves are still the ones already in every
        // database, because changing either strands every calendar that was
        // added before the change.
        assert_eq!(ON_A_SERVER, "caldav");
        assert_eq!(FROM_A_FEED, "subscription");
    }

    // ── The whole act of adding one ──────────────────────────────────────

    use crate::common::answering::{answering, heard};

    fn temp_cache(label: &str) -> MessageCache {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock set later than 1970")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wixen_add_cal_{label}_{nanos}"));
        MessageCache::new(dir, None).expect("a cache in a directory of its own")
    }

    fn one_calendar_offered() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/dav/home/work/</d:href>
    <d:propstat><d:prop>
      <d:displayname>Work</d:displayname>
      <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
    </d:prop></d:propstat>
  </d:response>
</d:multistatus>"#
            .to_string()
    }

    #[tokio::test]
    async fn test_adding_a_calendar_from_a_server_stores_the_calendar_and_its_sign_in() {
        let cache = temp_cache("server");
        let (address, _listening) = answering(
            "207 Multi-Status",
            "application/xml",
            one_calendar_offered(),
        )
        .await;

        let added = add_from_a_server(
            &cache,
            "acc-1",
            &format!("http://{address}/dav/"),
            "sam",
            "hunter2",
            |offers| {
                assert_eq!(offers.len(), 1, "{offers:?}");
                Some(0)
            },
        )
        .await
        .expect("the calendar to be added")
        .expect("a calendar rather than a cancel");

        let stored = cache
            .get_calendars_for_account("acc-1")
            .expect("the calendars back");
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id, added.id);
        assert_eq!(stored[0].name, "Work");
        assert_eq!(
            stored[0].caldav_url.as_deref(),
            Some(format!("http://{address}/dav/home/work/").as_str())
        );
        assert_eq!(
            sign_in::load(&added.id),
            Some(("sam".to_string(), "hunter2".to_string()))
        );
    }

    #[tokio::test]
    async fn test_the_password_is_never_written_to_the_database() {
        // The database travels with a profile and with every backup. A
        // password in it is a password on somebody else's disk.
        let cache = temp_cache("no-password");
        let (address, _listening) = answering(
            "207 Multi-Status",
            "application/xml",
            one_calendar_offered(),
        )
        .await;

        add_from_a_server(
            &cache,
            "acc-1",
            &format!("http://{address}/dav/"),
            "sam",
            "hunter2",
            |_| Some(0),
        )
        .await
        .expect("the calendar to be added")
        .expect("a calendar rather than a cancel");

        let stored = &cache.get_calendars_for_account("acc-1").expect("rows")[0];
        for column in [
            stored.id.as_str(),
            stored.account_id.as_str(),
            stored.name.as_str(),
            stored.color.as_str(),
            stored.source_provider.as_deref().unwrap_or_default(),
            stored.caldav_url.as_deref().unwrap_or_default(),
            stored.subscription_url.as_deref().unwrap_or_default(),
            stored.etag.as_deref().unwrap_or_default(),
            stored.ctag.as_deref().unwrap_or_default(),
            stored.sync_token.as_deref().unwrap_or_default(),
            stored.created_at.as_str(),
            stored.updated_at.as_str(),
        ] {
            assert!(!column.contains("hunter2"), "the password is in: {column}");
            assert!(!column.contains("sam"), "the user name is in: {column}");
        }
    }

    #[tokio::test]
    async fn test_a_calendar_is_never_filed_under_this_computer() {
        // Only the account on screen is ever synced, and "local" is never it.
        // A calendar filed there is one nothing will ever visit, so saying so
        // beats a calendar that appears and stays empty for ever.
        let cache = temp_cache("local");

        let refused = add_from_a_server(
            &cache,
            crate::application::new_item::LOCAL_ACCOUNT_ID,
            "https://cal.example.com/dav/",
            "sam",
            "hunter2",
            |_| Some(0),
        )
        .await
        .expect_err("a refusal");

        assert!(refused.contains("account"), "{refused}");
        assert!(
            cache
                .get_calendars_for_account("local")
                .expect("rows")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_an_address_that_was_refused_never_reaches_the_network() {
        // The refusal has to happen before anything is sent, not after.
        let cache = temp_cache("refused");
        let (address, listening) = answering(
            "207 Multi-Status",
            "application/xml",
            one_calendar_offered(),
        )
        .await;

        let refused = add_from_a_server(
            &cache,
            "acc-1",
            // Unencrypted and not on this computer, so the sign-in would be
            // in clear on the wire.
            &format!("http://cal.example.com@{address}/dav/"),
            "sam",
            "hunter2",
            |_| Some(0),
        )
        .await
        .expect_err("a refusal");

        assert!(!refused.is_empty());
        let waited = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            heard(listening, "a request that should never have been made"),
        )
        .await;
        assert!(
            waited.is_err(),
            "a refused address still reached the server"
        );
    }

    #[tokio::test]
    async fn test_a_server_that_offers_nothing_leaves_no_calendar_behind() {
        let cache = temp_cache("empty");
        let nothing = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:multistatus xmlns:d="DAV:"></d:multistatus>"#
            .to_string();
        let (address, _listening) = answering("207 Multi-Status", "application/xml", nothing).await;

        let refused = add_from_a_server(
            &cache,
            "acc-1",
            &format!("http://{address}/dav/"),
            "sam",
            "hunter2",
            |_| Some(0),
        )
        .await
        .expect_err("a refusal");

        assert!(refused.contains("no calendars"), "{refused}");
        assert!(
            cache
                .get_calendars_for_account("acc-1")
                .expect("rows")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_choosing_nothing_adds_nothing_and_is_not_a_failure() {
        let cache = temp_cache("cancel");
        let (address, _listening) = answering(
            "207 Multi-Status",
            "application/xml",
            one_calendar_offered(),
        )
        .await;

        let chosen = add_from_a_server(
            &cache,
            "acc-1",
            &format!("http://{address}/dav/"),
            "sam",
            "hunter2",
            |_| None,
        )
        .await
        .expect("a cancel is not a failure");

        assert!(chosen.is_none());
        assert!(
            cache
                .get_calendars_for_account("acc-1")
                .expect("rows")
                .is_empty()
        );
    }

    #[test]
    fn test_adding_a_feed_needs_no_sign_in_and_asks_no_server() {
        let cache = temp_cache("feed");

        let added = add_from_a_feed(
            &cache,
            "acc-1",
            "webcal://example.com/holidays.ics",
            "Holidays",
        )
        .expect("the feed to be added");

        let stored = cache.get_calendars_for_account("acc-1").expect("rows");
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id, added.id);
        assert_eq!(
            stored[0].subscription_url.as_deref(),
            Some("https://example.com/holidays.ics")
        );
        assert_eq!(sign_in::load(&added.id), None);
    }

    #[test]
    fn test_a_feed_with_no_name_is_still_given_one_somebody_can_hear() {
        // A blank line in the sidebar is a row a screen reader reads as
        // nothing at all.
        let cache = temp_cache("unnamed-feed");

        let added = add_from_a_feed(&cache, "acc-1", "https://example.com/holidays.ics", "  ")
            .expect("the feed to be added");

        assert!(!added.name.trim().is_empty(), "{}", added.name);
    }

    #[test]
    fn test_a_feed_at_an_address_nobody_could_fetch_is_refused_before_it_is_stored() {
        let cache = temp_cache("bad-feed");

        let refused = add_from_a_feed(&cache, "acc-1", "file:///c:/holidays.ics", "Holidays")
            .expect_err("a refusal");

        assert_eq!(refused, Unusable::NotTheWeb.said());
        assert!(
            cache
                .get_calendars_for_account("acc-1")
                .expect("rows")
                .is_empty()
        );
    }

    // ── The loop that had no way in ──────────────────────────────────────

    #[tokio::test]
    async fn test_a_calendar_added_from_a_server_is_then_synced_by_the_loop_that_had_no_way_in() {
        // Not "it compiles": a row this screen writes is a row the sync visits
        // and fills. Two listeners, because each answers one request: the
        // first is asked what calendars there are, the second is the calendar.
        let cache = temp_cache("end-to-end");
        let (calendar_address, _events_listening) = answering(
            "207 Multi-Status",
            "application/xml",
            events_multistatus("evt-1"),
        )
        .await;
        let home_set = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>http://{calendar_address}/cal/</d:href>
    <d:propstat><d:prop>
      <d:displayname>Work</d:displayname>
      <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
    </d:prop></d:propstat>
  </d:response>
</d:multistatus>"#
        );
        let (home_address, _home_listening) =
            answering("207 Multi-Status", "application/xml", home_set).await;

        let added = add_from_a_server(
            &cache,
            "acc-1",
            &format!("http://{home_address}/dav/sam/"),
            "sam",
            "hunter2",
            |_| Some(0),
        )
        .await
        .expect("the calendar to be added")
        .expect("a calendar rather than a cancel");

        let (user_name, password) = sign_in::load(&added.id).expect("the sign-in back");
        let result = crate::application::caldav_sync::sync_caldav_calendar(
            &cache,
            &crate::service::caldav::CalDavClient::new(),
            &added,
            "acc-1",
            &user_name,
            &password,
        )
        .await
        .expect("the sync to run");

        assert_eq!(result.errors, Vec::<String>::new());
        let events = cache
            .get_events_for_calendar(&added.id)
            .expect("the events back");
        assert_eq!(events.len(), 1, "{events:?}");
        assert_eq!(events[0].summary, "Event evt-1");
    }

    fn events_multistatus(uid: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">\
             <d:response><d:href>/cal/{uid}.ics</d:href><d:propstat><d:prop>\
             <d:getetag>\"tag\"</d:getetag>\
             <c:calendar-data>BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:{uid}\n\
             SUMMARY:Event {uid}\nDTSTART:20260305T090000Z\nDTEND:20260305T100000Z\n\
             STATUS:CONFIRMED\nEND:VEVENT\nEND:VCALENDAR</c:calendar-data>\
             </d:prop></d:propstat></d:response></d:multistatus>"
        )
    }
}
