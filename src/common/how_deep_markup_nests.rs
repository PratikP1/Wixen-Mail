//! How deep a stranger's markup nests, asked without recursion.
//!
//! Every reader that turns a message's HTML into words walks the parsed tree
//! by calling itself once per level, and nothing bounds the levels: html5ever
//! builds a tree as deep as the sender writes it. A body of twenty thousand
//! nested `div` elements passes every filter and overflows the window's 1 MB
//! stack, which is not a panic but the end of the process, with no log line,
//! every time the message is selected (audit finding #17 of 2026-10-14).
//!
//! So the depth is asked once, right after each parse whose tree a recursive
//! walker reads: in the vendored `HtmlToText::convert` and in `long_text`'s
//! reader. `hidden_text::drop_what_the_sender_hid` asked it too until
//! 2026-10-22, and handed deeper markup on with what its sender hid still in
//! it; its walk holds one edge at a time since 13.3-19, so it reads markup of
//! any depth and asks nothing. The question walks
//! [`ego_tree::NodeRef::traverse`], an iterator holding one edge at a time,
//! so asking it costs no stack however deep the answer is. Markup deeper than
//! the limit is read as its words alone, gathered the same way.

use ego_tree::{NodeRef, iter::Edge};
use scraper::Node;

/// The deepest markup the readers below a parse are handed, counted in
/// elements from the top of the parsed tree, the parser's own `html` and
/// `body` included.
///
/// Browsers stop nesting at 512, so no layout a sender sees on their own
/// screen is deeper than this, and it is measured to fit. The window's
/// thread has Windows' default 1 MB stack, the smallest this program runs
/// on, and a debug build's frames are larger than a release build's. On
/// 2026-10-15 at `2260eb58`, the cases in `vendor::paperback` and
/// `application::long_text` that walk markup exactly this deep, as `div`,
/// inside an `h1` and inside an `li`, on a 1 MiB thread in a debug build:
/// 512 and 768 walked whole, and 1024 overflowed the stack in the converter
/// and in the reader's `div` case. The row is on
/// `docs/development/measurements.md`.
pub const DEEPEST_MARKUP_WALKED: usize = 512;

/// Whether the markup under `root` nests more than `limit` elements deep.
///
/// Stops at the first element past the limit, so a tree a hundred thousand
/// deep costs no more than one just past it.
pub fn nests_deeper_than(root: NodeRef<'_, Node>, limit: usize) -> bool {
    let mut depth = 0usize;
    for edge in root.traverse() {
        match edge {
            Edge::Open(node) if node.value().is_element() => {
                depth += 1;
                if depth > limit {
                    return true;
                }
            }
            Edge::Close(node) if node.value().is_element() => depth -= 1,
            _ => {}
        }
    }
    false
}

/// The words of markup too deep to walk, one entry per text node in the
/// order they stand, each with its runs of space made one.
///
/// What no reader is shown stays out: a stylesheet, a script, the page's
/// head and title, and the content a page never draws, the same elements
/// the readers leave out when they can walk the page.
pub fn the_words_alone(root: NodeRef<'_, Node>) -> Vec<String> {
    let mut words = Vec::new();
    let mut inside_what_is_never_read = 0usize;
    for edge in root.traverse() {
        match edge {
            Edge::Open(node) if is_never_read(node) => inside_what_is_never_read += 1,
            Edge::Close(node) if is_never_read(node) => inside_what_is_never_read -= 1,
            Edge::Open(node) if inside_what_is_never_read == 0 => {
                if let Some(text) = node.value().as_text() {
                    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !line.is_empty() {
                        words.push(line);
                    }
                }
            }
            _ => {}
        }
    }
    words
}

/// Whether a node is an element whose content no reader is shown.
fn is_never_read(node: NodeRef<'_, Node>) -> bool {
    node.value().as_element().is_some_and(|element| {
        matches!(
            element.name(),
            "head"
                | "title"
                | "style"
                | "script"
                | "noscript"
                | "template"
                | "iframe"
                | "object"
                | "embed"
        )
    })
}

/// Markup nested to `depth` elements of `tag`, around `words`, for the cases
/// that feed a reader markup of a known depth.
#[cfg(test)]
pub fn nested(tag: &str, depth: usize, words: &str) -> String {
    format!(
        "{}{words}{}",
        format!("<{tag}>").repeat(depth),
        format!("</{tag}>").repeat(depth)
    )
}

