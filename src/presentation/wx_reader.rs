//! The window a message opens into.
//!
//! One window, tabs inside it, one message or one conversation per tab. The
//! reading surface is a read-only rich text control, not a WebView.
//!
//! That choice is the whole point. A native text control is focusable, moves by
//! character, word, line and paragraph with the arrow keys, supports selection
//! and copy, is searchable, exposes its caret position to a screen reader, and
//! gives focus back when you press Escape. A WebView does none of that reliably
//! once it has focus, which is how the preview pane came to trap people with no
//! way out but the system menu.
//!
//! The design follows Paperback (MIT, Quin Gillespie), an accessible document
//! reader built on the same wxWidgets bindings, which renders every format it
//! supports into a `MultiLine | ReadOnly | Rich2` text control inside a
//! `Notebook` and keeps its WebView for a separate optional dialog. It reached
//! that shape for the same reasons, and there was no sense in learning them
//! twice.

use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::reader_text::ReaderDocument;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

const ID_CLOSE_TAB: Id = ID_HIGHEST + 900;
const ID_NEXT_LANDMARK: Id = ID_HIGHEST + 901;
const ID_PREV_LANDMARK: Id = ID_HIGHEST + 902;
const ID_READER_FIND: Id = ID_HIGHEST + 903;

/// A reader window, kept alive for as long as it is open.
pub struct ReaderWindow {
    frame: Frame,
    notebook: Notebook,
    /// One entry per tab, in tab order.
    documents: Rc<RefCell<Vec<ReaderDocument>>>,
    /// Which tab is showing. Tracked here because the notebook reports its
    /// selection on the page-changed event and not on demand.
    current: Rc<std::cell::Cell<usize>>,
    a11y: Arc<Accessibility>,
}

/// Shorten a subject to something that works as a tab label.
///
/// A tab is announced every time focus reaches it, so a fifty word subject is
/// fifty words on every switch. The full subject is the first line of the
/// document, so nothing is lost by cutting it here.
pub fn tab_label(title: &str) -> String {
    const LIMIT: usize = 40;
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return "No subject".to_string();
    }
    if trimmed.chars().count() <= LIMIT {
        return trimmed.to_string();
    }
    let kept: String = trimmed.chars().take(LIMIT - 1).collect();
    // Cut on a word boundary where there is one nearby, so the label does not
    // end mid-word and get read as a nonsense syllable.
    match kept.rsplit_once(' ') {
        Some((head, _)) if head.chars().count() >= LIMIT / 2 => format!("{}\u{2026}", head),
        _ => format!("{}\u{2026}", kept.trim_end()),
    }
}

impl ReaderWindow {
    /// Create the window. It is not shown until a document is added.
    pub fn new(parent: &Frame, a11y: &Arc<Accessibility>) -> Self {
        let frame = Frame::builder()
            .with_parent(parent)
            .with_title("Reading - Wixen Mail")
            .with_size(Size::new(900, 700))
            .build();

        let notebook = Notebook::builder(&frame).build();
        set_accessible_name(&notebook, "Open messages");

        // A menu bar rather than bare accelerators, because a menu is
        // discoverable: someone who cannot see the window can walk it and find
        // out what the window does instead of having to be told.
        let file = Menu::builder()
            .append_item(ID_CLOSE_TAB, "&Close Tab\tCtrl+W", "Close this message")
            .append_item(ID_CANCEL, "Close &Window\tEsc", "Close the reader window")
            .build();
        let go = Menu::builder()
            .append_item(
                ID_NEXT_LANDMARK,
                "&Next Message\tCtrl+Down",
                "Move to the next message in this conversation",
            )
            .append_item(
                ID_PREV_LANDMARK,
                "&Previous Message\tCtrl+Up",
                "Move to the previous message in this conversation",
            )
            .append_separator()
            .append_item(ID_READER_FIND, "&Find\tCtrl+F", "Find text in this message")
            .build();
        let menu_bar = MenuBar::builder()
            .append(file, "&File")
            .append(go, "&Go")
            .build();
        frame.set_menu_bar(menu_bar);

        let current = Rc::new(std::cell::Cell::new(0usize));
        notebook.on_page_changed({
            let current = current.clone();
            move |event| {
                if let Some(page) = event.get_selection()
                    && page >= 0
                {
                    current.set(page as usize);
                }
            }
        });

        Self {
            frame,
            notebook,
            documents: Rc::new(RefCell::new(Vec::new())),
            current,
            a11y: a11y.clone(),
        }
    }

    /// Add a document as a new tab and show the window.
    pub fn open(&self, document: ReaderDocument) {
        let panel = Panel::builder(&self.notebook).build();
        let sizer = BoxSizer::builder(Orientation::Vertical).build();

        // Rich2 because the plain multiline control on Windows has a text
        // length limit that a long conversation reaches, and because it is the
        // control a screen reader reports a caret position for.
        let text = TextCtrl::builder(&panel)
            .with_style(
                TextCtrlStyle::MultiLine
                    | TextCtrlStyle::ReadOnly
                    | TextCtrlStyle::Rich2
                    | TextCtrlStyle::WordWrap,
            )
            .build();
        set_accessible_name(&text, &document.title);
        text.set_value(&document.text);
        // The caret starts at the top: a control that opens with the caret at
        // the end reads the last line first, which is not where a message
        // begins.
        text.set_insertion_point(0);
        sizer.add(&text, 1, SizerFlag::Expand | SizerFlag::All, 4);
        panel.set_sizer(sizer, true);

        let label = tab_label(&document.title);
        self.notebook.add_page(&panel, &label, true, None);
        self.documents.borrow_mut().push(document.clone());

        let index = self.documents.borrow().len().saturating_sub(1);
        self.current.set(index);
        self.wire_tab(&text, index);

        self.frame.show(true);
        self.frame.raise();
        text.set_focus();

        let _ = self.a11y.announce(
            &format!("{}, reading. Escape closes.", document.title),
            Priority::Normal,
        );
    }

