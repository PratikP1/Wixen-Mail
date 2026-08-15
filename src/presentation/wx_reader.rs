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
use crate::presentation::accessibility::feedback::Event as FeedbackEvent;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::reader_text::{ReaderAttachment, ReaderDocument};
use crate::presentation::theme;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

const ID_CLOSE_TAB: Id = ID_HIGHEST + 900;
const ID_NEXT_LANDMARK: Id = ID_HIGHEST + 901;
const ID_PREV_LANDMARK: Id = ID_HIGHEST + 902;
const ID_READER_FIND: Id = ID_HIGHEST + 903;
const ID_SAVE_ATTACHMENT: Id = ID_HIGHEST + 904;
const ID_GO_ATTACHMENTS: Id = ID_HIGHEST + 905;
const ID_READ_ATTACHMENT: Id = ID_HIGHEST + 906;

/// What the window does when somebody asks to save an attachment.
///
/// Set by the application rather than done here. Saving means fetching the
/// message again, which needs the account, the credentials and the runtime,
/// and none of those belong to a window whose job is to show text.
type SaveHandler = Box<dyn Fn(&ReaderAttachment)>;

/// What the window does when somebody asks to read an attachment here.
///
/// Set by the application for the same reason as the save handler: reading one
/// means fetching the message again first.
type ReadHandler = Box<dyn Fn(&ReaderAttachment)>;

/// A reader window, kept alive for as long as it is open.
/// Where closing a reading window goes back to, if anywhere.
///
/// `None` means the window it was opened over, which is the mailbox. Set when
/// the window was opened from somewhere a person should return to, such as a
/// conversation, and taken out of the cell before it runs so closing twice
/// cannot open two.
pub type GoneBack = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

pub struct ReaderWindow {
    frame: Frame,
    notebook: Notebook,
    /// One entry per tab, in tab order.
    documents: Rc<RefCell<Vec<ReaderDocument>>>,
    /// Which tab is showing. Tracked here because the notebook reports its
    /// selection on the page-changed event and not on demand.
    current: Rc<std::cell::Cell<usize>>,
    /// The attachment list of each tab, so a Save command can find out which
    /// row is chosen without walking the window's children.
    attachment_lists: Rc<RefCell<Vec<Option<ListBox>>>>,
    save_attachment: Rc<RefCell<Option<SaveHandler>>>,
    read_attachment: Rc<RefCell<Option<ReadHandler>>>,
    /// What to do when this window is closed, if anything.
    ///
    /// Set when the reader was opened from somewhere a person should come back
    /// to. Escape out of a message opened from a conversation belongs back in
    /// the conversation, not in the mailbox two levels up: going up two at once
    /// loses their place, and for somebody navigating by ear their place is the
    /// only thing telling them where they are.
    ///
    /// Cleared whenever the reader is opened from somewhere else, or closing a
    /// message would keep reopening the conversation it once came from.
    closed: GoneBack,
    /// Runs the callback above, once, after the close has finished.
    ///
    /// A one-shot timer rather than a direct call. What the callback opens is a
    /// modal dialog, and showing one from inside the close handler of the
    /// window it came from means starting a message loop inside the one that is
    /// still finishing. The timer fires on the same thread a moment later, with
    /// the close over and done with.
    go_back: Rc<Timer<Frame>>,
    a11y: Arc<Accessibility>,
    /// The palette this window's tabs paint themselves with, read once at
    /// construction the same way the main window reads it once at startup,
    /// and replaced by [`Self::repaint`] whenever the Theme setting changes
    /// while this window is already built. `None` under Windows high
    /// contrast, or when the system is set up in a way nothing here has an
    /// opinion about; either way nothing is painted and Windows keeps
    /// deciding.
    ///
    /// A `Cell` rather than a plain field: every method here takes `&self`,
    /// because the window is shared as `Rc<ReaderWindow>` and reused for as
    /// long as Wixen Mail runs, so there is no `&mut self` to update it
    /// through.
    palette: Cell<Option<theme::Palette>>,
}

/// Handles to the widgets one open tab is made of.
///
/// Returned by [`ReaderWindow::open`] so a test can reach and read back what
/// each one was painted, the same way every other handle in this file is
/// reached. Production code has never needed them before, because the window
/// keeps its own records of what it built; they exist now only so live
/// controls are reachable from outside this module.
pub struct ReaderTabHandles {
    pub panel: Panel,
    pub text: TextCtrl,
    /// `None` for an ordinary message, which has no warning bar at all.
    pub warning: Option<TextCtrl>,
    /// `None` for a message with nothing attached, which has no list at all.
    pub attachments: Option<ListBox>,
}

