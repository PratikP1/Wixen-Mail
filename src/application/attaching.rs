//! Files chosen to go out with a message.
//!
//! The composer had an Attach File button from the beginning and nothing behind
//! it: no handler, no list, no column in the outbox, and no part in the message
//! SMTP built. A button that looks like it works and does nothing is the failure
//! this project's third guardrail names, and it is worse here than most places,
//! because somebody who cannot see the window has no way to notice that the file
//! they picked never arrived.
//!
//! What is in this module is the part that can be decided without a window: what
//! a chosen file is, what is said about it, and what the message may carry. The
//! picking, the list and the sending are elsewhere and use this.

use crate::common::{Error, Result};
use std::path::{Path, PathBuf};

/// The most a message may carry, in bytes.
///
/// Providers differ and most sit near this. Gmail and Outlook.com both refuse
/// past 25 MB, and the refusal arrives after the upload, as an SMTP error
/// somebody reads long after they pressed Send. Saying so before the file is
/// attached is the only version of this that helps.
///
/// This is the size on disk. What goes on the wire is base64, which is about a
/// third larger again, so the real ceiling is lower and [`over_the_limit`]
/// accounts for it.
pub const LIMIT_BYTES: u64 = 25 * 1024 * 1024;

/// How much base64 adds: four characters for every three bytes.
const ENCODED_OVER_RAW: f64 = 4.0 / 3.0;

/// A file picked to go with the message.
///
/// The bytes are not held. A message is composed over minutes and the file is
/// read when it is sent, so a picture edited in between goes out as the version
/// that existed at Send, which is the one somebody meant. It also keeps a
/// window that has been open all afternoon from holding a hundred megabytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// Where it is now. Not sent anywhere: the recipient gets the name.
    pub path: PathBuf,
    /// What the recipient will see, which is the file name and nothing else.
    pub name: String,
    /// Its size when it was picked, for saying so and for the running total.
    pub bytes: u64,
}

