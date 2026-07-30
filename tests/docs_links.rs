//! Every link between our own documents has to go somewhere.
//!
//! These files are not only read on GitHub, where a broken link is merely
//! annoying. `docs\*.md` ships inside the installer and sits beside the
//! program, and the first-run screen has a button that opens one of them. A
//! link to a file that was renamed, or never written, is a dead end for
//! somebody who is already having trouble.
//!
//! It has happened: `ALPHA_TESTING.md`, the page the first-run button opens,
//! pointed at `docs/DATA_LOCATIONS.md`, which has never existed.
//!
//! Links to the web are not checked. That needs the network, would make the
//! suite fail when somebody else's site is down, and is a different job.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// A link found in a document, and where it points.
#[derive(Debug)]
struct Link {
    /// The document it was found in.
    from: PathBuf,
    /// The text between the brackets, so a failure names the link a person sees.
    label: String,
    /// The part before any `#`, empty for a link to a heading in the same file.
    file: String,
    /// The part after any `#`, empty when there is none.
    anchor: String,
}

/// Every markdown file that ships or is read on the repository page.
fn documents() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    collect_markdown(Path::new("docs"), &mut found);
    found.push(PathBuf::from("README.md"));
    found.push(PathBuf::from("CLAUDE.md"));
    found.sort();
    found
}

fn collect_markdown(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, into);
        } else if path.extension().is_some_and(|e| e == "md") {
            into.push(path);
        }
    }
}

/// Pull `[label](target)` out of one document, skipping the web and code spans.
///
/// Hand-rolled rather than pulling in a markdown parser: the shape being
/// looked for is small and exact, and a dependency that exists only to run one
/// test is a dependency somebody has to keep.
fn links_in(path: &Path) -> Vec<Link> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut links = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut at = 0;

    while at < bytes.len() {
        if bytes[at] != '[' {
            at += 1;
            continue;
        }
        let Some(label_end) = find(&bytes, at + 1, ']') else {
            break;
        };
        if bytes.get(label_end + 1) != Some(&'(') {
            at += 1;
            continue;
        }
        let Some(target_end) = find(&bytes, label_end + 2, ')') else {
            break;
        };

        let label: String = bytes[at + 1..label_end].iter().collect();
        let target: String = bytes[label_end + 2..target_end].iter().collect();
        at = target_end + 1;

        let target = target.trim();
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
        {
            continue;
        }

        let (file, anchor) = match target.split_once('#') {
            Some((file, anchor)) => (file, anchor),
            None => (target, ""),
        };
        links.push(Link {
            from: path.to_path_buf(),
            label,
            file: file.to_string(),
            anchor: anchor.to_string(),
        });
    }
    links
}

fn find(chars: &[char], from: usize, wanted: char) -> Option<usize> {
    chars[from..]
        .iter()
        .position(|c| *c == wanted)
        .map(|offset| from + offset)
}

/// The identifiers GitHub gives headings, which is what `#anchor` addresses.
///
/// Lowercased, punctuation dropped, spaces turned into hyphens. Close enough
/// for the headings this project writes; it would need more care for headings
/// carrying code spans or emoji, and there are none.
fn anchors_in(path: &Path) -> HashSet<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return HashSet::new();
    };
    text.lines()
        .filter_map(|line| line.trim_start().strip_prefix('#'))
        .map(|rest| rest.trim_start_matches('#').trim())
        .map(|heading| {
            heading
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .collect::<String>()
                .to_lowercase()
                .replace(' ', "-")
        })
        .collect()
}

#[test]
fn test_every_link_between_our_documents_goes_somewhere() {
    let mut dead = Vec::new();

    for document in documents() {
        let folder = document.parent().unwrap_or(Path::new(".")).to_path_buf();

        for link in links_in(&document) {
            let target = if link.file.is_empty() {
                document.clone()
            } else {
                folder.join(&link.file)
            };

            if !target.exists() {
                dead.push(format!(
                    "{}: [{}] points at {}, which is not there",
                    link.from.display(),
                    link.label,
                    target.display()
                ));
                continue;
            }

            if !link.anchor.is_empty() && !anchors_in(&target).contains(&link.anchor) {
                dead.push(format!(
                    "{}: [{}] points at a heading \"{}\" that {} does not have",
                    link.from.display(),
                    link.label,
                    link.anchor,
                    target.display()
                ));
            }
        }
    }

    assert!(dead.is_empty(), "dead links:\n  {}", dead.join("\n  "));
}

#[test]
fn test_the_check_is_actually_reading_links() {
    // Without this, deleting the body of `links_in` would leave the test above
    // passing on an empty list and reporting that every link is fine.
    let counted: usize = documents().iter().map(|d| links_in(d).len()).sum();

    assert!(
        counted > 20,
        "only {counted} links between our documents were found, which means the reader is broken"
    );
}
