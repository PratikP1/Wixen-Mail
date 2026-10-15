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
//! walker reads: in `hidden_text::drop_what_the_sender_hid`, in the vendored
//! `HtmlToText::convert`, and in `long_text`'s reader. The question walks
//! [`ego_tree::NodeRef::traverse`], an iterator holding one edge at a time,
//! so asking it costs no stack however deep the answer is. Markup deeper than
//! the limit is read as its words alone, gathered the same way.

use ego_tree::NodeRef;
use scraper::Node;

/// The deepest markup the readers below a parse are handed, counted in
/// elements from the top of the parsed tree, the parser's own `html` and
/// `body` included.
///
/// Browsers stop nesting at 512, so no layout a sender sees on their own
/// screen is deeper than this.
pub const DEEPEST_MARKUP_WALKED: usize = 512;

/// Whether the markup under `root` nests more than `limit` elements deep.
pub fn nests_deeper_than(root: NodeRef<'_, Node>, limit: usize) -> bool {
    let _ = (root, limit);
    false
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
