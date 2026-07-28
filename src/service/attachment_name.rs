//! Turning the name a stranger chose into one that is safe to write.
//!
//! An attachment's filename comes from whoever sent the message. It is not a
//! name, it is untrusted input that happens to look like one, and every part of
//! it is somebody's opportunity:
//!
//! - `..\..\..\Windows\System32\drivers\etc\hosts` writes outside the folder
//!   the person chose.
//! - `CON`, `PRN`, `NUL` and the `COM1` to `LPT9` names are devices on
//!   Windows, and opening a file with one of those names does something other
//!   than opening a file, whatever extension is on the end.
//! - A trailing dot or space is silently stripped by the filesystem, so
//!   `evil.txt.` and `evil.txt` are the same file and only one of them was
//!   checked.
//! - A right-to-left override in the middle of a name makes `annexe\u{202E}cod.exe`
//!   read as `annexeexe.doc` to a person and stay an executable to Windows.
//!
//! The last one matters more here than in most applications. A sighted reader
//! has a chance of noticing a filename that looks odd; a synthesiser reading
//! the reordered name aloud gives no hint at all.

/// Characters Windows will not accept in a filename.
const FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Names that are devices on Windows whatever extension follows them.
const DEVICE_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// What to call an attachment whose name is unusable or missing.
const FALLBACK: &str = "attachment";

/// The longest name to write.
///
/// Well under the path limit, because the folder somebody chose is in front of
/// it and a two hundred character name plus a deep path is a write that fails
/// for a reason nobody could guess from the error.
const MAX_LENGTH: usize = 120;

/// A filename that is safe to join to a folder somebody chose.
///
/// Never empty, never a path, never a device, and always something a person can
/// hear read back to them.
pub fn safe_file_name(proposed: &str) -> String {
    // Only the last segment, whichever separator was used. A sender who writes
    // a path has either made a mistake or is trying something.
    let leaf = proposed
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(proposed)
        .trim();

    let cleaned: String = leaf
        .chars()
        .filter(|c| {
            // Control characters, and the bidirectional overrides that make a
            // name read backwards to a person and forwards to the filesystem.
            !c.is_control() && !matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
        })
        .map(|c| if FORBIDDEN.contains(&c) { '_' } else { c })
        .collect();

    // A name that is only dots is `.` or `..`, which are directories.
    let cleaned = cleaned
        .trim_matches(|c: char| c.is_whitespace())
        .to_string();
    if cleaned.is_empty() || cleaned.chars().all(|c| c == '.') {
        return FALLBACK.to_string();
    }

    let cleaned = shorten(&cleaned);

    // Trailing dots and spaces are stripped by the filesystem after any check
    // we make, so they are stripped here where the result is what gets used.
    let trimmed = cleaned.trim_end_matches(['.', ' ']).to_string();
    let trimmed = if trimmed.is_empty() {
        FALLBACK.to_string()
    } else {
        trimmed
    };

    if is_device_name(&trimmed) {
        // Prefixed rather than rejected: the sender's name is still the most
        // useful thing to show, and "file-NUL.txt" says what it came from.
        return format!("file-{trimmed}");
    }
    trimmed
}

/// Whether the stem is one of Windows' device names.
fn is_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    DEVICE_NAMES.contains(&stem.as_str())
}

