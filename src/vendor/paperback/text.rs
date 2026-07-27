// Vendored from Paperback (https://github.com/trypsynth/paperback).
//
// MIT License
//
// Copyright (c) 2025-2026 Quin Gillespie
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Local changes: import paths point at this module rather than paperback-core;
// the `t!` translation macro routes through Wixen Mail's own i18n registry;
// parsers and types this application does not use have been left behind.

// Local change: roman numeral list markers are spelled out here rather than
// pulling in a crate for five lines, so an ordered list using type="i" still
// reads as "iv." rather than "4.".

#[must_use]
pub fn remove_soft_hyphens(input: &str) -> String {
    input.replace("\u{00AD}", "")
}

#[must_use]
pub fn collapse_whitespace(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let mut result = String::with_capacity(input.len());
    let mut in_leading = true;
    let mut pending_space = false;
    for ch in input.chars() {
        if is_space_like(ch) {
            if in_leading {
                result.push(' ');
            } else {
                pending_space = true;
            }
        } else {
            in_leading = false;
            if pending_space {
                result.push(' ');
                pending_space = false;
            }
            result.push(ch);
        }
    }
    if pending_space {
        result.push(' ');
    }
    result
}

#[must_use]
pub fn trim_string(s: &str) -> String {
    s.trim_matches(is_space_like).to_string()
}

/// Display units match the index space of the platform text control that the
/// document text is loaded into: UTF-16 code units on Windows (Win32 edit
/// control) and macOS (NSTextView's NSRange, which wxWidgets passes through
/// unconverted); Unicode characters on GTK (GtkTextIter offsets).
#[must_use]
pub fn display_len(s: &str) -> usize {
    if cfg!(any(windows, target_os = "macos")) {
        s.encode_utf16().count()
    } else {
        s.chars().count()
    }
}

#[must_use]
pub const fn ch_width(ch: char) -> usize {
    if cfg!(any(windows, target_os = "macos")) {
        ch.len_utf16()
    } else {
        1
    }
}

#[must_use]
pub const fn is_space_like(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '\u{00A0}' | '\u{200B}')
}

#[must_use]
pub fn format_list_item(number: i32, list_type: &str) -> String {
    match list_type {
        "a" => to_alpha(number, false),
        "A" => to_alpha(number, true),
        "i" => to_roman(number).map_or_else(|| number.to_string(), |s| s.to_lowercase()),
        "I" => to_roman(number).unwrap_or_else(|| number.to_string()),
        _ => number.to_string(),
    }
}

fn to_alpha(mut n: i32, uppercase: bool) -> String {
    if n <= 0 {
        return n.to_string();
    }
    let mut result = String::new();
    let base = if uppercase { b'A' } else { b'a' };
    while n > 0 {
        n -= 1;
        let offset = u8::try_from(n % 26).unwrap_or(0);
        result.insert(0, (base + offset) as char);
        n /= 26;
    }
    result
}

// Upstream's own test module is not vendored: it reaches into paperback-core's
// document buffer and marker types, which are not here, and uses a test crate
// this project does not depend on. The tests that matter for the way this
// application uses the converter live in the module file beside it.

/// A roman numeral, for an ordered list that asked for one.
///
/// Local addition, replacing the `roman` crate. `None` outside the range roman
/// numerals can express, where the caller falls back to the plain number rather
/// than inventing notation.
fn to_roman(number: i32) -> Option<String> {
    const NUMERALS: [(i32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    if !(1..4000).contains(&number) {
        return None;
    }
    let mut left = number;
    let mut out = String::new();
    for (value, numeral) in NUMERALS {
        while left >= value {
            out.push_str(numeral);
            left -= value;
        }
    }
    Some(out)
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn test_roman_numerals_match_what_the_crate_produced() {
        // The cases an ordered list actually reaches, plus the boundaries.
        for (number, expected) in [
            (1, "I"),
            (4, "IV"),
            (9, "IX"),
            (14, "XIV"),
            (40, "XL"),
            (90, "XC"),
            (400, "CD"),
            (900, "CM"),
            (1987, "MCMLXXXVII"),
            (3999, "MMMCMXCIX"),
        ] {
            assert_eq!(
                to_roman(number).as_deref(),
                Some(expected),
                "for {}",
                number
            );
        }
    }

    #[test]
    fn test_a_number_roman_cannot_express_falls_back_to_digits() {
        // Inventing notation would be worse than reading the number.
        for number in [0, -1, 4000, i32::MAX] {
            assert_eq!(to_roman(number), None);
            assert_eq!(format_list_item(number, "i"), number.to_string());
        }
    }

    #[test]
    fn test_alphabetic_list_markers_carry_past_z() {
        assert_eq!(format_list_item(1, "a"), "a");
        assert_eq!(format_list_item(26, "a"), "z");
        assert_eq!(format_list_item(27, "a"), "aa");
        assert_eq!(format_list_item(1, "A"), "A");
    }
}
