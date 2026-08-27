//! Which typefaces this computer actually has.
//!
//! # Why they are counted rather than listed
//!
//! The language setting used to offer the same six languages whatever the
//! machine had, so choosing one it could not check set a value that changed
//! nothing, and the only way to find out was to write in that language and have
//! every word called a mistake. A typeface list written here would be the same
//! mistake with a different noun: Windows quietly substitutes something else
//! for a face it does not have, so a font that is not installed does not fail,
//! it just is not the font on the screen and nothing says so.
//!
//! So the list comes from the machine. Somebody choosing from it is choosing
//! something they have.
//!
//! # Why not through the toolkit
//!
//! wxWidgets has `wxFontEnumerator` and wxdragon does not wrap it. Asking
//! Windows directly is one call behind a small `#[link]` block, which is what
//! this project does everywhere else it needs Windows and not a whole crate.

use crate::common::{Error, Result};

/// A typeface name as Windows gives it, at most this many characters.
///
/// `LOGFONTW` carries a fixed 32, including the terminator, which is a Windows
/// limit rather than a choice made here.
const MOST_A_FACE_NAME_HOLDS: usize = 32;

/// Every typeface family installed, sorted, with no name twice.
///
/// Empty is a real answer only on a machine with no fonts, which does not
/// happen, so a caller treating empty as "could not ask" is not wrong.
#[cfg(target_os = "windows")]
pub fn installed_families() -> Result<Vec<String>> {
    #[repr(C)]
    struct LogFontW {
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u8,
        underline: u8,
        strike_out: u8,
        char_set: u8,
        out_precision: u8,
        clip_precision: u8,
        quality: u8,
        pitch_and_family: u8,
        face_name: [u16; MOST_A_FACE_NAME_HOLDS],
    }

    /// `DEFAULT_CHARSET`, which is what asks for every face rather than the
    /// faces of one script.
    const EVERY_CHARACTER_SET: u8 = 1;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetDC(window: isize) -> isize;
        fn ReleaseDC(window: isize, dc: isize) -> i32;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn EnumFontFamiliesExW(
            dc: isize,
            log_font: *mut LogFontW,
            each: extern "system" fn(*const LogFontW, *const u8, u32, isize) -> i32,
            carried: isize,
            flags: u32,
        ) -> i32;
    }

    /// Called once per face. Windows owns the pointer for the length of the
    /// call and nothing here keeps it.
    extern "system" fn take_one(
        found: *const LogFontW,
        _metrics: *const u8,
        _kind: u32,
        carried: isize,
    ) -> i32 {
        // Safe: Windows hands a live `LOGFONTW` for the length of this call,
        // and `carried` is the address of the vector passed in below, which
        // outlives the enumeration it is driving.
        let (found, into) = unsafe { (&*found, &mut *(carried as *mut Vec<String>)) };
        let name: String = found
            .face_name
            .iter()
            .take_while(|character| **character != 0)
            .filter_map(|character| char::from_u32(u32::from(*character)))
            .collect();
        if is_worth_offering(&name) {
            into.push(name);
        }
        // Anything but nought means keep going.
        1
    }

    // Safe: the screen's device context, released below whatever happens.
    let screen = unsafe { GetDC(0) };
    if screen == 0 {
        return Err(Error::Other(
            "Windows would not say which typefaces are installed".to_string(),
        ));
    }

    let mut found: Vec<String> = Vec::new();
    let mut asking = LogFontW {
        height: 0,
        width: 0,
        escapement: 0,
        orientation: 0,
        weight: 0,
        italic: 0,
        underline: 0,
        strike_out: 0,
        char_set: EVERY_CHARACTER_SET,
        out_precision: 0,
        clip_precision: 0,
        quality: 0,
        pitch_and_family: 0,
        face_name: [0; MOST_A_FACE_NAME_HOLDS],
    };
    // Safe: both structures live across the call, and the address handed over
    // as `carried` is the vector above, which outlives the enumeration.
    unsafe {
        EnumFontFamiliesExW(
            screen,
            &raw mut asking,
            take_one,
            (&raw mut found) as isize,
            0,
        );
        ReleaseDC(0, screen);
    }

    Ok(tidied(found))
}

#[cfg(not(target_os = "windows"))]
pub fn installed_families() -> Result<Vec<String>> {
    Err(Error::Other(
        "Reading the installed typefaces is only written for Windows".to_string(),
    ))
}

/// Whether a face name is one to put in front of somebody.
///
/// Windows lists every face twice for scripts that can be written down the
/// page: once normally and once with an `@` in front, which is the same font
/// rotated. Offering both means a list where half the entries are a second copy
/// of the other half, and choosing one of them turns the message list on its
/// side.
fn is_worth_offering(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('@')
}

/// Sorted, with no name twice.
///
/// Windows lists a family once per character set it covers, so a face with a
/// Cyrillic and a Latin version arrives twice with the same name.
fn tidied(mut found: Vec<String>) -> Vec<String> {
    found.sort_by_key(|name| name.to_lowercase());
    found.dedup_by_key(|name| name.to_lowercase());
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_font_rotated_down_the_page_is_not_offered() {
        // Windows lists these with an `@` in front, and they are the same font
        // turned on its side. Offering them doubles the list and choosing one
        // turns the message list sideways.
        assert!(!is_worth_offering("@MS Gothic"));
        assert!(is_worth_offering("MS Gothic"));
    }

    #[test]
    fn test_a_face_with_no_name_is_not_offered() {
        assert!(!is_worth_offering(""));
    }

    #[test]
    fn test_a_family_listed_once_per_script_is_offered_once() {
        // Windows enumerates a family once for each character set it covers,
        // so the same name arrives several times. A list with Arial three
        // times in it is a list somebody has to read three times.
        let tidy = tidied(vec![
            "Arial".to_string(),
            "Arial".to_string(),
            "Calibri".to_string(),
            "arial".to_string(),
        ]);

        assert_eq!(tidy, vec!["Arial".to_string(), "Calibri".to_string()]);
    }

    #[test]
    fn test_the_list_is_in_an_order_somebody_can_look_through() {
        // Sorted without regard to case, because a list where every capitalised
        // name comes before every lowercase one reads as two lists.
        let tidy = tidied(vec![
            "verdana".to_string(),
            "Arial".to_string(),
            "Consolas".to_string(),
        ]);

        assert_eq!(
            tidy,
            vec![
                "Arial".to_string(),
                "Consolas".to_string(),
                "verdana".to_string()
            ]
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_this_machine_really_says_which_typefaces_it_has() {
        // Proves the call works before anything trusts it being empty. An
        // enumeration that always failed would offer a list of nothing, which
        // looks exactly like a machine with no fonts and would send somebody
        // looking for a fault in the settings screen.
        let found = installed_families().expect("Windows would not answer");

        assert!(
            found.len() > 10,
            "only {} typefaces were found, which is not a real Windows machine: {found:?}",
            found.len()
        );
        // One that every Windows install has had for thirty years, so its
        // absence means the enumeration is not reading names properly rather
        // than that somebody uninstalled it.
        assert!(
            found.iter().any(|name| name.eq_ignore_ascii_case("Arial")),
            "Arial was not among {} typefaces, so the names are not being read",
            found.len()
        );
    }
}