/// Shorten the stem, keeping the extension.
///
/// The extension is what decides how a file opens, so it is the last thing to
/// lose. Cutting from the end of the whole name would drop it and leave
/// something Windows cannot open at all.
fn shorten(name: &str) -> String {
    if name.chars().count() <= MAX_LENGTH {
        return name.to_string();
    }
    match name.rsplit_once('.') {
        Some((stem, extension)) if extension.chars().count() < 16 => {
            let room = MAX_LENGTH.saturating_sub(extension.chars().count() + 1);
            let kept: String = stem.chars().take(room).collect();
            format!("{kept}.{extension}")
        }
        _ => name.chars().take(MAX_LENGTH).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_ordinary_name_is_left_alone() {
        assert_eq!(
            safe_file_name("Quarterly report.pdf"),
            "Quarterly report.pdf"
        );
    }

    #[test]
    fn test_a_path_becomes_its_last_part() {
        // The only thing a sender can decide is what the file is called, not
        // where it goes. That is the person saving it.
        for proposed in [
            r"..\..\..\Windows\System32\drivers\etc\hosts",
            "../../etc/passwd",
            r"C:\Windows\notepad.exe",
        ] {
            let safe = safe_file_name(proposed);

            assert!(!safe.contains(['/', '\\']), "{proposed} kept a separator");
            assert!(!safe.contains(".."), "{proposed} kept a parent step");
        }
    }

    #[test]
    fn test_a_name_that_is_only_dots_is_not_a_directory() {
        for proposed in [".", "..", "..."] {
            assert_eq!(safe_file_name(proposed), "attachment", "for {proposed}");
        }
    }

    #[test]
    fn test_a_device_name_does_not_stay_one() {
        // Opening a file called NUL.txt on Windows does something other than
        // opening a file, whatever is on the end of it.
        for proposed in ["NUL", "nul.txt", "COM1.pdf", "LPT9.docx", "con"] {
            let safe = safe_file_name(proposed);

            assert!(!is_device_name(&safe), "{proposed} stayed a device: {safe}");
            // The sender's name is still in there, because it is still the
            // most useful thing to show somebody.
            assert!(safe.len() > proposed.len(), "{proposed} lost its name");
        }
    }

    #[test]
    fn test_a_name_written_backwards_is_not_written_backwards() {
        // A right-to-left override makes this read as "annexeexe.doc" to a
        // person and stay an executable to Windows. A synthesiser reading the
        // reordered name aloud gives no hint at all, which is why this matters
        // here more than in most applications.
        let disguised = "annexe\u{202E}cod.exe";

        let safe = safe_file_name(disguised);

        assert!(!safe.contains('\u{202E}'), "kept the override: {safe:?}");
        assert!(safe.ends_with(".exe"), "the real extension moved: {safe}");
    }

    #[test]
    fn test_a_trailing_dot_does_not_survive_to_be_stripped_later() {
        // The filesystem strips these after any check, so "evil.txt." and
        // "evil.txt" become the same file and only one of them was looked at.
        assert_eq!(safe_file_name("report.pdf."), "report.pdf");
        assert_eq!(safe_file_name("report.pdf   "), "report.pdf");
    }

    #[test]
    fn test_forbidden_characters_become_underscores_rather_than_vanishing() {
        // Removing them can weld two words together into something that reads
        // as one; an underscore keeps the shape of the name a sender chose.
        let safe = safe_file_name("Q1:Q2 report<final>.pdf");

        assert!(!safe.contains([':', '<', '>']), "{safe}");
        assert!(safe.contains("Q1_Q2"), "{safe}");
    }

    #[test]
    fn test_an_empty_or_missing_name_still_gets_one() {
        for proposed in ["", "   ", "\u{202E}"] {
            assert_eq!(safe_file_name(proposed), "attachment", "for {proposed:?}");
        }
    }

    #[test]
    fn test_a_very_long_name_keeps_its_extension() {
        // The extension decides how the file opens, so it is the last thing to
        // lose. Cutting from the end would drop it and leave something Windows
        // cannot open at all.
        let long = format!("{}.pdf", "a".repeat(400));

        let safe = safe_file_name(&long);

        assert!(safe.ends_with(".pdf"), "{safe}");
        assert!(
            safe.chars().count() <= MAX_LENGTH,
            "{}",
            safe.chars().count()
        );
    }

    #[test]
    fn test_every_result_is_usable_as_a_name() {
        // The property the rest of the application relies on, over everything
        // above at once.
        for proposed in [
            "",
            "..",
            "NUL",
            r"..\..\evil.exe",
            "annexe\u{202E}cod.exe",
            "report.pdf.",
            &"x".repeat(500),
            "Q1:Q2<>|?*.pdf",
        ] {
            let safe = safe_file_name(proposed);

            assert!(!safe.is_empty(), "empty for {proposed:?}");
            assert!(!safe.contains(['/', '\\']), "separator for {proposed:?}");
            assert!(!safe.contains(FORBIDDEN), "forbidden for {proposed:?}");
            assert!(!safe.ends_with(['.', ' ']), "trailing for {proposed:?}");
            assert!(!is_device_name(&safe), "device for {proposed:?}");
            assert!(
                safe.chars().count() <= MAX_LENGTH + 5,
                "long for {proposed:?}"
            );
        }
    }
}