impl Chosen {
    /// Read a file that has just been picked.
    ///
    /// Fails rather than attaching a name with nothing behind it. A file that
    /// cannot be read now will not read at Send either, and finding that out
    /// then means finding it out after the rest of the message has gone.
    pub fn at(path: &Path) -> Result<Self> {
        let data = std::fs::metadata(path)
            .map_err(|e| Error::Other(format!("Could not read {}: {e}", path.display())))?;
        if data.is_dir() {
            return Err(Error::Other(format!(
                "{} is a folder, and a folder cannot be attached",
                path.display()
            )));
        }
        Ok(Self {
            path: path.to_path_buf(),
            name: crate::service::attachment_name::safe_file_name(
                &path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            bytes: data.len(),
        })
    }

    /// The one sentence a screen reader says when this row is reached.
    ///
    /// The same shape the reader uses for an attachment that arrived, because
    /// they are the same kind of thing and learning one list should teach the
    /// other.
    pub fn label(&self) -> String {
        format!(
            "{}, {}",
            self.name,
            crate::presentation::reader_text::human_size(self.bytes as usize)
        )
    }
}

/// What the line under the message says.
///
/// Said out loud when it changes as well as shown, because somebody who cannot
/// see it has nothing else to tell them the file went on.
pub fn summary(chosen: &[Chosen]) -> String {
    match chosen.len() {
        0 => "No attachments".to_string(),
        1 => format!("1 attachment: {}", chosen[0].label()),
        many => format!(
            "{many} attachments, {} in total",
            crate::presentation::reader_text::human_size(total_bytes(chosen) as usize)
        ),
    }
}

/// Everything picked, added up.
pub fn total_bytes(chosen: &[Chosen]) -> u64 {
    chosen.iter().map(|file| file.bytes).sum()
}

/// Why this will not send, if it will not.
///
/// Checked when a file is added rather than at Send, so the answer arrives
/// while somebody still has the file dialog in mind.
pub fn over_the_limit(chosen: &[Chosen]) -> Option<String> {
    let raw = total_bytes(chosen);
    let encoded = (raw as f64 * ENCODED_OVER_RAW) as u64;
    if encoded <= LIMIT_BYTES {
        return None;
    }
    Some(format!(
        "These files come to {} once encoded, and most providers refuse anything past {}. \
         Take one off, or send a link instead.",
        crate::presentation::reader_text::human_size(encoded as usize),
        crate::presentation::reader_text::human_size(LIMIT_BYTES as usize),
    ))
}

/// The content type to put on the part.
///
/// From the extension, because that is all there is: the file is on this
/// computer and nothing has declared what it holds. Unknown means
/// `application/octet-stream`, which is the honest answer and tells the
/// receiving client to treat it as a file rather than guess.
pub fn content_type(name: &str) -> &'static str {
    let extension = name
        .rsplit_once('.')
        .map(|(_, ext)| ext.trim().to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "pdf" => "application/pdf",
        "txt" | "log" | "md" => "text/plain",
        "html" | "htm" => "text/html",
        "csv" => "text/csv",
        "ics" => "text/calendar",
        "eml" => "message/rfc822",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "zip" => "application/zip",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

/// A file read and ready to go on a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready {
    /// What the recipient sees. Through the same cleaner a received name goes
    /// through, because a name with a path in it is a name that came from
    /// somewhere and is about to be written into a header.
    pub name: String,
    pub content_type: &'static str,
    pub bytes: Vec<u8>,
}

/// Read the files a queued message names.
///
/// At the moment of sending rather than when they were picked, so what goes out
/// is the version that exists now. The cost is that a file moved or deleted in
/// between cannot be sent, and this says which one and stops, rather than
/// sending a message that is missing the thing it was written about.
pub fn read_all(paths: &[PathBuf]) -> Result<Vec<Ready>> {
    paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(path).map_err(|e| {
                Error::Other(format!(
                    "{} could not be read, so the message was not sent: {e}",
                    path.display()
                ))
            })?;
            let name = crate::service::attachment_name::safe_file_name(
                &path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            );
            Ok(Ready {
                content_type: content_type(&name),
                name,
                bytes,
            })
        })
        .collect()
}