/// Hand one attachment to whatever the application said to do with it.
///
/// Every way of failing says something. A command that reports nothing leaves
/// somebody pressing the key again to find out whether it worked, and the
/// answer both times is silence.
fn save_chosen(
    documents: &Rc<RefCell<Vec<ReaderDocument>>>,
    handler: &Rc<RefCell<Option<SaveHandler>>>,
    a11y: &Arc<Accessibility>,
    page: usize,
    chosen: Option<u32>,
    what: Doing,
) {
    let documents = documents.borrow();
    let Some(document) = documents.get(page) else {
        return;
    };
    if document.attachments.is_empty() {
        let _ = a11y.announce("This message has no attachments", Priority::Normal);
        return;
    }
    // No selection means the list was never reached, so the first row is the
    // one meant. A list with a selection is asking about that row.
    let index = chosen.unwrap_or(0) as usize;
    let Some(attachment) = document.attachments.get(index) else {
        let _ = a11y.announce(
            &crate::application::pim_command::something_no_longer_there("attachment", ""),
            Priority::Normal,
        );
        return;
    };
    if what == Doing::Reading && !can_be_read_here(attachment) {
        // Named rather than a shrug. "Cannot open" leaves somebody wondering
        // what would have worked; this says what to do instead.
        let _ = a11y.announce(
            &format!(
                "Wixen Mail cannot read a {}. Control S saves it.",
                describe_for_refusal(attachment)
            ),
            Priority::High,
        );
        return;
    }
    match handler.borrow().as_ref() {
        Some(act) => act(attachment),
        None => {
            let _ = a11y.announce(
                match what {
                    Doing::Saving => "Saving attachments is not available",
                    Doing::Reading => "Reading attachments is not available",
                },
                Priority::High,
            );
        }
    }
}

/// Which of the two things is being asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Doing {
    Saving,
    Reading,
}

/// Whether this is something the reader can turn into text.
///
/// PDF, and nothing else yet. A message body arrives as text or HTML and is
/// already handled; everything else is a file for another application.
fn can_be_read_here(attachment: &ReaderAttachment) -> bool {
    attachment
        .mime_type
        .trim()
        .eq_ignore_ascii_case("application/pdf")
        || attachment
            .name
            .rsplit_once('.')
            .is_some_and(|(_, extension)| extension.trim().eq_ignore_ascii_case("pdf"))
}

/// What to call the thing that cannot be read, for the sentence that says so.
fn describe_for_refusal(attachment: &ReaderAttachment) -> String {
    match attachment.name.rsplit_once('.') {
        Some((_, extension)) if !extension.trim().is_empty() => {
            format!("{} file", extension.trim().to_uppercase())
        }
        _ => "file of this kind".to_string(),
    }
}

/// What the opening announcement says about the attachments, if anything.
///
/// Said on the way in because the list is at the bottom of the tab order, and
/// somebody who reads the message and closes it would otherwise never learn it
/// was there. It names the key, since a feature nobody can find is a feature
/// nobody has.
///
/// A program among them is called out by name. It is the one fact that should
/// change what somebody does next, and it is exactly the fact a message
/// pretending to be an invoice is relying on them not noticing.
fn attachment_summary(attachments: &[ReaderAttachment]) -> String {
    if attachments.is_empty() {
        return String::new();
    }
    let runnable = attachments.iter().filter(|a| a.is_runnable()).count();
    let count = match attachments.len() {
        1 => "1 attachment".to_string(),
        many => format!("{many} attachments"),
    };
    let programs = match (runnable, attachments.len()) {
        (0, _) => String::new(),
        (1, 1) => ", which is a program".to_string(),
        (1, _) => ", one of them a program".to_string(),
        (many, _) => format!(", {many} of them programs"),
    };
    format!(" {count}{programs}, F8 for the list.")
}

