//! Which zone a stored zone name means, whoever wrote the name.
//!
//! A calendar document names the zone its times are written in, and two
//! vocabularies arrive here. Google, Apple and every calendar server write the
//! zone database's names, `America/Los_Angeles`; some older servers write the
//! same name behind a path of their own, `/mozilla.org/20050126_1/America/New_York`,
//! which RFC 5545 allows by the leading solidus. Outlook and Microsoft Graph
//! write Windows' names, `Pacific Standard Time`, which the zone database does
//! not know at all.
//!
//! Windows' own copy of ICU carries the table between the two and Windows keeps
//! it current, so a Windows name is asked of Windows rather than of a table
//! kept here. Measured on 2026-09-26 against the 141 zones this machine's
//! registry lists: 139 came back as a name `chrono-tz` reads, and the two that
//! did not, Kamchatka and Mid-Atlantic, are zones Windows has retired.
//!
//! A name nothing here can place is not guessed at. The caller keeps the hour
//! as it was written and says so, because a meeting put at a guessed hour is
//! one somebody turns up to at the wrong time.
//!
//! Here in `common` because `common::moment` needs the answer and `common`
//! imports from no other layer.

use chrono_tz::Tz;

/// The zone a stored name means, when this computer can place it.
///
/// The one place a zone's name becomes a zone. Trimmed first, because a name
/// with a space after it is still the zone it names and Windows answers
/// nothing for it; then the zone database's own names, then a calendar
/// server's path ending in one, then Windows' names through Windows.
pub fn the_zone_called(name: &str) -> Option<Tz> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    name.parse()
        .ok()
        .or_else(|| the_zone_a_path_ends_in(name))
        .or_else(|| the_zone_windows_calls(name)?.parse().ok())
}

/// The zone a calendar server's path names, by the longest run of its last
/// parts that is a zone's name: `/mozilla.org/20050126_1/America/New_York` is
/// `America/New_York`. Longest first, so a zone named in two parts is never
/// read by its last part alone.
fn the_zone_a_path_ends_in(name: &str) -> Option<Tz> {
    let parts: Vec<&str> = name.strip_prefix('/')?.split('/').collect();
    (0..parts.len()).find_map(|from| parts[from..].join("/").parse().ok())
}

/// The zone database's name for a zone Windows names, asked of Windows' own
/// ICU, which keeps the table between the two current.
#[cfg(target_os = "windows")]
fn the_zone_windows_calls(name: &str) -> Option<String> {
    // Only this half calls Windows, so only this half imports it: an import
    // the other half does not use is a warning, and warnings fail the build.
    use windows::Win32::Globalization::{U_ZERO_ERROR, UErrorCode, ucal_getTimeZoneIDForWindowsID};

    let asked: Vec<u16> = name.encode_utf16().collect();
    let asked_length = i32::try_from(asked.len()).ok()?;
    let mut answer = [0u16; 128];
    let capacity = i32::try_from(answer.len()).ok()?;
    let mut status: UErrorCode = U_ZERO_ERROR;
    // SAFETY: the name is read for exactly `asked_length` units from a buffer
    // that holds that many, and ICU writes at most `capacity` units into
    // `answer`, which is the buffer's own length. No region is passed, so
    // Windows' default mapping is the one asked for. The name comes from a
    // stranger's calendar document; nothing here trusts what it says beyond
    // handing it over and reading back only what ICU says it wrote.
    let written = unsafe {
        ucal_getTimeZoneIDForWindowsID(
            asked.as_ptr(),
            asked_length,
            windows::core::PCSTR::null(),
            answer.as_mut_ptr(),
            capacity,
            &mut status,
        )
    };
    // A status above zero is ICU's failure, below it a warning such as an
    // answer that exactly filled the buffer with no terminator, which is still
    // an answer.
    let written = usize::try_from(written)
        .ok()
        .filter(|written| (1..=answer.len()).contains(written))?;
    (status.0 <= 0).then(|| String::from_utf16_lossy(&answer[..written]))
}

/// Where there is no Windows, a Windows name places nothing. No table is kept
/// here to stand in for Windows, so the hour is kept as it was written.
#[cfg(not(target_os = "windows"))]
fn the_zone_windows_calls(_name: &str) -> Option<String> {
    None
}

/// The zone database's ways of writing universal time, beside the `Etc/`
/// family, none of which is a place anybody is.
const UNIVERSAL_TIME: [&str; 9] = [
    "UTC",
    "UCT",
    "GMT",
    "GMT0",
    "GMT+0",
    "GMT-0",
    "Greenwich",
    "Universal",
    "Zulu",
];