    /// Give one tab's text control its keys.
    fn wire_tab(&self, text: &TextCtrl, index: usize) {
        let documents = self.documents.clone();
        let a11y = self.a11y.clone();
        let frame = self.frame;
        let control = *text;

        control.bind_internal(EventType::KEY_DOWN, move |event| {
            event.skip(true);
            let Some(key) = event.get_key_code() else {
                return;
            };
            // Escape leaves. In a text control this is ours to define, and the
            // control is a real window, so the key actually arrives.
            if key == 27 {
                frame.show(false);
                return;
            }
            if !event.control_down() {
                return;
            }
            // Ctrl+Up and Ctrl+Down move between the messages of a
            // conversation. Reading through to find the next one is exactly
            // what a landmark exists to avoid.
            let forwards = match key {
                // WXK_UP, WXK_DOWN
                315 => false,
                317 => true,
                _ => return,
            };
            let documents = documents.borrow();
            let Some(document) = documents.get(index) else {
                return;
            };
            let caret = control.get_insertion_point().max(0) as usize;
            let target = if forwards {
                document.next_landmark(caret)
            } else {
                document.previous_landmark(caret)
            };
            match target {
                Some(landmark) => {
                    control.set_insertion_point(landmark.offset as i64);
                    control.show_position(landmark.offset as i64);
                    let _ = a11y.announce(&landmark.label, Priority::Normal);
                }
                None => {
                    // Said rather than done silently: nothing happening is
                    // indistinguishable from a key that does not work.
                    let _ = a11y.announce(
                        if forwards {
                            "Last message"
                        } else {
                            "First message"
                        },
                        Priority::Normal,
                    );
                }
            }
        });
    }

    /// Whether the window is currently on screen.
    pub fn is_open(&self) -> bool {
        self.frame.is_shown()
    }

    /// Wire the window's menu. Called once, after construction.
    pub fn wire_menu(&self) {
        let frame = self.frame;
        let notebook = self.notebook;
        let documents = self.documents.clone();
        let current = self.current.clone();
        let a11y = self.a11y.clone();

        frame.on_menu(move |event| {
            let id = event.get_id();
            if id == ID_CANCEL {
                frame.show(false);
                return;
            }
            if id == ID_CLOSE_TAB {
                let page = current.get();
                if page >= notebook.get_page_count() {
                    return;
                }
                notebook.remove_page(page);
                let mut open = documents.borrow_mut();
                if page < open.len() {
                    open.remove(page);
                }
                if open.is_empty() {
                    // An empty tabbed window is a window with nothing in it and
                    // no way to tell what it is for.
                    drop(open);
                    frame.show(false);
                    let _ = a11y.announce("Reader closed", Priority::Normal);
                } else {
                    let _ = a11y.announce(
                        &format!("Tab closed, {} left", open.len()),
                        Priority::Normal,
                    );
                }
            }
        });

        // Closing the window hides it rather than destroying it, so the same
        // window is reused. Destroying it would invalidate every handle held
        // here and take the application down with the next message opened.
        frame.on_close(move |event| {
            if let WindowEventData::General(ref base) = event {
                base.veto();
            }
            frame.show(false);
        });
    }

    /// Bring an already open window back to the front.
    pub fn focus(&self) {
        self.frame.show(true);
        self.frame.raise();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_long_subject_is_cut_for_the_tab_label() {
        // A tab is announced on every switch, so a fifty word subject costs
        // fifty words each time. The full subject is the document's first line.
        let long = "Quarterly report for the financial year including all regional breakdowns";
        let label = tab_label(long);
        assert!(label.chars().count() <= 40, "label too long: {}", label);
        assert!(label.ends_with('\u{2026}'));
        assert!(long.starts_with(label.trim_end_matches('\u{2026}').trim_end()));
    }

    #[test]
    fn test_a_cut_label_does_not_end_mid_word() {
        // Half a word read aloud is a nonsense syllable.
        let label = tab_label("Quarterly report for the financial year including breakdowns");
        let body = label.trim_end_matches('\u{2026}');
        assert!(!body.ends_with(' '));
        assert!(
            "Quarterly report for the financial year including breakdowns"
                .split_whitespace()
                .any(|w| body.ends_with(w)),
            "cut mid word: {}",
            label
        );
    }

    #[test]
    fn test_a_short_subject_is_left_alone() {
        assert_eq!(tab_label("Invoice 4021"), "Invoice 4021");
        assert_eq!(tab_label("  Invoice 4021  "), "Invoice 4021");
    }

    #[test]
    fn test_an_empty_subject_gets_a_label_rather_than_a_blank_tab() {
        // A blank tab cannot be identified or returned to.
        assert_eq!(tab_label(""), "No subject");
        assert_eq!(tab_label("   "), "No subject");
    }

    #[test]
    fn test_a_long_single_word_subject_is_still_cut() {
        // No space to cut on, so it cuts anyway rather than returning
        // something over the limit.
        let label = tab_label(&"a".repeat(100));
        assert!(label.chars().count() <= 40);
    }
}