/// The paths, joined so one text column can hold them.
///
/// The outbox is a table with one row per message, and a message can carry
/// several files. A newline separates them because it is the one character a
/// Windows path cannot contain, so nothing needs escaping and nothing can be
/// split in the wrong place.
pub fn joined(chosen: &[Chosen]) -> String {
    chosen
        .iter()
        .map(|file| file.path.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The paths, back out of the column.
pub fn split(stored: &str) -> Vec<PathBuf> {
    stored
        .split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sized(name: &str, bytes: u64) -> Chosen {
        Chosen {
            path: PathBuf::from(format!("C:\\files\\{name}")),
            name: name.to_string(),
            bytes,
        }
    }

    #[test]
    fn test_nothing_attached_says_so() {
        assert_eq!(summary(&[]), "No attachments");
    }

    #[test]
    fn test_one_file_is_named_and_measured() {
        // The whole of what somebody needs to know it is the right file, in
        // the line they are already being read.
        assert_eq!(
            summary(&[sized("report.pdf", 245_760)]),
            "1 attachment: report.pdf, 240 KB"
        );
    }

    #[test]
    fn test_several_files_are_counted_and_added_up() {
        // Not every name: five names read out on every change is a paragraph
        // where a number was wanted, and the list beside it holds the names.
        assert_eq!(
            summary(&[sized("a.pdf", 1024), sized("b.png", 1024)]),
            "2 attachments, 2 KB in total"
        );
    }

    #[test]
    fn test_a_message_under_the_limit_is_not_complained_about() {
        assert_eq!(over_the_limit(&[sized("a.pdf", 1024 * 1024)]), None);
    }

    #[test]
    fn test_the_limit_counts_what_goes_on_the_wire() {
        // 20 MB on disk is about 27 MB encoded, which Gmail refuses. Checking
        // the size on disk would call this fine and let somebody find out from
        // a bounce an hour later.
        let twenty_megabytes = sized("film.mp4", 20 * 1024 * 1024);

        let complaint = over_the_limit(&[twenty_megabytes]).expect("20 MB should be too much");

        assert!(complaint.contains("25 MB"), "{complaint}");
        assert!(complaint.contains("send a link"), "{complaint}");
    }

    #[test]
    fn test_the_type_comes_from_the_extension() {
        for (name, expected) in [
            ("report.pdf", "application/pdf"),
            ("Photo.JPG", "image/jpeg"),
            ("notes.txt", "text/plain"),
            (
                "sheet.xlsx",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
        ] {
            assert_eq!(content_type(name), expected, "for {name}");
        }
    }

    #[test]
    fn test_a_type_nobody_knows_is_admitted_to_rather_than_guessed() {
        // octet-stream tells the receiving client to treat it as a file. A
        // guess would tell it to open something as the wrong kind.
        assert_eq!(content_type("archive.qqq"), "application/octet-stream");
        assert_eq!(content_type("no-extension"), "application/octet-stream");
    }

    #[test]
    fn test_the_paths_survive_the_trip_through_one_column() {
        let files = vec![sized("a.pdf", 1), sized("b b.png", 2)];

        let back = split(&joined(&files));

        assert_eq!(back, vec![files[0].path.clone(), files[1].path.clone()]);
    }

    #[test]
    fn test_an_empty_column_is_no_attachments_rather_than_one_with_no_name() {
        assert!(split("").is_empty());
        assert!(split("\n\n").is_empty());
    }

    #[test]
    fn test_a_folder_cannot_be_attached() {
        // A folder has a size and a name, so everything downstream would work
        // until the read at Send, which is after the rest of the message went.
        let folder = tempfile::tempdir().expect("temp dir");

        let refusal = Chosen::at(folder.path()).expect_err("a folder is not a file");

        assert!(format!("{refusal}").contains("folder"), "{refusal}");
    }

    #[test]
    fn test_a_file_that_is_not_there_is_refused_now_rather_than_at_send() {
        let missing = std::path::Path::new("C:\\nothing\\here\\at\\all.pdf");

        assert!(Chosen::at(missing).is_err());
    }

    #[test]
    fn test_the_file_is_read_at_the_moment_of_sending() {
        // Not when it was picked. A message is written over minutes, and the
        // version somebody meant to send is the one that exists at Send.
        let folder = tempfile::tempdir().expect("temp dir");
        let file = folder.path().join("notes.txt");
        std::fs::write(&file, b"first").expect("write");
        std::fs::write(&file, b"second, after some editing").expect("rewrite");

        let ready = read_all(&[file]).expect("a real file");

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].name, "notes.txt");
        assert_eq!(ready[0].content_type, "text/plain");
        assert_eq!(ready[0].bytes, b"second, after some editing");
    }

    #[test]
    fn test_a_file_that_has_gone_stops_the_send_rather_than_thinning_it() {
        // Sending the message without the file it was written about is the
        // worst of the three outcomes, and the one nobody would notice.
        let gone = PathBuf::from("C:\\moved\\away\\report.pdf");

        let refusal = read_all(&[gone]).expect_err("a file that is not there");

        let said = format!("{refusal}");
        assert!(said.contains("report.pdf"), "{said}");
        assert!(said.contains("not sent"), "{said}");
    }

    #[test]
    fn test_a_picked_file_is_named_and_measured() {
        let folder = tempfile::tempdir().expect("temp dir");
        let file = folder.path().join("report.pdf");
        std::fs::write(&file, vec![0u8; 2048]).expect("write");

        let chosen = Chosen::at(&file).expect("a real file");

        assert_eq!(chosen.name, "report.pdf");
        assert_eq!(chosen.bytes, 2048);
        assert_eq!(chosen.label(), "report.pdf, 2 KB");
    }
}
