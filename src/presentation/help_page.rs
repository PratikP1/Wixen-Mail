//! Turning a shipped document into a page a person can actually read.
//!
//! The guides are written and kept as Markdown, which is right for writing and
//! wrong for reading. Opening a `.md` file hands it to whatever Windows has
//! associated, which on most machines is a text editor or nothing at all, and
//! on this project's first-run screen that is the first thing an alpha tester
//! is offered. Markdown source read out by a screen reader is hash signs,
//! pipes and square brackets: the headings that make a long page navigable are
//! punctuation rather than headings.
//!
//! So the Markdown is turned into HTML and the HTML is opened. The browser is
//! the reader for now, and it is a good one: real heading navigation, find,
//! zoom, and a screen reader that already knows it well.
//!
//! # What this is not yet
//!
//! A help system. There is no landing page, no organisation by topic, no `F1`,
//! and nothing published on the website. That is its own piece of work; this
//! is the converter it will be built on, and the reason it exists now is that
//! the first-run button had to stop opening raw Markdown before anybody else
//! saw it.

use crate::common::{Error, Result};
use std::path::{Path, PathBuf};

/// Where the shipped documents are, relative to the program.
const DOCUMENTS: &str = "docs";

/// Find one of the documents that ship beside the program.
///
/// Beside the executable, not beside the working directory. An installed copy
/// is started from wherever the shortcut points, usually the person's home
/// folder, so a relative path finds nothing and the button appears broken.
/// That is not something a test in this repository would notice, because
/// `cargo test` runs with the repository as the working directory, and it is
/// exactly the bug this function was written for.
///
/// Falls back to the plain relative path, which is where the documents are
/// under `cargo run`.
pub fn shipped(file_name: &str) -> PathBuf {
    let relative = Path::new(DOCUMENTS).join(file_name);
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(&relative)))
        .filter(|beside| beside.exists())
        .unwrap_or(relative)
}

/// Convert the shipped documents and open one of them.
///
/// All of them, not only the one asked for. The pages link to each other, and
/// those links are rewritten to point at converted pages, so converting one
/// alone would give somebody a page whose every link is broken. There are two
/// dozen small files; doing the lot takes less time than the browser takes to
/// start.
pub fn open(file_name: &str) -> Result<PathBuf> {
    let source = shipped(file_name);
    let folder = source
        .parent()
        .ok_or_else(|| Error::Other(format!("{} has nowhere to sit", source.display())))?;

    let into = convert_folder(folder)?;
    let page = into.join(Path::new(file_name).with_extension("html"));

    open::that(&page)
        .map_err(|e| Error::Other(format!("Could not open {}: {e}", page.display())))?;
    Ok(page)
}

/// Turn every Markdown file in a folder into a page, and say where they went.
///
/// Written next to the sources rather than into a temporary folder, so a
/// person who wants to keep a page open, bookmark it or send it to somebody
/// has a file that is still there tomorrow, and so the links between them work
/// without any rewriting of paths. Falls back to a folder of our own under the
/// person's temporary directory when the program's folder cannot be written
/// to, which is what an all-users install under Program Files looks like to
/// somebody who is not an administrator.
fn convert_folder(from: &Path) -> Result<PathBuf> {
    let into = destination_for(from, is_writable(from));
    std::fs::create_dir_all(&into)
        .map_err(|e| Error::Other(format!("Could not make {}: {e}", into.display())))?;

    convert_into(from, &into)?;
    Ok(into)
}

/// Where the pages go, given whether the folder holding the documents takes a
/// file.
///
/// Apart from the asking, because asking means writing a file and deleting it
/// again, and what can go wrong here is the decision rather than the probe.
fn destination_for(from: &Path, can_write: bool) -> PathBuf {
    if can_write {
        from.to_path_buf()
    } else {
        std::env::temp_dir().join("wixen-mail-help")
    }
}

/// Convert every Markdown file in `from`, writing the pages to `into`.
fn convert_into(from: &Path, into: &Path) -> Result<usize> {
    let entries = std::fs::read_dir(from)
        .map_err(|e| Error::Other(format!("Could not read {}: {e}", from.display())))?;

    let mut converted = 0;
    for entry in entries.flatten() {
        let source = entry.path();
        if source.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let Some(file_name) = source.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // Not silently absorbed: one unreadable document should not stop the
        // rest, and the one that failed is named.
        match std::fs::read_to_string(&source) {
            Ok(markdown) => {
                let page = to_html(&markdown, &title_of(&markdown, file_name));
                let destination = into.join(Path::new(file_name).with_extension("html"));
                if let Err(e) = std::fs::write(&destination, page) {
                    tracing::warn!("Could not write {}: {e}", destination.display());
                } else {
                    converted += 1;
                }
            }
            Err(e) => tracing::warn!("Could not read {}: {e}", source.display()),
        }
    }
    Ok(converted)
}