/// Run `work` on a thread of its own with a stack of `bytes`, and hand back
/// what it returned.
///
/// A stack overflow ends the whole test program rather than failing one
/// test, so a case that walks deep markup on a stack big enough for the
/// walk fails on its assertion instead, and a case proving a small stack is
/// enough says how small.
#[cfg(test)]
pub fn on_a_stack_of<T: Send + 'static>(
    bytes: usize,
    work: impl FnOnce() -> T + Send + 'static,
) -> T {
    std::thread::Builder::new()
        .stack_size(bytes)
        .spawn(work)
        .expect("a thread for the case")
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// Run one case of this test program in a process of its own, and fail
/// unless it ran and passed.
///
/// For a case whose failure is a stack overflow. Run in the program with
/// every other case, an overflow ends the program, every case after it never
/// runs, and nothing names the one that failed; run apart, it is this case's
/// exit status, said in this case's failure. The count of passed cases is
/// read as well, because a name that matches nothing also exits cleanly.
///
/// The child's own `test <name> ... FAILED` line is left out of the failure,
/// because the guard runner and the red gate read such lines from the whole
/// run, and would take it for a second failing case.
#[cfg(test)]
pub fn passes_in_a_process_of_its_own(case: &str) {
    let ended =
        std::process::Command::new(std::env::current_exe().expect("this test program's path"))
            .args(["--exact", case, "--ignored", "--test-threads=1"])
            .output()
            .expect("this test program, started again");
    let said = String::from_utf8_lossy(&ended.stdout);
    let complained = String::from_utf8_lossy(&ended.stderr);
    let without_result_lines: Vec<&str> = said
        .lines()
        .filter(|line| !line.starts_with("test "))
        .collect();
    assert!(
        ended.status.success() && said.contains("1 passed"),
        "{case} ended with {}:\n{}\n{complained}",
        ended.status,
        without_result_lines.join("\n")
    );
}

/// The audit's message: `div` elements nested this deep around a few words.
///
/// Twice the shallowest round depth that ended the program on a 1 MiB stack
/// before the depth check, so a case that reads it would have crashed with
/// room to spare. Measured on 2026-10-15 by 13.2-12 in a debug build, with
/// the check made never to answer yes: the preview overflowed at 1,000
/// levels, and the HTML-only save passed at 1,000 and overflowed at 2,000.
/// At 4,000 both overflowed. The audit's own 20,000 was not used, because
/// html5ever's parse of nested `div` elements is quadratic in their depth
/// and the case runs several parses.
#[cfg(test)]
pub fn the_audits_message() -> String {
    nested("div", 4_000, "Deep words")
}

/// A stack big enough for any walk the unfixed readers make over the cases'
/// markup.
#[cfg(test)]
pub const ROOM_FOR_ANY_WALK: usize = 64 * 1024 * 1024;

/// The stack of the window's own thread, the smallest the program runs on.
#[cfg(test)]
pub const THE_WINDOWS_STACK: usize = 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn deeper_than(markup: &str, limit: usize) -> bool {
        let parsed = Html::parse_fragment(markup);
        nests_deeper_than(parsed.tree.root(), limit)
    }

    #[test]
    fn test_markup_exactly_as_deep_as_the_limit_is_not_deeper_than_it() {
        // `parse_fragment` puts the markup under an `html` element of its
        // own, so four `span` elements are five deep.
        assert!(!deeper_than(&nested("span", 4, "words"), 5));
    }

    #[test]
    fn test_markup_one_level_past_the_limit_is_deeper_than_it() {
        assert!(deeper_than(&nested("span", 5, "words"), 5));
    }

    #[test]
    fn test_siblings_side_by_side_are_not_counted_as_depth() {
        // Ten paragraphs one after another are two deep, not eleven: the
        // count goes down again as each element closes.
        assert!(!deeper_than(&"<p>one</p>".repeat(10), 2));
        assert!(deeper_than(&"<p><b>one</b></p>".repeat(10), 2));
    }

    #[test]
    fn test_a_hundred_thousand_levels_are_answered_on_the_windows_stack() {
        // `span` rather than `div`: html5ever looks back along the open
        // elements for a `p` on every `div`, which is quadratic, and a
        // `span` asks it nothing, so the parse stays quick and the case is
        // about the question.
        let answered = on_a_stack_of(THE_WINDOWS_STACK, || {
            deeper_than(&nested("span", 100_000, "words"), DEEPEST_MARKUP_WALKED)
        });
        assert!(answered);
    }
}
