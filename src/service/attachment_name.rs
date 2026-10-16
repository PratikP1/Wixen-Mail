//! Turning the name a stranger chose into one that is safe to write, and
//! marking what is written as having come from a stranger.
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
//!
//! A safe name is not a safe file. Once written, a stranger's `.docm` or `.js`
//! is a file Windows would open or run like one made on this computer, unless
//! it carries the Mark of the Web that Outlook, Thunderbird and every browser
//! write. [`mark_as_from_the_internet`] writes it, so Office opens the file in
//! Protected View and Windows asks before a script or program runs.

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
    //
    // The `||` cannot be swapped for `&&` and change anything: a non-empty
    // `cleaned` that is all dots still has to pass through `shorten` and then
    // `trim_end_matches(['.', ' '])` below, and a string of nothing but dots
    // is stripped down to empty by that call however long it is, landing on
    // the same `FALLBACK` this early return would have given. No input can
    // tell the two operators apart, so there is nothing here for a test to
    // pin; the paragraph above is the record of that, in place of one.
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

/// What Windows reads in a file's `Zone.Identifier` stream to know it came from
/// the internet: zone 3, the Internet zone, as browsers write it.
#[cfg(windows)]
const FROM_THE_INTERNET: &str = "[ZoneTransfer]\r\nZoneId=3\r\n";

/// Mark a file just written from a stranger's message as having come from the
/// internet, so Windows treats it as it treats a download.
///
/// The mark is the `Zone.Identifier` alternate stream. Rust hands
/// `path:Zone.Identifier` to `CreateFileW` unchanged, which names the stream
/// rather than a file. A volume that holds no alternate streams, FAT, exFAT
/// and some network shares, refuses the write, and that error comes back for
/// the caller to report while the file it saved stands.
#[cfg(windows)]
pub fn mark_as_from_the_internet(path: &std::path::Path) -> std::io::Result<()> {
    let mut stream = path.as_os_str().to_owned();
    stream.push(":Zone.Identifier");
    std::fs::write(stream, FROM_THE_INTERNET)
}

/// Off Windows there is no such mark to write, so there is nothing to do.
#[cfg(not(windows))]
pub fn mark_as_from_the_internet(_path: &std::path::Path) -> std::io::Result<()> {
    Ok(())
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
    fn test_a_name_with_no_dot_in_it_is_still_the_name_it_came_with() {
        // Every test here used a name containing a dot, so the check for a
        // name that is nothing but dots could be turned round and only a name
        // without one would show it. Every attachment called "Scan" or
        // "Notes" would have been saved as "attachment", and a mailbox full of
        // those is a mailbox where none of them can be told apart.
        assert_eq!(safe_file_name("Quarterly report"), "Quarterly report");
        assert_eq!(safe_file_name("Scan"), "Scan");
    }

    #[test]
    fn test_a_long_tail_after_the_last_dot_is_not_treated_as_an_extension() {
        // Keeping what follows the last dot only makes sense while it is short
        // enough to be an extension. Fifteen characters is one and sixteen is
        // not, and on either side of that the whole name still has to come in
        // under the limit, which is the thing this function is for.
        let fifteen = format!("{}.{}", "a".repeat(200), "b".repeat(15));
        let sixteen = format!("{}.{}", "a".repeat(200), "b".repeat(16));

        let kept = safe_file_name(&fifteen);
        assert!(
            kept.ends_with(&format!(".{}", "b".repeat(15))),
            "a fifteen character extension was not kept: {kept}"
        );
        assert!(kept.chars().count() <= MAX_LENGTH);

        let cut = safe_file_name(&sixteen);
        assert!(
            !cut.ends_with(&format!(".{}", "b".repeat(16))),
            "sixteen characters were kept as though they were an extension: {cut}"
        );
        assert!(cut.chars().count() <= MAX_LENGTH);
    }

    #[cfg(windows)]
    #[test]
    fn test_a_saved_file_is_marked_as_from_the_internet_and_keeps_its_bytes() {
        // A macro document a stranger sent. Without the mark Office opens it
        // outside Protected View, and a script saved the same way runs on a
        // double-click with no prompt.
        let folder = tempfile::tempdir().expect("a folder to save into");
        let saved = folder.path().join("invoice.docm");
        let bytes = b"PK\x03\x04 a stranger's macros";
        std::fs::write(&saved, bytes).expect("the save itself");

        mark_as_from_the_internet(&saved).expect("the mark");

        // Read as nothing when the stream is absent, so a file with no mark
        // fails on what it lacks rather than on the read.
        let mark = std::fs::read_to_string(format!("{}:Zone.Identifier", saved.display()))
            .unwrap_or_default();
        let lines: Vec<&str> = mark.lines().collect();
        assert!(
            lines.contains(&"[ZoneTransfer]") && lines.contains(&"ZoneId=3"),
            "the file carries no mark naming the Internet zone on lines of its own: {mark:?}"
        );
        assert_eq!(
            std::fs::read(&saved).expect("the saved file"),
            bytes,
            "marking the file changed what it holds"
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_a_place_that_cannot_hold_the_mark_says_so_rather_than_panicking() {
        // FAT, exFAT and some shares hold no alternate streams, so the write
        // of the mark fails there while the file itself was saved. A folder
        // that does not exist fails the same write the same way, and is one
        // every machine can make.
        let folder = tempfile::tempdir().expect("a folder to save into");
        let nowhere = folder
            .path()
            .join("a folder nobody made")
            .join("invoice.docm");

        let answer = mark_as_from_the_internet(&nowhere);

        assert!(
            answer.is_err(),
            "a mark that could not be written was reported as written"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn test_off_windows_marking_does_nothing() {
        let folder = tempfile::tempdir().expect("a folder to save into");
        let saved = folder.path().join("invoice.docm");
        std::fs::write(&saved, b"bytes").expect("the save itself");

        mark_as_from_the_internet(&saved).expect("nothing to fail");

        let entries = std::fs::read_dir(folder.path())
            .expect("the folder")
            .count();
        assert_eq!(entries, 1, "marking off Windows left something behind");
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