/// Whether we can put a file in this folder.
///
/// Asked by trying, because the answer on Windows depends on the folder's
/// permissions, on whether the process is elevated, and on virtualisation
/// rules that no attribute on the folder reports faithfully.
fn is_writable(folder: &Path) -> bool {
    let probe = folder.join(".wixen-mail-write-test");
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// The document's own first heading, or its file name if it has none.
fn title_of(markdown: &str, file_name: &str) -> String {
    markdown
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(|heading| heading.trim().to_string())
        .unwrap_or_else(|| file_name.to_string())
}

/// Turn Markdown into a whole HTML document.
///
/// A whole document rather than a fragment, because the parts a fragment
/// leaves out are the accessible ones: the language, which is what tells a
/// screen reader which voice to read it in, the title, which is what the
/// window and the tab announce, and a width that keeps lines readable at 200%
/// zoom.
///
/// Colours follow the system light or dark setting rather than being fixed,
/// and nothing here conveys anything by colour alone.
pub fn to_html(markdown: &str, title: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    // Our documents link to each other, and those links point at `.md` files
    // that are being converted alongside this one, so they are rewritten
    // below to point at the converted pages.
    let parser = Parser::new_ext(markdown, options);

    let mut body = String::new();
    html::push_html(&mut body, parser);
    let body = point_links_at_converted_pages(&body);

    // English, and stated rather than worked out from the machine. A help page
    // is English text this project wrote, so it is English whatever machine it
    // is read on. A message document is the other case and asks a different
    // question: see `language_attribute` in the HTML renderer.
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} - Wixen Mail</title>
<style>
:root {{ color-scheme: light dark; }}
body {{
  font-family: "Segoe UI", system-ui, sans-serif;
  font-size: 1rem;
  line-height: 1.6;
  /* In characters, so it still holds at any text size rather than squeezing
     the line into fewer words as somebody zooms in. */
  max-width: 70ch;
  margin: 0 auto;
  padding: 2rem 1.25rem 4rem;
}}
h1, h2, h3 {{ line-height: 1.25; margin-top: 2rem; }}
code, pre {{ font-family: Consolas, "Cascadia Mono", monospace; font-size: 0.95em; }}
pre {{ padding: 0.75rem; overflow-x: auto; }}
table {{ border-collapse: collapse; }}
th, td {{ border: 1px solid; padding: 0.4rem 0.6rem; text-align: left; }}
:focus-visible {{ outline: 3px solid; outline-offset: 2px; }}
</style>
</head>
<body>
{body}</body>
</html>
"#
    )
}