/// Shorten a subject to something that works as a tab label.
///
/// A tab is announced every time focus reaches it, so a fifty word subject is
/// fifty words on every switch. The full subject is the first line of the
/// document, so nothing is lost by cutting it here.
fn tab_label(title: &str) -> String {
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

        // Read once here rather than per tab, the same reason the main
        // window reads it once at startup rather than per control: a
        // settings file opened once per widget is a file opened a hundred
        // times over a long reading session. This window is built once and
        // reused for as long as Wixen Mail runs, so "once" here means once
        // per run, the same as everywhere else the palette is read.
        let palette = theme::current(
            &crate::data::config::ConfigManager::load_stored()
                .map(|mgr| mgr.app_config().theme.clone())
                .unwrap_or_default(),
        );
        if let Some(palette) = palette {
            theme::paint(&frame, palette.main_surface());
        }

        let notebook = Notebook::builder(&frame).build();
        set_accessible_name(&notebook, "Open messages");
        if let Some(palette) = palette {
            theme::paint(&notebook, palette.main_surface());
        }

        // A menu bar rather than bare accelerators, because a menu is
        // discoverable: someone who cannot see the window can walk it and find
        // out what the window does instead of having to be told.
        let file = Menu::builder()
            .append_item(
                ID_READ_ATTACHMENT,
                "&Read Attachment	Ctrl+O",
                "Read the chosen attachment in a tab of its own",
            )
            .append_item(
                ID_SAVE_ATTACHMENT,
                "&Save Attachment...\tCtrl+S",
                "Save the chosen attachment to a file",
            )
            .append_separator()
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
            .append_item(
                ID_GO_ATTACHMENTS,
                "&Attachments\tF8",
                "Move to the list of attachments",
            )
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

        let closed: GoneBack = Rc::new(RefCell::new(None));
        let go_back = Rc::new(Timer::new(&frame));
        go_back.on_tick({
            let closed = closed.clone();
            move |_| {
                // Taken out before it runs, so closing twice cannot open two.
                let back_to = closed.borrow_mut().take();
                if let Some(back_to) = back_to {
                    back_to();
                }
            }
        });

        Self {
            frame,
            notebook,
            documents: Rc::new(RefCell::new(Vec::new())),
            current,
            attachment_lists: Rc::new(RefCell::new(Vec::new())),
            save_attachment: Rc::new(RefCell::new(None)),
            read_attachment: Rc::new(RefCell::new(None)),
            closed,
            go_back,
            a11y: a11y.clone(),
            palette: Cell::new(palette),
        }
    }

    /// The window itself, for reaching what it was painted.
    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// The tab strip, for reaching what it was painted.
    pub fn notebook(&self) -> &Notebook {
        &self.notebook
    }

    /// Re-apply the theme immediately when the Theme setting changes while
    /// this window is already built, using the same [`theme::paint`] call
    /// [`Self::new`] makes once at startup.
    ///
    /// This window's own frame and tab strip repaint on the spot. Any tab
    /// opened after this call paints itself with the new palette too,
    /// because this also replaces the palette [`Self::open`] reads from.
    ///
    /// A tab already open when this runs keeps the colours it was built
    /// with. Repainting its message text, warning bar and attachment list
    /// would mean this window keeping hold of them past the moment they are
    /// built, which today it does not: [`Self::open`] hands its widgets back
    /// to the caller and keeps none of them except the attachment list,
    /// for reasons its own doc comment gives. Reopening the message, or the
    /// next restart, is what picks up the new colour there.
    pub fn repaint(&self, palette: Option<theme::Palette>) {
        self.palette.set(palette);
        let Some(palette) = palette else {
            return;
        };
        theme::paint(&self.frame, palette.main_surface());
        theme::paint(&self.notebook, palette.main_surface());
    }

    /// Say what to do when somebody asks to save an attachment.
    ///
    /// Until this is called the Save command reports that saving is not
    /// available, rather than doing nothing and leaving somebody to work out
    /// whether they pressed the key.
    pub fn on_save_attachment(&self, handler: impl Fn(&ReaderAttachment) + 'static) {
        *self.save_attachment.borrow_mut() = Some(Box::new(handler));
    }

    /// Say what to do when somebody asks to read an attachment here.
    pub fn on_read_attachment(&self, handler: impl Fn(&ReaderAttachment) + 'static) {
        *self.read_attachment.borrow_mut() = Some(Box::new(handler));
    }

    /// Add a document as a new tab and show the window.
    ///
    /// Returns handles to the tab's own widgets. Production code has no use
    /// for them, since the window keeps its own records of what it built;
    /// they exist so a test can reach a live control and read back what it
    /// was painted.
    pub fn open(&self, document: ReaderDocument) -> ReaderTabHandles {
        let panel = Panel::builder(&self.notebook).build();
        if let Some(palette) = self.palette.get() {
            theme::paint(&panel, palette.main_surface());
        }
        let sizer = BoxSizer::builder(Orientation::Vertical).build();

        // The warning goes above the message and therefore first in the tab
        // order, so it is met on the way in rather than found afterwards. A
        // sighted reader gets a coloured strip without asking for it; this is
        // the equivalent that does not depend on catching an announcement as it
        // goes past, because an announcement cannot be replayed and this can be
        // read as many times as somebody likes.
        //
        // It exists only when there is something to say. An empty bar in the
        // tab order of every ordinary message is a stop on the way to the text
        // that teaches people to tab straight past the one that matters.
        let warning = document.warning.as_ref().map(|text| {
            let bar = TextCtrl::builder(&panel)
                .with_style(TextCtrlStyle::ReadOnly | TextCtrlStyle::MultiLine)
                .build();
            bar.set_value(text);
            // Named for what it is, not where it is. "Security warning" says
            // why to stop; "Notification bar" says there is furniture here.
            set_accessible_name(&bar, "Security warning");
            bar.set_insertion_point(0);
            sizer.add(&bar, 0, SizerFlag::Expand | SizerFlag::All, 4);
            // Plain main surface, the same as the rest of the tab. `Palette`
            // has no colour of its own for a warning role today, only a text
            // colour tested against the two existing surfaces, and inventing
            // a background for one is a design decision this round does not
            // make.
            if let Some(palette) = self.palette.get() {
                theme::paint(&bar, palette.main_surface());
            }
            bar
        });

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
        if let Some(palette) = self.palette.get() {
            theme::paint(&text, palette.main_surface());
        }
        text.set_value(&document.text);
        // The caret starts at the top: a control that opens with the caret at
        // the end reads the last line first, which is not where a message
        // begins.
        text.set_insertion_point(0);
        sizer.add(&text, 1, SizerFlag::Expand | SizerFlag::All, 4);

        // Below the message and therefore after it in the tab order, because
        // an attachment is something you deal with once you know what the
        // message says. It exists only when there is something in it: an empty
        // list on every ordinary message is another stop to tab past, and a
        // control that is usually empty is one people stop checking.
        let attachments = (!document.attachments.is_empty()).then(|| {
            let list = ListBox::builder(&panel).build();
            for attachment in &document.attachments {
                list.append(&attachment.label());
            }
            // The count is in the name because a list announces its name
            // before its first row, so "Attachments, 2 items" arrives before
            // anything has to be read to find that out. The word "program" is
            // in the name too when one of them is, since that is the thing
            // somebody needs to hear before they start opening files a
            // stranger sent them.
            let runnable = document
                .attachments
                .iter()
                .filter(|a| a.is_runnable())
                .count();
            let name = match (document.attachments.len(), runnable) {
                (1, 0) => "Attachments, 1 item".to_string(),
                (count, 0) => format!("Attachments, {count} items"),
                (1, _) => "Attachments, 1 item, a program".to_string(),
                (count, 1) => format!("Attachments, {count} items, one is a program"),
                (count, many) => format!("Attachments, {count} items, {many} are programs"),
            };
            set_accessible_name(&list, &name);
            if let Some(palette) = self.palette.get() {
                theme::paint(&list, palette.main_surface());
            }
            list.set_selection(0, true);
            sizer.add(&list, 0, SizerFlag::Expand | SizerFlag::All, 4);
            list
        });

        panel.set_sizer(sizer, true);

        let label = tab_label(&document.title);
        self.notebook.add_page(&panel, &label, true, None);
        self.documents.borrow_mut().push(document.clone());
        self.attachment_lists.borrow_mut().push(attachments);

        let index = self.documents.borrow().len().saturating_sub(1);
        self.current.set(index);
        self.wire_tab(&text, index);

        self.frame.show(true);
        self.frame.raise();
        // Focus starts in the message, not in the bar. Landing in the warning
        // would mean pressing a key to get to the mail every time, and the
        // warning is announced anyway. Shift+Tab and F7 both reach it.
        text.set_focus();

        let _ = self.a11y.announce(
            &format!(
                "{}, reading.{} Escape closes.",
                document.title,
                attachment_summary(&document.attachments)
            ),
            Priority::Normal,
        );
        // Through `signal` rather than `announce`, so it obeys the feedback
        // settings like every other event and can be switched off or moved to
        // a tone by somebody who reads their junk folder on purpose.
        if let Some(warning) = document.warning.as_deref() {
            let _ = self.a11y.signal(FeedbackEvent::UnsafeMessage, warning);
        }

        if let Some(bar) = warning {
            self.wire_warning_jump(&text, bar);
        }
        if let Some(list) = attachments {
            self.wire_attachment_list(list, index);
        }

        ReaderTabHandles {
            panel,
            text,
            warning,
            attachments,
        }
    }

    /// Enter on an attachment row saves it, and F8 goes back to the message.
    ///
    /// Enter because that is what pressing Enter on a row means everywhere
    /// else, and a list where the obvious key does nothing reads as broken.
    /// F8 both ways for the same reason F7 goes both ways on the warning bar:
    /// somewhere to jump to is only useful with a way back.
    fn wire_attachment_list(&self, list: ListBox, index: usize) {
        let documents = self.documents.clone();
        let save = self.save_attachment.clone();
        let read = self.read_attachment.clone();
        let a11y = self.a11y.clone();
        let notebook = self.notebook;

        list.bind_internal(EventType::KEY_DOWN, move |event| {
            event.skip(true);
            match event.get_key_code() {
                // WXK_RETURN and the numeric keypad's Enter. Reading is what
                // Enter does, because reading it here is what this
                // application is for and saving has its own key. A file that
                // cannot be read says so and names the key that would work.
                Some(13) | Some(370) => {
                    save_chosen(
                        &documents,
                        &read,
                        &a11y,
                        index,
                        list.get_selection(),
                        Doing::Reading,
                    );
                }
                // Ctrl+S reaches the same command as the menu, from the list.
                Some(83) if event.control_down() => {
                    save_chosen(
                        &documents,
                        &save,
                        &a11y,
                        index,
                        list.get_selection(),
                        Doing::Saving,
                    );
                }
                // WXK_F8 goes back to the message, which is the first control
                // on the page after any warning bar.
                Some(347) => {
                    if let Some(page) = notebook.get_page(index) {
                        page.set_focus();
                        let _ = a11y.announce("Message", Priority::Normal);
                    }
                }
                _ => {}
            }
        });
    }

    /// F7 moves between the message and the warning above it.
    ///
    /// A key rather than only the tab order, because the bar is one stop away
    /// when you are at the top of a message and a very long way away when you
    /// are not. It goes both ways: somewhere to jump to is only useful with a
    /// way back.
    fn wire_warning_jump(&self, text: &TextCtrl, bar: TextCtrl) {
        let body = *text;
        let a11y = self.a11y.clone();

        for (control, target, spoken) in [(*text, bar, "Security warning"), (bar, body, "Message")]
        {
            let a11y = a11y.clone();
            control.bind_internal(EventType::KEY_DOWN, move |event| {
                event.skip(true);
                // WXK_F7
                if event.get_key_code() != Some(346) {
                    return;
                }
                target.set_focus();
                let _ = a11y.announce(spoken, Priority::Normal);
            });
        }
    }

    /// Give one tab's text control its keys.
    fn wire_tab(&self, text: &TextCtrl, index: usize) {
        let closed = self.closed.clone();
        let go_back = self.go_back.clone();
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
                // Whatever this was opened from gets its turn back.
                if closed.borrow().is_some() {
                    go_back.start(1, true);
                }
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
        let lists = self.attachment_lists.clone();
        let save = self.save_attachment.clone();
        let read = self.read_attachment.clone();

        frame.on_menu(move |event| {
            let id = event.get_id();
            if id == ID_CANCEL {
                frame.show(false);
                return;
            }
            if id == ID_SAVE_ATTACHMENT || id == ID_READ_ATTACHMENT {
                let page = current.get();
                let chosen = lists
                    .borrow()
                    .get(page)
                    .and_then(|list| list.as_ref().and_then(ListBox::get_selection));
                let (act, what) = if id == ID_SAVE_ATTACHMENT {
                    (&save, Doing::Saving)
                } else {
                    (&read, Doing::Reading)
                };
                save_chosen(&documents, act, &a11y, page, chosen, what);
                return;
            }
            if id == ID_GO_ATTACHMENTS {
                let page = current.get();
                match lists.borrow().get(page).and_then(Option::as_ref) {
                    Some(list) => {
                        list.set_focus();
                        // The list announces its own name and row on focus, so
                        // saying anything here would repeat it.
                    }
                    None => {
                        // Said rather than left silent: a key that does
                        // nothing is indistinguishable from one that is broken.
                        let _ = a11y.announce("No attachments", Priority::Normal);
                    }
                }
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
                let mut open_lists = lists.borrow_mut();
                if page < open_lists.len() {
                    open_lists.remove(page);
                }
                drop(open_lists);
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
        frame.on_close({
            let closed = self.closed.clone();
            let go_back = self.go_back.clone();
            move |event| {
                if let WindowEventData::General(ref base) = event {
                    base.veto();
                }
                frame.show(false);
                if closed.borrow().is_some() {
                    go_back.start(1, true);
                }
            }
        });
    }

    /// Do the save the application set up, for a window that is not this one.
    ///
    /// The page has its own attachment list and needs the same two actions.
    /// Going through here rather than being handed its own copies means there
    /// is one place that knows how to save an attachment, so the two surfaces
    /// cannot come to disagree about what saving does.
    pub fn save_attachment_now(&self, attachment: &ReaderAttachment) {
        if let Some(handler) = self.save_attachment.borrow().as_ref() {
            handler(attachment);
        }
    }

    /// Do the read the application set up, for a window that is not this one.
    pub fn read_attachment_now(&self, attachment: &ReaderAttachment) {
        if let Some(handler) = self.read_attachment.borrow().as_ref() {
            handler(attachment);
        }
    }

    /// Say where closing this window should go back to.
    ///
    /// `None` means the window it was opened over, which is the mailbox.
    pub fn on_closed(&self, back_to: Option<Rc<dyn Fn()>>) {
        *self.closed.borrow_mut() = back_to;
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

    fn attachment(name: &str) -> ReaderAttachment {
        ReaderAttachment {
            message_row_id: 1,
            uid: 1,
            index: 0,
            name: name.to_string(),
            mime_type: "application/pdf".to_string(),
            size: 1024,
        }
    }

    #[test]
    fn test_a_pdf_can_be_read_here_whichever_way_it_says_so() {
        // Some senders label a PDF application/octet-stream, and some name it
        // without an extension. Either one on its own is enough, because
        // refusing to read a PDF somebody was sent because its label was
        // sloppy helps nobody.
        let mut by_type = attachment("report");
        by_type.mime_type = "application/pdf".to_string();
        let mut by_name = attachment("report.PDF");
        by_name.mime_type = "application/octet-stream".to_string();

        assert!(can_be_read_here(&by_type));
        assert!(can_be_read_here(&by_name));
    }

    #[test]
    fn test_everything_else_is_not_pretended_to_be_readable() {
        // Opening a spreadsheet in a text control produces a screenful of
        // nonsense, which is worse than saying it cannot be read.
        for (name, mime_type) in [
            ("book.epub", "application/epub+zip"),
            ("sheet.xlsx", "application/vnd.ms-excel"),
            ("photo.jpg", "image/jpeg"),
            ("archive.zip", "application/zip"),
        ] {
            let mut file = attachment(name);
            file.mime_type = mime_type.to_string();

            assert!(!can_be_read_here(&file), "{name}");
        }
    }

    #[test]
    fn test_a_message_with_nothing_attached_says_nothing_about_attachments() {
        // The announcement is read on every message that opens, so anything
        // in it that is not worth hearing is a cost paid on every message.
        assert_eq!(attachment_summary(&[]), "");
    }

    #[test]
    fn test_the_announcement_names_the_key_that_reaches_the_list() {
        // The list is last in the tab order, so somebody who reads the message
        // and closes it would never find out it was there.
        let said = attachment_summary(&[attachment("report.pdf")]);

        assert!(said.contains("1 attachment"), "{said}");
        assert!(said.contains("F8"), "{said}");
    }

    #[test]
    fn test_a_program_among_the_attachments_is_said_out_loud() {
        // The one fact that should change what somebody does next, and the one
        // a message pretending to be an invoice needs them not to notice.
        let said = attachment_summary(&[attachment("invoice.pdf"), attachment("invoice.pdf.exe")]);

        assert!(said.contains("2 attachments"), "{said}");
        assert!(said.contains("program"), "{said}");
    }

    #[test]
    fn test_several_programs_are_counted_rather_than_mentioned_once() {
        let said = attachment_summary(&[
            attachment("a.exe"),
            attachment("b.scr"),
            attachment("c.pdf"),
        ]);

        assert!(said.contains("3 attachments"), "{said}");
        assert!(said.contains("2 of them programs"), "{said}");
    }

    #[test]
    fn test_a_lone_program_is_described_as_what_it_is() {
        let said = attachment_summary(&[attachment("invoice.exe")]);

        assert!(said.contains("1 attachment"), "{said}");
        assert!(said.contains("which is a program"), "{said}");
    }

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