/// Whether a zone is somewhere people are, rather than a way of writing
/// universal time.
///
/// A server that delivers every time in UTC says nothing about where the
/// meeting's organiser sits, so a time written in it is worth nothing said
/// about its own clock.
pub fn names_a_place(zone: Tz) -> bool {
    let name = zone.name();
    !(name.starts_with("Etc/") || UNIVERSAL_TIME.contains(&name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> Option<&'static str> {
        the_zone_called(name).map(|zone| zone.name())
    }

    #[test]
    fn test_a_name_the_zone_database_knows_is_that_zone() {
        assert_eq!(named("America/Los_Angeles"), Some("America/Los_Angeles"));
        assert_eq!(named("UTC"), Some("UTC"));
        assert_eq!(named("Etc/GMT+5"), Some("Etc/GMT+5"));
    }

    #[test]
    fn test_a_calendar_servers_path_is_the_zone_it_ends_in() {
        // RFC 5545 section 3.2.19: a TZID beginning with a solidus is a
        // globally unique name, and the servers that write one end it in the
        // zone database's own name.
        assert_eq!(
            named("/mozilla.org/20050126_1/America/New_York"),
            Some("America/New_York")
        );
        assert_eq!(
            named("/citadel.org/20190914_1/Europe/London"),
            Some("Europe/London")
        );
    }

    #[test]
    fn test_a_name_with_space_round_it_is_still_the_zone_it_names() {
        assert_eq!(named("  America/Los_Angeles "), Some("America/Los_Angeles"));
    }

    #[test]
    fn test_a_name_of_nothing_but_space_names_no_zone() {
        for blank in ["", " ", "\t", "\r\n"] {
            assert_eq!(named(blank), None, "for {blank:?}");
        }
    }

    #[test]
    fn test_a_name_nobody_knows_names_no_zone() {
        // Outlook writes the first for a zone somebody built by hand and the
        // third where it wrote the zone's display name instead of its key.
        for unknown in [
            "Customized Time Zone",
            "Not A Zone",
            "(UTC-05:00) Eastern Time (US & Canada)",
            "tzone://Microsoft/Utc",
            "/",
            "/only.a.path/",
        ] {
            assert_eq!(named(unknown), None, "for {unknown:?}");
        }
    }

    #[test]
    fn test_a_zone_windows_has_retired_names_no_zone() {
        // Measured on 2026-09-26: the two names in this machine's registry
        // that Windows' ICU maps to nothing.
        for retired in ["Kamchatka Standard Time", "Mid-Atlantic Standard Time"] {
            assert_eq!(named(retired), None, "for {retired:?}");
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_windows_name_is_the_zone_windows_says_it_means() {
        for (windows, zone) in [
            ("Pacific Standard Time", "America/Los_Angeles"),
            ("GMT Standard Time", "Europe/London"),
            ("W. Europe Standard Time", "Europe/Berlin"),
            ("Tokyo Standard Time", "Asia/Tokyo"),
            ("UTC", "UTC"),
        ] {
            assert_eq!(named(windows), Some(zone), "for {windows:?}");
        }
        // Windows answers the zone database's older name here, and either is
        // the same clock.
        assert!(
            matches!(
                named("India Standard Time"),
                Some("Asia/Calcutta" | "Asia/Kolkata")
            ),
            "{:?}",
            named("India Standard Time")
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_windows_name_with_space_round_it_is_asked_without_the_space() {
        // Windows answers nothing for "Eastern Standard Time " with the space.
        assert_eq!(
            named(" Pacific Standard Time "),
            Some("America/Los_Angeles")
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_off_windows_a_windows_name_names_no_zone() {
        // There is no Windows to ask, and no table is kept here to ask
        // instead, so the hour is kept as written and said to be unplaced.
        assert_eq!(named("Pacific Standard Time"), None);
    }

    #[test]
    fn test_a_way_of_writing_universal_time_names_no_place() {
        for universal in [
            "UTC",
            "GMT",
            "Zulu",
            "Universal",
            "UCT",
            "Greenwich",
            "Etc/UTC",
            "Etc/GMT+5",
        ] {
            let zone: Tz = universal.parse().expect("a zone the database knows");
            assert!(!names_a_place(zone), "{universal} names a place");
        }
    }

    #[test]
    fn test_a_zone_somebody_lives_in_names_a_place() {
        for place in ["America/Los_Angeles", "Europe/London", "Asia/Tokyo"] {
            let zone: Tz = place.parse().expect("a zone the database knows");
            assert!(names_a_place(zone), "{place} names no place");
        }
    }
}