/// Rewrite `something.md` links so they open the converted page.
///
/// Only links to our own documents, which are the ones being converted. An
/// address with a scheme in it is somebody else's and is left alone, including
/// the unlikely case of a `.md` file on the web.
fn point_links_at_converted_pages(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(at) = rest.find("href=\"") {
        let (before, from_href) = rest.split_at(at + "href=\"".len());
        out.push_str(before);
        let Some(end) = from_href.find('"') else {
            rest = from_href;
            break;
        };
        let (target, after) = from_href.split_at(end);

        let (address, fragment) = match target.split_once('#') {
            Some((address, fragment)) => (address, Some(fragment)),
            None => (target, None),
        };

        if address.ends_with(".md") && !address.contains("://") {
            out.push_str(&address[..address.len() - "md".len()]);
            out.push_str("html");
            if let Some(fragment) = fragment {
                out.push('#');
                out.push_str(fragment);
            }
        } else {
            out.push_str(target);
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headings_survive_as_headings() {
        // The whole reason for converting. A screen reader user moves through
        // a long page by heading; in the Markdown source those are hash signs
        // read out as punctuation.
        let page = to_html("# Testing Wixen Mail\n\n## The short version\n", "Testing");

        assert!(page.contains("<h1>Testing Wixen Mail</h1>"));
        assert!(page.contains("<h2>The short version</h2>"));
    }

    #[test]
    fn test_the_page_says_what_language_it_is_in() {
        // Without it a screen reader reads English in whatever voice it was
        // left in, which for somebody whose system language is not English
        // makes the page unintelligible rather than merely wrong.
        let page = to_html("# Hello\n", "Hello");

        assert!(page.contains(r#"<html lang="en">"#));
    }

    #[test]
    fn test_the_title_comes_from_the_document() {
        // It is what the tab and the window announce, so "ALPHA_TESTING.md"
        // is not good enough.
        assert_eq!(
            title_of("# Testing Wixen Mail\n\nwords\n", "ALPHA_TESTING.md"),
            "Testing Wixen Mail"
        );
    }

    #[test]
    fn test_a_document_with_no_heading_falls_back_to_its_name() {
        assert_eq!(title_of("just words\n", "NOTES.md"), "NOTES.md");
    }

    #[test]
    fn test_links_between_our_documents_point_at_the_converted_pages() {
        // Otherwise following one lands back on raw Markdown, which is the
        // thing being fixed.
        let page = to_html("[installing](installing.md)\n", "Test");

        assert!(page.contains(r#"href="installing.html""#), "{page}");
    }

    #[test]
    fn test_a_link_to_a_heading_in_another_document_keeps_the_heading() {
        let page = to_html("[support](USER_GUIDE.md#screen-reader-support)\n", "Test");

        assert!(
            page.contains(r##"href="USER_GUIDE.html#screen-reader-support""##),
            "{page}"
        );
    }

    #[test]
    fn test_a_link_to_the_web_is_left_alone() {
        let page = to_html("[NVDA](https://www.nvaccess.org/)\n", "Test");

        assert!(
            page.contains(r#"href="https://www.nvaccess.org/""#),
            "{page}"
        );
    }

    #[test]
    fn test_tables_come_out_as_tables() {
        // The alpha testing page says what is allowed by default in a table,
        // and a table read as a row of pipe characters says nothing.
        let markdown = "| What | Default |\n|---|---|\n| Mail | Off |\n";

        let page = to_html(markdown, "Test");

        assert!(page.contains("<table>"), "{page}");
        assert!(page.contains("<th>What</th>"), "{page}");
    }

    #[test]
    fn test_every_link_in_a_converted_page_lands_on_a_converted_page() {
        // The bug this was written for is one step past the obvious one.
        // Converting only the page somebody asked for gives them a page whose
        // links have all been rewritten to `.html` files that were never
        // made, so every link is broken and the raw Markdown at least worked.
        let into = tempfile::tempdir().expect("somewhere to put them");
        let made = convert_into(Path::new("docs"), into.path()).expect("converting docs");
        assert!(made > 10, "only {made} documents converted");

        let mut broken = Vec::new();
        for page in std::fs::read_dir(into.path())
            .expect("the converted pages")
            .flatten()
        {
            let html = std::fs::read_to_string(page.path()).expect("a page");
            for target in links_in(&html) {
                if target.contains("://") || target.starts_with('#') {
                    continue;
                }
                let (file, _) = target.split_once('#').unwrap_or((target.as_str(), ""));
                if !into.path().join(file).exists() {
                    broken.push(format!("{:?} -> {file}", page.file_name()));
                }
            }
        }

        assert!(
            broken.is_empty(),
            "links with nothing behind them:\n  {}",
            broken.join("\n  ")
        );
    }

    /// Every `href` in a page, for the test above.
    #[cfg(test)]
    fn links_in(html: &str) -> Vec<String> {
        html.split("href=\"")
            .skip(1)
            .filter_map(|rest| rest.split_once('"').map(|(target, _)| target.to_string()))
            .collect()
    }

    #[test]
    fn test_the_pages_are_written_beside_the_documents_when_that_folder_takes_a_file() {
        // The folder that comes back is where the pages went, and the caller
        // joins the page name onto it. A folder that is not where they went is
        // a button that opens nothing, and the links between the pages only
        // resolve without rewriting because they sit beside their sources.
        let documents = tempfile::tempdir().expect("somewhere to put them");
        std::fs::write(documents.path().join("guide.md"), "# A guide\n").expect("a document");

        let into = convert_folder(documents.path()).expect("a folder that takes a file");

        assert_eq!(into, documents.path());
        assert!(documents.path().join("guide.html").exists());
    }

    #[test]
    fn test_the_pages_go_to_a_folder_of_our_own_when_the_program_s_folder_refuses() {
        // What an all-users install under Program Files looks like to somebody
        // who is not an administrator. Asked as a decision rather than by
        // arranging a folder Windows will refuse, which depends on the
        // machine's permissions, on elevation and on virtualisation.
        let program_files = Path::new(r"C:\Program Files\Wixen Mail\docs");

        let into = destination_for(program_files, false);

        assert_ne!(into, program_files);
        assert!(into.starts_with(std::env::temp_dir()), "{}", into.display());
    }

    #[test]
    fn test_the_real_testing_page_converts() {
        // The one the first-run screen offers, converted for real rather than
        // from a snippet written to suit the parser.
        let markdown =
            std::fs::read_to_string("docs/ALPHA_TESTING.md").expect("the alpha testing page");

        let page = to_html(&markdown, &title_of(&markdown, "ALPHA_TESTING.md"));

        assert!(page.contains("<title>Testing Wixen Mail - Wixen Mail</title>"));
        assert!(page.contains("<h2>"), "its sections should be headings");
        assert!(
            !page.contains(".md\""),
            "a link to raw Markdown survived: {page}"
        );
    }
}
