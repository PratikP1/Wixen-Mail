//! wxdragon Composition Dialog
//!
//! Provides a modal dialog for composing, replying to, and forwarding emails.
//! The message body is a `WebView` holding a contenteditable page, built by
//! [`crate::presentation::editor_document`].

use crate::application::reading_habits::CopyLines;
use crate::common::types::MessageBody;
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::compose_toolbar;
use crate::presentation::editor_document;
use crate::presentation::editor_document::Reached;
use crate::presentation::html_renderer::HtmlRenderer;
use crate::presentation::ui_types::CompositionData;
use std::cell::RefCell;
use std::rc::Rc;
use wxdragon::event::WebViewEvents;
use wxdragon::event::webview_events::WebViewEventData;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

// ── Formatting toolbar IDs ──────────────────────────────────────────────────

// Button IDs: used as return codes via end_modal() or in button .with_id()
const ID_BOLD: Id = ID_HIGHEST + 100;
const ID_ITALIC: Id = ID_HIGHEST + 101;
const ID_UNDERLINE: Id = ID_HIGHEST + 102;
/// The first of a run, one per `Format::ALL`, so the handler can subtract and
/// index rather than matching thirteen constants against thirteen variants and
/// getting one pair wrong.
const ID_FORMAT_FIRST: Id = ID_HIGHEST + 120;
/// The last id the formatting run may take.
///
/// Nothing else in this file may use one between these two. The run grew by
/// thirteen entries in a single change and the preview's two ids were sitting
/// inside it, which was latent only because the preview's buttons raise one
/// kind of event and the formatting handler listens for another. Nothing
/// enforced that, so there is a test below instead of a hope.
const ID_FORMAT_LAST: Id = ID_HIGHEST + 149;
const ID_INSERT_LINK: Id = ID_HIGHEST + 150;
const ID_INSERT_TABLE: Id = ID_HIGHEST + 152;
const ID_SPELL_CHECK: Id = ID_HIGHEST + 153;
const ID_SPELL_CHANGE: Id = ID_HIGHEST + 154;
const ID_SPELL_CHANGE_ALL: Id = ID_HIGHEST + 155;
const ID_SPELL_IGNORE: Id = ID_HIGHEST + 156;
const ID_SPELL_IGNORE_ALL: Id = ID_HIGHEST + 157;
const ID_SPELL_ADD: Id = ID_HIGHEST + 158;
const ID_FORMAT_MENU: Id = ID_HIGHEST + 151;
const ID_SEND: Id = ID_HIGHEST + 110;
const ID_SAVE_DRAFT: Id = ID_HIGHEST + 111;
const ID_DISCARD: Id = ID_HIGHEST + 112;
const ID_ATTACH: Id = ID_HIGHEST + 113;
/// `WXK_DELETE`, the key that takes a row out of a list on Windows.
const KEY_DELETE: i32 = 127;
const ID_UNDO: Id = ID_HIGHEST + 114;
const ID_REDO: Id = ID_HIGHEST + 115;

/// Result of showing the compose dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeResult {
    /// User clicked Send
    Send(ComposeData),
    /// User clicked Save Draft
    SaveDraft(ComposeData),
    /// User discarded or cancelled
    Cancelled,
}

/// Data collected from the compose dialog fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeData {
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    /// The message as HTML, which is what the editor holds.
    pub body: String,
    /// The same message as plain text, for the other half of the multipart.
    ///
    /// Taken from the editor rather than converted here, so it is what the
    /// engine computed somebody to be looking at.
    pub body_plain: String,
    pub html_mode: bool,
    pub account_index: Option<u32>,
    /// The files to send with it, where they are on this computer.
    ///
    /// Paths rather than bytes, all the way to the moment of sending. A message
    /// is written over minutes, and a file read at Send is the version somebody
    /// meant to send rather than the one that existed when they picked it.
    pub attachments: Vec<std::path::PathBuf>,
}

/// Mode for opening the compose dialog
#[derive(Debug, Clone)]
pub enum ComposeMode {
    /// New blank message
    New,
    /// Reply to a message
    Reply {
        to: String,
        subject: String,
        quoted_body: MessageBody,
    },
    /// Reply to all recipients
    ReplyAll {
        to: String,
        cc: String,
        subject: String,
        quoted_body: MessageBody,
    },
    /// Forward a message
    Forward { subject: String, body: MessageBody },
    /// Edit an existing draft
    Draft(CompositionData),
}

/// Format a reply subject line: prepends "Re: " unless already present.
fn format_reply_subject(subject: &str) -> String {
    if subject.starts_with("Re: ") {
        subject.to_string()
    } else {
        format!("Re: {}", subject)
    }
}

/// Format a forward subject line: prepends "Fwd: " unless already present.
fn format_forward_subject(subject: &str) -> String {
    if subject.starts_with("Fwd: ") {
        subject.to_string()
    } else {
        format!("Fwd: {}", subject)
    }
}

/// Format a quoted body for reply.
fn format_reply_body(quoted_body: &MessageBody) -> MessageBody {
    quoted_under(quoted_body, "--- Original Message ---")
}

/// Format a forwarded body.
fn format_forward_body(body: &MessageBody) -> MessageBody {
    quoted_under(body, "---------- Forwarded message ----------")
}

/// Put an original under a line saying what it is, leaving room to type above.
///
/// The separator has to be made of whatever it is being joined to. A newline
/// in front of markup renders as nothing, and text with a marker glued on is
/// still text, so the one thing that must not happen is the join deciding the
/// body is now the other kind.
fn quoted_under(body: &MessageBody, marker: &str) -> MessageBody {
    match body {
        MessageBody::Plain(text) => MessageBody::Plain(format!("\n\n{marker}\n{text}")),
        MessageBody::Html(html) | MessageBody::Multipart { html, .. } => {
            MessageBody::Html(format!("<p><br></p><p>{marker}</p>{html}"))
        }
    }
}

/// Title for the compose dialog based on mode.
fn compose_title(mode: &ComposeMode) -> &'static str {
    match mode {
        ComposeMode::New => "Compose New Message",
        ComposeMode::Reply { .. } => "Reply",
        ComposeMode::ReplyAll { .. } => "Reply All",
        ComposeMode::Forward { .. } => "Forward",
        ComposeMode::Draft(_) => "Edit Draft",
    }
}

/// Work put back on the ordinary event loop, and what it is.
///
/// Everything here arrives inside the web view's own callback, where it cannot
/// be done: the spelling check opens modal dialogs from inside a callback the
/// browser control has not finished making, and a focus change made there is
/// undone by the browser's own focus work a moment later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Deferred {
    /// Walk the spelling of the message.
    Spelling,
    /// Leave the message body. `back` is Shift+Tab.
    Leaving { back: bool },
    /// Go to the toolbar.
    Toolbar,
}

/// Whether a half-written message is worth keeping as a draft.
///
/// Nothing typed yet is not worth a row in the drafts list, and it would be one
/// that reads as blank. A signature on its own is nothing typed: it is there
/// before anybody starts, so the body is asked about as text rather than as
/// markup, and empty markup counts as empty.
fn worth_keeping(data: &ComposeData) -> bool {
    !data.to.trim().is_empty()
        || !data.cc.trim().is_empty()
        || !data.bcc.trim().is_empty()
        || !data.subject.trim().is_empty()
        || !data.body_plain.trim().is_empty()
}

/// Show a message that has one thing to say and nothing to decide.
///
/// Beside the announcement rather than instead of it. The announcement is the
/// one somebody hears while their hands are still on the file dialog; this is
/// what is still on screen a moment later, for anybody reading rather than
/// listening.
fn say_so(parent: &Dialog, title: &str, said: &str) {
    let box_ = MessageDialog::builder(parent, said, title)
        .with_style(MessageDialogStyle::OK | MessageDialogStyle::IconInformation)
        .build();
    box_.show_modal();
    box_.destroy();
}

/// Every formatting command on one menu, raised where the caret is.
///
/// A menu bar would be the obvious home and a dialog cannot have one, so this
/// is a menu a button and a key both pop up. Raised from two places now, which
/// is why it is a function: the button, and Alt+O pressed inside the message
/// body, which is a web view and keeps every key it is given until the page
/// hands one back.
fn show_format_menu(dialog: &Dialog) {
    let mut menu = Menu::builder();
    for (index, format) in editor_document::Format::ALL.iter().enumerate() {
        menu = menu.append_item(
            ID_FORMAT_FIRST + index as Id,
            &format.label(),
            format.spoken(),
        );
    }
    let mut menu = menu
        .append_separator()
        .append_item(
            ID_INSERT_LINK,
            "Insert &Link...",
            "Turn the selected text into a link",
        )
        .append_item(
            ID_INSERT_TABLE,
            "Insert &Table...",
            "Add a table with proper column headers",
        )
        .build();
    dialog.popup_menu(&mut menu, None);
}

/// The compose dialog, with automatic draft saving.
///
/// `autosave` decides how often, and `on_autosave` is handed the fields as
/// they stand each time. The callback rather than a return value because the
/// dialog is modal: it does not come back until somebody is finished, and a
/// draft that is only kept at the end is not a draft that survives a crash.
#[allow(clippy::too_many_arguments)]
pub fn show_compose_dialog_full(
    parent: &Frame,
    mode: ComposeMode,
    account_names: &[String],
    active_account_index: u32,
    preview_before_send: bool,
    autosave: crate::application::autosave::AutosaveInterval,
    a11y: std::sync::Arc<crate::presentation::accessibility::Accessibility>,
    on_autosave: impl Fn(&ComposeData) + 'static,
) -> ComposeResult {
    // ── Create Dialog ────────────────────────────────────────────────────
    let title = compose_title(&mode);

    let dialog = Dialog::builder(parent, title)
        .with_size(850, 700)
        .with_style(
            DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder | DialogStyle::MaximizeBox,
        )
        .build();

    // ── Layout ───────────────────────────────────────────────────────────
    let main_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Header fields panel --
    let fields_sizer = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields_sizer.add_growable_col(1, 1);

    // Everything the message will carry. Paths rather than bytes: the file is
    // read at Send, so a picture edited while the message was being written
    // goes out as the version that existed when it was sent.
    let attached: Rc<RefCell<Vec<crate::application::attaching::Chosen>>> =
        Rc::new(RefCell::new(Vec::new()));

    // Account selector
    let account_label = StaticText::builder(&dialog)
        .with_label(Reached::From.label())
        .build();
    let account_choice = Choice::builder(&dialog)
        .with_choices(account_names.iter().map(|s| s.to_string()).collect())
        .with_selection(Some(active_account_index))
        .build();
    set_accessible_name(&account_choice, "From account");
    fields_sizer.add(
        &account_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields_sizer.add(&account_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // To field
    let to_label = StaticText::builder(&dialog)
        .with_label(Reached::To.label())
        .build();
    let to_field = TextCtrl::builder(&dialog).build();
    set_accessible_name(&to_field, "To");
    fields_sizer.add(
        &to_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields_sizer.add(&to_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // CC field
    let cc_label = StaticText::builder(&dialog)
        .with_label(Reached::Cc.label())
        .build();
    let cc_field = TextCtrl::builder(&dialog).build();
    set_accessible_name(&cc_field, "Cc");
    fields_sizer.add(
        &cc_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields_sizer.add(&cc_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // BCC field
    let bcc_label = StaticText::builder(&dialog)
        .with_label(Reached::Bcc.label())
        .build();
    let bcc_field = TextCtrl::builder(&dialog).build();
    set_accessible_name(&bcc_field, "Bcc");
    fields_sizer.add(
        &bcc_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields_sizer.add(&bcc_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Subject field
    let subject_label = StaticText::builder(&dialog)
        .with_label(Reached::Subject.label())
        .build();
    let subject_field = TextCtrl::builder(&dialog).build();
    set_accessible_name(&subject_field, "Subject");
    fields_sizer.add(
        &subject_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields_sizer.add(&subject_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    main_sizer.add_sizer(&fields_sizer, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // -- Compose toolbar --
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();

    // Prominent Send button (Outlook-style, first in toolbar)
    let send_toolbar_btn = Button::builder(&dialog)
        .with_label(Reached::Send.label())
        .with_id(ID_SEND)
        .with_size(Size::new(72, 30))
        .build();
    set_accessible_name(&send_toolbar_btn, "Send message, Ctrl+Enter");
    toolbar_sizer.add(&send_toolbar_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Undo / Redo
    let undo_btn = Button::builder(&dialog)
        .with_label(Reached::Undo.label())
        .with_id(ID_UNDO)
        .with_size(Size::new(52, 28))
        .build();
    set_accessible_name(&undo_btn, "Undo, Ctrl+Z");
    let redo_btn = Button::builder(&dialog)
        .with_label(Reached::Redo.label())
        .with_id(ID_REDO)
        .with_size(Size::new(52, 28))
        .build();
    set_accessible_name(&redo_btn, "Redo, Ctrl+Y");
    toolbar_sizer.add(&undo_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&redo_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Formatting: Bold, Italic and Underline, with accessible labels for screen readers
    let bold_btn = Button::builder(&dialog)
        .with_label("B")
        .with_id(ID_BOLD)
        .with_size(Size::new(32, 28))
        .build();
    set_accessible_name(&bold_btn, "Bold, Ctrl+B");
    let italic_btn = Button::builder(&dialog)
        .with_label("I")
        .with_id(ID_ITALIC)
        .with_size(Size::new(32, 28))
        .build();
    set_accessible_name(&italic_btn, "Italic, Ctrl+I");
    let underline_btn = Button::builder(&dialog)
        .with_label("U")
        .with_id(ID_UNDERLINE)
        .with_size(Size::new(32, 28))
        .build();
    set_accessible_name(&underline_btn, "Underline, Ctrl+U");
    // The other eight commands, and the link. Named as a word rather than a
    // symbol: B, I and U are recognisable letters, and there is no glyph for
    // "heading level 2" that anybody reads the same way.
    let format_btn = Button::builder(&dialog)
        .with_label(Reached::Format.label())
        .with_id(ID_FORMAT_MENU)
        .build();
    set_accessible_name(&format_btn, "Format, opens a menu");
    toolbar_sizer.add(&bold_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&italic_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&underline_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&format_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Spelling. On the toolbar and reachable by Tab as well as on F7, because
    // a key nobody can discover is a key nobody has, and this is the one
    // command in the window that nothing else hints at.
    let spell_btn = Button::builder(&dialog)
        .with_label(Reached::Spelling.label())
        .with_id(ID_SPELL_CHECK)
        .build();
    set_accessible_name(&spell_btn, "Check spelling, F7");
    toolbar_sizer.add(&spell_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Attach
    let attach_btn = Button::builder(&dialog)
        .with_label(Reached::Attach.label())
        .with_id(ID_ATTACH)
        .build();
    set_accessible_name(&attach_btn, "Attach a file, Alt+A");
    toolbar_sizer.add(&attach_btn, 0, SizerFlag::All, 2);

    // Remove all toolbar buttons from tab order: accessible via keyboard shortcuts
    send_toolbar_btn.set_can_focus(false);
    undo_btn.set_can_focus(false);
    redo_btn.set_can_focus(false);
    bold_btn.set_can_focus(false);
    italic_btn.set_can_focus(false);
    underline_btn.set_can_focus(false);
    attach_btn.set_can_focus(false);

    main_sizer.add_sizer(
        &toolbar_sizer,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right,
        8,
    );

    // -- The message body, which is a web view --
    //
    // Not a wxRichTextCtrl. That control is drawn by wxWidgets on every
    // platform, so it exposes no per-range accessibility attributes anywhere:
    // no misspelling can be marked, no heading can report itself, and every
    // announcement has to be made by hand and be slightly wrong forever. A web
    // view gets all of it from the engine, starting with native spelling
    // annotations that each screen reader announces itself.
    //
    // The objection that kept a web view out of the preview pane does not
    // apply. What trapped people there was a browser sharing a window with a
    // folder tree and a message list, where F6 had to cycle panes and Escape
    // had to return to the list. Here the keys that must escape are bound
    // inside the page and posted back out, and everything else belongs to the
    // editor, which is what somebody writing a message wants.
    let language = crate::data::config::ConfigManager::load_stored()
        .map(|config| config.app_config().language.clone())
        .unwrap_or_else(|_| "en".to_string());
    let body_editor = WebView::builder(&dialog)
        .with_backend(WebViewBackend::Default)
        .build();
    set_accessible_name(&body_editor, "Message body");
    body_editor.enable_context_menu(true);
    body_editor.enable_access_to_dev_tools(false);
    // The browser does not get this application's keys.
    body_editor.enable_browser_accelerator_keys(false);
    body_editor.add_script_message_handler(editor_document::CHANNEL);
    let set_body = {
        let language = language.clone();
        // One setting decides both the engine's marking and the sound at the
        // end of a wrong word. Read once, when the window opens, because the
        // page is built once and changing it would mean rebuilding the message
        // underneath somebody.
        let mark_spelling = crate::data::config::ConfigManager::load_stored()
            .map(|config| config.app_config().check_spelling_as_you_type)
            .unwrap_or(true);
        move |body: &MessageBody| {
            body_editor.set_page(
                &editor_document::editor_document(body, &language, mark_spelling),
                "",
            );
        }
    };
    set_body(&MessageBody::Plain(String::new()));

    main_sizer.add(&body_editor, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // -- What the message will carry --
    //
    // The line and the list say the same thing twice on purpose. The line is
    // one glance, and it is what gets announced when a file goes on or comes
    // off, because somebody who cannot see the window has nothing else to tell
    // them the file arrived. The list is where a particular file is found and
    // taken off again.
    let attachment_label = StaticText::builder(&dialog)
        .with_label(&crate::application::attaching::summary(&[]))
        .build();
    main_sizer.add(
        &attachment_label,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right,
        8,
    );

    let attachment_list = ListBox::builder(&dialog)
        .with_size(Size::new(-1, 72))
        .build();
    set_accessible_name_and_description(
        &attachment_list,
        "Attachments",
        "Delete takes the selected file off the message",
    );
    // Nothing attached yet, and an empty list in the tab order is a stop that
    // announces nothing. It comes back the moment there is a file in it.
    attachment_list.hide();
    main_sizer.add(
        &attachment_list,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right,
        8,
    );

    // -- Action buttons (Send is in toolbar above) --
    let button_sizer = BoxSizer::builder(Orientation::Horizontal).build();

    let draft_btn = Button::builder(&dialog)
        .with_label(Reached::SaveDraft.label())
        .with_id(ID_SAVE_DRAFT)
        .build();
    let discard_btn = Button::builder(&dialog)
        .with_label(Reached::Discard.label())
        .with_id(ID_DISCARD)
        .build();
    let cancel_btn = Button::builder(&dialog)
        .with_label(Reached::Cancel.label())
        .with_id(ID_CANCEL)
        .build();

    button_sizer.add_spacer(0); // Push buttons right
    button_sizer.add(&draft_btn, 0, SizerFlag::All, 4);
    button_sizer.add(&discard_btn, 0, SizerFlag::All, 4);
    button_sizer.add(&cancel_btn, 0, SizerFlag::All, 4);

    main_sizer.add_sizer(&button_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(main_sizer, true);

    // ── Pre-populate fields based on mode ────────────────────────────────
    // Filled before the copy lines are hidden or kept, because whether a reply
    // already copies somebody is what decides it.
    match &mode {
        ComposeMode::New => {}
        ComposeMode::Reply {
            to,
            subject,
            quoted_body,
        } => {
            to_field.set_value(to);
            subject_field.set_value(&format_reply_subject(subject));
            set_body(&format_reply_body(quoted_body));
        }
        ComposeMode::ReplyAll {
            to,
            cc,
            subject,
            quoted_body,
        } => {
            to_field.set_value(to);
            cc_field.set_value(cc);
            subject_field.set_value(&format_reply_subject(subject));
            set_body(&format_reply_body(quoted_body));
        }
        ComposeMode::Forward { subject, body } => {
            subject_field.set_value(&format_forward_subject(subject));
            set_body(&format_forward_body(body));
            to_field.set_focus();
        }
        ComposeMode::Draft(data) => {
            to_field.set_value(&data.to);
            cc_field.set_value(&data.cc);
            bcc_field.set_value(&data.bcc);
            subject_field.set_value(&data.subject);
            // A draft was written here, so it is this editor's own markup
            // coming back. Escaping it would show somebody their tags.
            set_body(&MessageBody::Html(data.body.clone()));
        }
    }

    // ── The copy lines ────────────────────────────────────────────────────
    //
    // Two empty fields between the recipient and the subject are two stops on
    // the way, every time anybody writes anything, and most messages go to one
    // person. Anybody who copies people on most of what they send wants the
    // opposite, so it is a choice, and a reply that already copies somebody
    // shows them regardless: hiding a field with an address in it hides who is
    // being written to.
    //
    // Alt+C and Alt+B still reach them, and bring them back when they do.
    let copy_lines = crate::data::config::ConfigManager::load_stored()
        .map(|config| CopyLines::from_setting(&config.app_config().copy_lines))
        .unwrap_or_default();
    if !copy_lines.shows(&cc_field.get_value(), &bcc_field.get_value()) {
        cc_label.hide();
        cc_field.hide();
        bcc_label.hide();
        bcc_field.hide();
        dialog.layout();
    }

    // ── Formatting ───────────────────────────────────────────────────────
    //
    // One place that applies a command, announces it, and puts the caret back
    // where somebody was typing. The last of those matters because a toolbar
    // button takes focus, and without it applying bold leaves you on the button
    // with the next keystroke going nowhere.
    //
    // Through the announcement queue rather than straight to the screen reader.
    // These fire from keys as well as buttons, so somebody applying formatting
    // as they write produces a stream of them, and the queue is what bounds and
    // coalesces that. Going direct would be the one path in the application
    // that can flood.
    let apply_format = {
        let a11y = a11y.clone();
        move |format: editor_document::Format| {
            run_in_editor(&body_editor, &editor_document::format_script(format));
            let _ = a11y.announce(
                format.spoken(),
                crate::presentation::accessibility::announcements::Priority::Normal,
            );
        }
    };

    for (button, format) in [
        (&bold_btn, editor_document::Format::Bold),
        (&italic_btn, editor_document::Format::Italic),
        (&underline_btn, editor_document::Format::Underline),
        (&undo_btn, editor_document::Format::Undo),
        (&redo_btn, editor_document::Format::Redo),
    ] {
        let apply = apply_format.clone();
        button.on_click(move |_| apply(format));
    }

    // Every command on one menu, raised by a button.
    //
    // A menu bar would be the obvious home and a dialog cannot have one, so
    // this is a button that pops the menu up. That is no worse for somebody
    // working by keyboard: Tab reaches the button, Enter opens the menu, and
    // the menu is arrowed and announced like any other. It is better than the
    // alternative of keys alone, because a key nobody can discover is a key
    // nobody has.
    //
    // The buttons on the toolbar cover five of the thirteen. The other eight
    // include the headings and lists, which are the ones that decide whether
    // the person receiving the message can navigate it.
    // The menu's commands arrive as ordinary menu events on the dialog, which
    // is where a popup menu sends them. Bound once here rather than inside the
    // button's handler, so opening the menu twice does not bind it twice.
    //
    // wxdragon does not implement MenuEvents for Dialog, only for Frame and
    // Panel, so this goes through the same bind_internal the reader window uses
    // for its keys.
    dialog.bind_internal(EventType::MENU, {
        let apply = apply_format.clone();
        let a11y = a11y.clone();
        move |event| {
            let id = event.get_id();
            if id == ID_INSERT_LINK {
                insert_link(&dialog, body_editor, &a11y);
                return;
            }
            if id == ID_INSERT_TABLE {
                insert_table(&dialog, body_editor, &a11y);
                return;
            }
            // Inside the run kept for formatting, or not ours at all. The
            // upper bound matters as much as the lower one: an id above the
            // run subtracts to a large number rather than a negative one, so
            // without this it depends on `Format::ALL` being short enough
            // that the lookup misses.
            if !(ID_FORMAT_FIRST..=ID_FORMAT_LAST).contains(&id) {
                event.skip(true);
                return;
            }
            let Ok(index) = usize::try_from(id - ID_FORMAT_FIRST) else {
                event.skip(true);
                return;
            };
            match editor_document::Format::ALL.get(index) {
                Some(format) => apply(*format),
                // Not ours: the toolbar buttons raise menu events too, and
                // swallowing them would stop Send working.
                None => event.skip(true),
            }
        }
    });

    spell_btn.on_click({
        let a11y = a11y.clone();
        move |_| check_spelling(&dialog, body_editor, &a11y)
    });

    format_btn.on_click(move |_| show_format_menu(&dialog));

    // ── The toolbar, worked from the keyboard ─────────────────────────────
    //
    // Nine buttons, in the order `compose_toolbar::GROUPS` describes them, so
    // the position a key lands on is the button it operates. A test pins every
    // position to exactly one button; nothing pins this list to that one, so
    // it is written once, here, next to the comment saying so.
    let toolbar_buttons = [
        send_toolbar_btn,
        undo_btn,
        redo_btn,
        bold_btn,
        italic_btn,
        underline_btn,
        format_btn,
        spell_btn,
        attach_btn,
    ];
    debug_assert_eq!(
        toolbar_buttons.len(),
        compose_toolbar::GROUPS
            .iter()
            .map(|group| group.keys.len())
            .sum::<usize>()
    );

    // Where the toolbar cursor is, kept between visits so leaving and coming
    // back lands where somebody was rather than at the start.
    // Where the toolbar cursor was, or `None` when focus is somewhere else.
    //
    // `None` is what makes the group get named on arrival: the announcement
    // leaves the group out when it has not changed, and somebody who has just
    // arrived has not been anywhere for it to have changed from.
    let toolbar_at: Rc<std::cell::Cell<Option<compose_toolbar::At>>> =
        Rc::new(std::cell::Cell::new(None));

    // Going to the toolbar: forget where focus was, then focus the button. The
    // saying is left to the handler below, which is the one place that does it,
    // so arriving does not announce twice.
    let go_to = {
        let toolbar_at = toolbar_at.clone();
        move |to: Option<compose_toolbar::At>| {
            let at = to.unwrap_or_default();
            let Some(position) = at.position() else {
                return;
            };
            let Some(button) = toolbar_buttons.get(position) else {
                return;
            };
            toolbar_at.set(None);
            button.set_focus();
        }
    };

    // Say where each button is when it takes focus, however it was reached.
    //
    // Not by taking the arrow keys, and the reason is narrower than it first
    // looked. Key events do reach a button here: a handler bound with
    // `bind_internal` sees an ordinary letter. It does not see the arrows or
    // Escape, because a dialog on Windows offers every message to
    // `IsDialogMessage` first and that is where Tab, the arrows and Escape are
    // taken to move focus and to close. The supported way out is the
    // `wxWANTS_CHARS` style on the control, and this binding exposes neither
    // that nor a char hook nor the navigation event.
    //
    // What wxWidgets does with the arrows is right anyway, left to right along
    // the row, so this rides on it rather than fighting it and adds the part
    // that was missing, which is knowing where you are.
    for (position, button) in toolbar_buttons.iter().enumerate() {
        let toolbar_at = toolbar_at.clone();
        let a11y = a11y.clone();
        button.on_set_focus(move |event| {
            event.skip(true);
            let Some(to) = compose_toolbar::At::at_position(position) else {
                return;
            };
            let from = toolbar_at.replace(Some(to));
            let _ = a11y.announce(
                &to.spoken(from),
                crate::presentation::accessibility::announcements::Priority::Normal,
            );
        });
    }

    // ── Attachments ───────────────────────────────────────────────────────
    //
    // Put the list, the line under the message and the announcement back in
    // agreement. Called after every change, because three places that each
    // update themselves are three places that drift.
    let refresh_attachments = {
        let attached = attached.clone();
        move |say: bool, a11y: &crate::presentation::accessibility::Accessibility| {
            let files = attached.borrow();
            let summary = crate::application::attaching::summary(&files);
            attachment_label.set_label(&summary);
            attachment_list.clear();
            for file in files.iter() {
                attachment_list.append(&file.label());
            }
            if files.is_empty() {
                attachment_list.hide();
            } else {
                attachment_list.show(true);
            }
            // The window has to lay out again: the list appears and disappears,
            // and the line above it changes width.
            dialog.layout();
            if say {
                let _ = a11y.announce(
                    &summary,
                    crate::presentation::accessibility::announcements::Priority::Normal,
                );
            }
        }
    };

    let attach_files = {
        let attached = attached.clone();
        let refresh = refresh_attachments.clone();
        move |a11y: &crate::presentation::accessibility::Accessibility| {
            let picker = FileDialog::builder(&dialog)
                .with_message("Attach files")
                .with_style(FileDialogStyle::Open | FileDialogStyle::FileMustExist)
                .build();
            if picker.show_modal() != ID_OK {
                // Cancelling is a decision. There is no outcome to report, and
                // announcing one would be noise on a key somebody pressed by
                // mistake.
                return;
            }
            let Some(path) = picker.get_path() else {
                let _ = a11y.announce(
                    "No file was chosen",
                    crate::presentation::accessibility::announcements::Priority::High,
                );
                return;
            };
            match crate::application::attaching::Chosen::at(std::path::Path::new(&path)) {
                Ok(file) => {
                    let name = file.label();
                    let first = attached.borrow().is_empty();
                    attached.borrow_mut().push(file);
                    // Said before the summary, because the name of the file
                    // that just went on is the thing being waited for.
                    //
                    // How to take one off again is said once, with the first
                    // file, and then not repeated. It belongs on the list, as
                    // the description of a control, and a description set that
                    // way does not reach the accessibility tree for a native
                    // list in this wxWidgets binding: what a screen reader
                    // reads there is the line above it. So it is said here,
                    // where it is heard, rather than left somewhere it is not.
                    let said = if first {
                        format!(
                            "Attached {name}. Press Delete in the attachments list to take one off"
                        )
                    } else {
                        format!("Attached {name}")
                    };
                    let _ = a11y.announce(
                        &said,
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );
                    refresh(false, a11y);
                    if let Some(complaint) =
                        crate::application::attaching::over_the_limit(&attached.borrow())
                    {
                        let _ = a11y.announce(
                            &complaint,
                            crate::presentation::accessibility::announcements::Priority::High,
                        );
                        say_so(&dialog, "Too much to send", &complaint);
                    }
                }
                // Said out loud as well as shown. A file that could not be read
                // is the case where somebody most needs to know nothing went on
                // the message, and a dialog they have to find is not that.
                Err(why) => {
                    let why = format!("{why}");
                    let _ = a11y.announce(
                        &why,
                        crate::presentation::accessibility::announcements::Priority::High,
                    );
                    say_so(&dialog, "That file could not be attached", &why);
                }
            }
        }
    };

    attach_btn.on_click({
        let attach = attach_files.clone();
        let a11y = a11y.clone();
        move |_| attach(&a11y)
    });

    // Delete takes the selected file off the message. The key Windows uses for
    // removing a row from a list, so nothing new has to be learned, and the
    // announcement names what went rather than only what is left.
    attachment_list.on_key_down({
        let attached = attached.clone();
        let refresh = refresh_attachments.clone();
        let a11y = a11y.clone();
        move |event| {
            let WindowEventData::Keyboard(ref key) = event else {
                event.skip(true);
                return;
            };
            if key.event.get_key_code() != Some(KEY_DELETE) {
                event.skip(true);
                return;
            }
            let Some(row) = attachment_list.get_selection() else {
                return;
            };
            let row = row as usize;
            let gone = {
                let mut files = attached.borrow_mut();
                if row >= files.len() {
                    return;
                }
                files.remove(row).name
            };
            let _ = a11y.announce(
                &format!("Removed {gone}"),
                crate::presentation::accessibility::announcements::Priority::Normal,
            );
            refresh(false, &a11y);
        }
    });

    // Send button (in toolbar) closes dialog with ID_SEND
    send_toolbar_btn.on_click({
        move |_| {
            dialog.end_modal(ID_SEND);
        }
    });

    // Ctrl+Enter, Ctrl+S and Escape come out of the page rather than from a
    // key handler on the control. A web view consumes keys once it has focus,
    // so the page binds the ones that have to leave and posts them here. The
    // formatting keys are bound there too, and arrive already applied: they
    // come back only so the announcement goes through the same queue as every
    // other one rather than straight at the screen reader.
    // Built once and kept. Making a Windows spell checker is a COM object
    // creation, and one per word typed would be a cost paid on every keystroke
    // for the life of the window.
    let typing_speller = std::rc::Rc::new(crate::service::spellcheck::for_language(&language));

    // Where Tab is taking focus, and the timer that takes it there.
    //
    // Not set from inside the web view's callback. Tab is a focus key, so the
    // browser does its own focus work after handing the message over, and it
    // lands one control past wherever this put it: Tab reached Discard when it
    // meant Save Draft, and Shift+Tab reached Save Draft when it meant the
    // Subject line. Putting it back on the ordinary event loop lets the
    // browser finish first, which is the same reason F7 goes through a timer.
    let read_compose_data = {
        let attached = attached.clone();
        move || {
            let (body, body_plain) = editor_document::message_from_editor(
                body_editor.run_script(&editor_document::read_body_script()),
                body_editor.run_script(&editor_document::read_plain_script()),
            )?;
            Some(ComposeData {
                to: to_field.get_value(),
                cc: cc_field.get_value(),
                bcc: bcc_field.get_value(),
                subject: subject_field.get_value(),
                body,
                body_plain,
                html_mode: true,
                account_index: account_choice.get_selection(),
                attachments: attached
                    .borrow()
                    .iter()
                    .map(|file| file.path.clone())
                    .collect(),
            })
        }
    };

    // Reading the window, and the automatic save that uses it.
    //
    // Both live above the timer handler because the handler calls the save,
    // and there is one handler for every timer this dialog owns.
    let save_draft_now = {
        let read_compose_data = read_compose_data.clone();
        move || {
            // Could not read it, so there is nothing to save that is better
            // than what is already saved. Silent to the user on purpose: this
            // fires every couple of minutes and the message is untouched, so
            // there is nothing for them to do. The log is where it belongs.
            let Some(data) = read_compose_data() else {
                tracing::warn!("Autosave skipped: the editor did not answer");
                return;
            };
            // Nothing typed yet is not worth a row in the drafts list, and it
            // would be one that reads as blank.
            if data.to.trim().is_empty()
                && data.subject.trim().is_empty()
                && data.body.trim().is_empty()
            {
                return;
            }
            on_autosave(&data);
        }
    };

    let waiting: Rc<std::cell::Cell<Option<Deferred>>> = Rc::new(std::cell::Cell::new(None));
    let later = Rc::new(Timer::new(&dialog));

    // One handler for every timer this window owns, because that is what a
    // timer event is here.
    //
    // `Timer::on_tick` binds the timer event on the *owner*, with nothing to
    // say which timer fired. Three timers on one dialog therefore meant three
    // handlers all running on every tick, and that was not only a tidiness
    // problem: the autosave timer and the spelling timer were both on this
    // dialog, so every automatic draft save was also opening the spelling
    // check, a modal dialog, in the middle of somebody typing. It was latent
    // until a third timer made it obvious.
    later.on_tick({
        let waiting = waiting.clone();
        let go_to = go_to.clone();
        let toolbar_at = toolbar_at.clone();
        let a11y = a11y.clone();
        move |_| {
            match waiting.take() {
                Some(Deferred::Spelling) => check_spelling(&dialog, body_editor, &a11y),
                // Where the toolbar was left, so leaving and coming back lands
                // where somebody was. The group is named on arrival, always,
                // because arriving is the moment nobody knows where they are.
                Some(Deferred::Toolbar) => go_to(toolbar_at.get()),
                // The two directions are not done the same way, and each is
                // the one that was measured to land where it should.
                //
                // Forward asks the window, so the answer is whatever the tab
                // order says and stays right when a control is added. Naming a
                // control was tried and arrives one further on: focus set to
                // Save Draft landed on Discard, and Tab is not a key anybody
                // should have to aim at a destructive button. The argument
                // reads backwards because it does: `navigate(true)` moves to
                // the control before this one.
                //
                // Backward names the Subject line, because asking the window
                // walks into the formatting toolbar, which sits between the
                // fields and the message in the order the controls were made.
                // Somebody pressing Shift+Tab in the body is going back to the
                // recipients and the subject, not to the Bold button, which
                // has its own key.
                Some(Deferred::Leaving { back }) => {
                    if back {
                        subject_field.set_focus();
                    } else {
                        body_editor.navigate(false);
                    }
                }
                // Nothing was asked for, so this is the autosave timer.
                None => save_draft_now(),
            }
        }
    });

    body_editor.on_script_message_received({
        let a11y = a11y.clone();
        let typing_speller = typing_speller.clone();
        let later = later.clone();
        let waiting = waiting.clone();
        let attach_now = attach_files.clone();
        let apply_undo = apply_format.clone();
        move |event| {
            let Some(raw) = event.get_string() else {
                return;
            };
            match editor_document::parse_message(&raw) {
                Some(editor_document::EditorMessage::Send) => dialog.end_modal(ID_SEND),
                Some(editor_document::EditorMessage::Save) => dialog.end_modal(ID_SAVE_DRAFT),
                Some(editor_document::EditorMessage::Cancel) => dialog.end_modal(ID_CANCEL),
                Some(editor_document::EditorMessage::Formatted(format)) => {
                    let _ = a11y.announce(
                        format.spoken(),
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );
                }
                // Tab in the last cell made another row. Said out loud
                // because the alternative is discovering it by finding
                // yourself in a cell that was not there a moment ago.
                Some(editor_document::EditorMessage::TableRowAdded) => {
                    let _ = a11y.announce(
                        "Row added",
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );
                }
                // Not run from in here. This is the web view's own callback,
                // and the check opens modal dialogs and runs scripts, so
                // doing it now is a nested event loop started from inside a
                // callback the browser control has not finished making. The
                // one-shot timer puts it back on the ordinary event loop,
                // which is where the Spelling button's version already runs,
                // so both routes into the feature behave the same way.
                Some(editor_document::EditorMessage::CheckSpelling) => {
                    waiting.set(Some(Deferred::Spelling));
                    later.start(1, true);
                }
                // The sound at the end of a word that is wrong. Not spoken:
                // the engine has already marked the word, and the screen
                // reader says it as the caret crosses it, which is better than
                // anything said here and would collide with the echo of what
                // was just typed.
                Some(editor_document::EditorMessage::WordFinished(word)) => {
                    if !typing_speller.check(&word).is_empty() {
                        let _ = a11y.earcon(
                            crate::presentation::accessibility::feedback::Event::MisspelledWord,
                        );
                    }
                }
                Some(editor_document::EditorMessage::Styled(style)) => {
                    let _ = a11y.announce(
                        style.spoken,
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );
                }
                // The page marked the words and is waiting to hear whether the
                // address is one this will carry. Either answer says what it
                // did: a link that quietly failed to become a link is a
                // message somebody sends believing it has one.
                Some(editor_document::EditorMessage::Link(url)) => {
                    let (script, spoken) = editor_document::resolve_markdown_link(&url);
                    run_in_editor(&body_editor, &script);
                    let _ = a11y.announce(
                        &spoken,
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );
                }
                // Alt and a letter, which the web view would otherwise have
                // kept. Everything the compose window offers is on this list,
                // so from inside the body Alt+T reaches the recipients and
                // Alt+O opens the formatting menu, the same as everywhere else
                // in the window.
                Some(editor_document::EditorMessage::Reached(reached)) => {
                    use editor_document::Reached;
                    match reached {
                        Reached::From => account_choice.set_focus(),
                        Reached::To => to_field.set_focus(),
                        // Asking for a line that is put away brings it back.
                        // A key that focuses a hidden field is a key that does
                        // nothing, and there would be no way to tell.
                        Reached::Cc => {
                            cc_label.show(true);
                            cc_field.show(true);
                            dialog.layout();
                            cc_field.set_focus();
                        }
                        Reached::Bcc => {
                            bcc_label.show(true);
                            bcc_field.show(true);
                            dialog.layout();
                            bcc_field.set_focus();
                        }
                        Reached::Subject => subject_field.set_focus(),
                        Reached::Send => dialog.end_modal(ID_SEND),
                        Reached::Undo => apply_undo(editor_document::Format::Undo),
                        Reached::Redo => apply_undo(editor_document::Format::Redo),
                        Reached::Format => show_format_menu(&dialog),
                        // Through the timer for the same reason F7 is: this is
                        // the web view's own callback, and the check opens
                        // modal dialogs and runs scripts.
                        Reached::Spelling => {
                            waiting.set(Some(Deferred::Spelling));
                            later.start(1, true);
                        }
                        Reached::Attach => attach_now(&a11y),
                        Reached::SaveDraft => dialog.end_modal(ID_SAVE_DRAFT),
                        Reached::Discard => dialog.end_modal(ID_DISCARD),
                        Reached::Cancel => dialog.end_modal(ID_CANCEL),
                    }
                }
                // Tab, which outside a table used to do nothing at all, so the
                // only ways out of the body were the mouse and Escape, and
                // Escape throws the message away.
                Some(editor_document::EditorMessage::Leaving { back }) => {
                    waiting.set(Some(Deferred::Leaving { back }));
                    later.start(1, true);
                }
                // Ctrl+backslash. Through the same timer as Tab, because this
                // moves focus too and the browser does its own focus work
                // after the message is handed over.
                Some(editor_document::EditorMessage::ToToolbar) => {
                    waiting.set(Some(Deferred::Toolbar));
                    later.start(1, true);
                }
                // The page is ours, so an unknown message is a bug rather than
                // an attack, and doing nothing beats guessing which command
                // somebody meant when every one of them sends or discards.
                None => tracing::warn!("Unrecognised editor message: {}", raw),
            }
        }
    });

    // Ctrl+Enter → Send from any other focused control (dialog-level fallback)
    dialog.on_key_down({
        move |event| {
            if let WindowEventData::Keyboard(ref kb) = event
                && kb.event.control_down()
                && kb.event.get_key_code() == Some(13)
            {
                dialog.end_modal(ID_SEND);
                return;
            }
            event.skip(true);
        }
    });

    // Save Draft
    draft_btn.on_click({
        move |_| {
            dialog.end_modal(ID_SAVE_DRAFT);
        }
    });

    // Discard
    discard_btn.on_click({
        move |_| {
            dialog.end_modal(ID_DISCARD);
        }
    });

    // Cancel
    cancel_btn.on_click({
        move |_| {
            dialog.end_modal(ID_CANCEL);
        }
    });

    // ── Automatic draft saving ────────────────────────────────────────
    //
    // Started before the modal loop, because a modal dialog does not return
    // until somebody has finished and a draft kept only at the end is not a
    // draft that survives anything. The timer belongs to the dialog and stops
    // with it.
    // Everything the dialog holds, read at the moment it is asked for.
    //
    // One place rather than two. The autosave timer and the send path both
    // need it, and when they each built their own a field added to ComposeData
    // got filled in one and forgotten in the other.
    // `None` when the editor did not answer, which is not the same as an empty
    // message and must not be treated as one: the autosave timer would write
    // it over the draft, and the send path would queue a message with nothing
    // in it and no error.
    //
    // Both reads are pure, which matters more than it looks. `run_script` uses
    // a four kilobyte buffer and runs the script a second time when the answer
    // does not fit, so anything asked for here has to be safe to run twice. A
    // script that changes the message must either return nothing or return
    // something small; `wixenReplaceWord` returns a position, which is both.
    // Held for the life of the dialog: dropping the timer stops it, and the
    // dialog is what it belongs to.
    // Held for the life of the dialog: dropping the timer stops it, and the
    // dialog is what it belongs to. It carries no handler of its own, because
    // a timer event here reaches every handler bound on the dialog, so there
    // is one of those and it decides what the tick was for.
    let _autosave_timer = autosave.interval().map(|every| {
        let timer = Timer::new(&dialog);
        // Milliseconds, and the cast is safe: the range stops at ten minutes.
        timer.start(every.as_millis() as i32, false);
        timer
    });

    // ── Show dialog modally (loop for preview-then-send) ───────────────
    //
    // Broken out of rather than returned from, so there is exactly one way
    // past this point and the window is taken down on all of them. The body
    // is a browser control, so a compose window left behind is a browser left
    // behind, and somebody who writes mail all day opens a lot of them.
    let outcome = 'compose: loop {
        let result = dialog.show_modal();

        // Every way out that is not Send or Save Draft ends the message. The
        // ones that mean it, Discard and Cancel, are buttons somebody chose.
        // Escape is not: it reaches the dialog from anywhere in the window,
        // including from a toolbar button, and it used to throw away whatever
        // had been written with nothing asked and nothing kept. A message
        // written over ten minutes was one keystroke from gone.
        //
        // Discard still discards, because that is the word on the button. The
        // other ways out keep what was written, and say so.
        if result == ID_CANCEL
            && let Some(data) = read_compose_data()
            && worth_keeping(&data)
        {
            let _ = a11y.announce(
                "Saved as a draft",
                crate::presentation::accessibility::announcements::Priority::Normal,
            );
            break 'compose ComposeResult::SaveDraft(data);
        }
        if result != ID_SEND && result != ID_SAVE_DRAFT {
            break 'compose ComposeResult::Cancelled;
        }

        // Could not read it. Going on would send or save an empty message
        // over the one that is still sitting in the window, so instead say so
        // and put the window back. Nothing is lost: the message is where it
        // was, and trying again is the whole recovery.
        let Some(data) = read_compose_data() else {
            tracing::error!("The editor did not answer, so nothing was sent or saved");
            let _ = a11y.announce(
                "The message could not be read. Nothing was sent or saved. Try again.",
                crate::presentation::accessibility::announcements::Priority::High,
            );
            continue;
        };

        match result {
            _ if result == ID_SEND => {
                if data.to.trim().is_empty() {
                    tracing::warn!("Send attempted with empty To field");
                    break 'compose ComposeResult::Cancelled;
                }
                // Before the preview rather than after it. Somebody who has
                // already confirmed a message is somebody who has decided, and
                // stopping them then reads as the application changing its
                // mind.
                if !confirm_spelling(&dialog, &data) {
                    continue;
                }
                if preview_before_send {
                    // Show preview-before-send dialog
                    match show_send_preview(&dialog, &data, account_names) {
                        PreviewDecision::ConfirmSend => break 'compose ComposeResult::Send(data),
                        PreviewDecision::GoBack => continue, // re-show compose dialog
                    }
                } else {
                    break 'compose ComposeResult::Send(data);
                }
            }
            // The only other way here: the guard above breaks for every
            // result that is neither of these two.
            _ => break 'compose ComposeResult::SaveDraft(data),
        }
    };

    // Stopped before the window goes, not after. Both are owned by it, and a
    // timer that fires into a handler that is no longer there is a crash
    // rather than a leak. Stopping is enough: each frees itself when its value
    // drops at the end of this function.
    if let Some(timer) = &_autosave_timer {
        timer.stop();
    }
    later.stop();
    // wxWidgets does not free a dialog when the Rust value goes; the docs say
    // to destroy it, and wx_calendar has always done so. Nothing here did, so
    // every compose window ever opened stayed for the life of the session,
    // and since the body became a web view that is a browser process tree
    // each time.
    dialog.destroy();
    outcome
}

/// Run a script in the editor, and put the keyboard back into the editor.
///
/// Both halves, always, which is why this exists rather than two calls at each
/// site. The `focus()` inside the generated scripts moves the DOM active
/// element and nothing else. When the command came from a toolbar button or a
/// menu the Win32 focus is on that button, and only `wxWindow::SetFocus`,
/// which reaches the WebView control, brings it back.
///
/// Splitting the two is how Insert Table came to announce "In the first cell.
/// Tab moves to the next cell" while Tab moved to the Spelling button and
/// nothing typed reached the table.
///
/// Not verified in the running application. Nothing in Rust can assert where
/// Win32 focus is, so the tests here check the script and not the focus, and
/// only a screen reader pass settles it.
fn run_in_editor(body_editor: &WebView, script: &str) {
    body_editor.run_script(script);
    body_editor.set_focus();
}

/// Whether there is anything worth stopping the send for, and what to say.
///
/// Reads the text half. The editor holds HTML, so the moment there is a second
/// line the body is "Hello Sam<div>See you tomorrow</div>", and a spell checker
/// has no idea what a tag is: it reports "Sam<div>See" and "tomorrow</div" and
/// asks about a message with nothing wrong in it.
///
/// Separate from the asking so the decision can be tested, because which half
/// it reads is the whole question and getting it wrong is invisible from the
/// outside. It produces a confirmation that looks right and fires every time,
/// which is the one thing this check must never become.
fn spelling_question(
    data: &ComposeData,
    speller: &dyn crate::service::spellcheck::Speller,
) -> Option<String> {
    crate::service::spellcheck::before_sending(&speller.check(&data.body_plain))
}

/// Check the body, and ask once if anything looks wrong.
///
/// `true` to carry on sending. `false` to go back to the message, which is what
/// happens when somebody chooses to look at the words rather than send anyway.
///
/// Silent when there is nothing to say, which is the common case and has to
/// stay free: a confirmation on every message is one people learn to dismiss
/// without reading, and then it is not there the time it mattered.
fn confirm_spelling(parent: &Dialog, data: &ComposeData) -> bool {
    use crate::service::spellcheck;

    // The text half again: an empty message is one with no words in it, and
    // "<p></p>" is not empty as markup.
    if data.body_plain.trim().is_empty() {
        return true;
    }
    let stored = crate::data::config::ConfigManager::load_stored();
    // Asked for, or not asked at all. Somebody who has turned this off has
    // decided, and a question they dismiss on every message is worse than no
    // question: it teaches them to dismiss the one that mattered.
    let wanted = stored
        .as_ref()
        .map(|config| config.app_config().check_spelling_before_send)
        .unwrap_or(true);
    if !wanted {
        return true;
    }
    let language = stored
        .map(|config| config.app_config().language.clone())
        .unwrap_or_else(|_| "en".to_string());

    let speller = spellcheck::for_language(&language);
    let Some(said) = spelling_question(data, speller.as_ref()) else {
        return true;
    };

    // Send is the default, so Enter sends. Somebody who meant to send and
    // heard the warning should not have to find a button, and the words are in
    // the question, so the decision can be made from hearing it alone.
    let asker = MessageDialog::builder(
        parent,
        // The buttons are Yes and No, which this builder cannot relabel, so
        // the question has to say what each one means. "Send anyway?" answered
        // Yes or No is unambiguous; "Send anyway, or go back?" answered Yes is
        // not.
        &format!(
            "{said}

Send it anyway?"
        ),
        "Check the spelling",
    )
    .with_style(MessageDialogStyle::YesNo | MessageDialogStyle::IconQuestion)
    .build();
    let answer = asker.show_modal();
    asker.destroy();

    answer == ID_YES
}

/// Ask for a URL and put it on the selected text.
///
/// The URL is checked before it goes anywhere near the document. That is the
/// same guard the reader puts on a link a stranger sent, and it applies here
/// for a reason worth stating: this message goes to somebody else, so a
/// `javascript:` URL typed into a composer is a link posted to another person.
/// What somebody decided about one word.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SpellChoice {
    /// Put this in its place.
    Change(String),
    /// Put this in the place of every copy of the word.
    ChangeAll(String),
    /// Leave this one.
    Ignore,
    /// Leave every copy of it for the rest of this pass.
    IgnoreAll,
    /// Teach the dictionary, so nothing asks again in any application.
    Add,
    /// Stop here.
    Stop,
}

/// Walk the message a misspelling at a time.
///
/// The engine already marks these and the screen reader already announces them
/// as the caret crosses one. What no engine exposes is the list, so without
/// this the only way to find three wrong words in a long message is to read the
/// whole message. This is the part that has to be ours.
///
/// The word is selected in the editor before the question is asked, so the
/// answer is about something somebody can hear in place rather than a word
/// quoted out of its sentence.
fn check_spelling(
    dialog: &Dialog,
    body_editor: WebView,
    a11y: &std::sync::Arc<crate::presentation::accessibility::Accessibility>,
) {
    use crate::application::spell_session as session;
    use crate::presentation::accessibility::announcements::Priority;

    let language = crate::data::config::ConfigManager::load_stored()
        .map(|config| config.app_config().language.clone())
        .unwrap_or_else(|_| "en".to_string());
    let speller = crate::service::spellcheck::for_language(&language);

    let mut ignored = session::Ignored::default();
    // Where the last thing that happened left off. `None` is the start.
    let mut resume_from: Option<crate::application::words::Position> = None;
    let mut corrected = 0usize;

    loop {
        let text = editor_document::text_from_editor(
            &body_editor
                .run_script(&editor_document::text_script())
                .unwrap_or_default(),
        );
        let words = crate::application::words::words_in(&text);
        let found = session::findings(
            &words,
            |word| !speller.check(word).is_empty(),
            |word| speller.suggest(word, 5),
        );
        let Some(finding) = session::next_finding(&found, &ignored, resume_from) else {
            break;
        };
        // Past the word rather than at it, so a word that is left alone is not
        // offered again on the next turn of the loop.
        let past = crate::application::words::Position {
            node: finding.at.node,
            offset: finding.end,
        };

        body_editor.run_script(&editor_document::select_word_script(
            finding.at,
            finding.end,
        ));
        let _ = a11y.announce(&finding.spoken(), Priority::High);

        match ask_about_word(dialog, finding) {
            SpellChoice::Stop => return,
            SpellChoice::Ignore => resume_from = Some(past),
            SpellChoice::IgnoreAll => {
                ignored.add(&finding.word);
                resume_from = Some(past);
            }
            SpellChoice::Add => {
                match speller.add_to_dictionary(&finding.word) {
                    Ok(()) => {
                        let _ = a11y.announce(
                            &format!("{} added to the dictionary", finding.word),
                            Priority::Normal,
                        );
                    }
                    // Said rather than swallowed: somebody who thinks a word is
                    // learned will meet it again and believe the feature is
                    // broken instead of knowing it never took.
                    Err(error) => {
                        tracing::warn!("Could not add {} to the dictionary: {error}", finding.word);
                        let _ = a11y.announce(
                            &format!("{} could not be added to the dictionary", finding.word),
                            Priority::High,
                        );
                        ignored.add(&finding.word);
                    }
                }
                resume_from = Some(past);
            }
            SpellChoice::Change(replacement) => {
                let landed = replace_one(&body_editor, finding.at, finding.end, &replacement);
                corrected += 1;
                // Where the page says the caret ended up. If it could not say,
                // stop rather than guess: carrying on from the wrong place
                // corrects a word nobody asked about.
                let Some(landed) = landed else { break };
                resume_from = Some(landed);
            }
            SpellChoice::ChangeAll(replacement) => {
                // Last in the message first, so replacing one never moves one
                // that has not been replaced yet, and the place this one is
                // standing on survives until its turn.
                let places = session::same_word_places(&found, &finding.word, finding.at);
                let ends: std::collections::HashMap<_, _> =
                    found.iter().map(|f| (f.at, f.end)).collect();
                let mut landed = None;
                for place in &places {
                    let Some(end) = ends.get(place) else { continue };
                    let after = replace_one(&body_editor, *place, *end, &replacement);
                    if *place == finding.at {
                        landed = after;
                    }
                }
                corrected += places.len();
                let Some(landed) = landed else { break };
                resume_from = Some(landed);
            }
        }
    }

    // Back into the message. The walk ends on a dialog closing, which returns
    // focus to whatever opened it, and that is the Spelling button when F7
    // came from the toolbar. Somebody who has just finished checking a message
    // is somebody about to carry on writing it.
    body_editor.set_focus();
    let _ = a11y.announce(&session::finished(corrected), Priority::High);
}

/// Replace one word, and say where the page put the caret afterwards.
///
/// The page reports it rather than this working it out. What a replacement
/// does to the positions after it is the page's business, and the version this
/// replaced added up how many words a replacement was, which was wrong for an
/// empty one and skipped the following misspelling without saying anything.
fn replace_one(
    body_editor: &WebView,
    at: crate::application::words::Position,
    end: usize,
    replacement: &str,
) -> Option<crate::application::words::Position> {
    let answer =
        body_editor.run_script(&editor_document::replace_word_script(at, end, replacement))?;
    editor_document::position_from_editor(&answer)
}

/// Ask what to do about one word.
///
/// The layout is the one people already know from Word: the word, a field
/// holding what it will become, and the suggestions under it. The field is
/// editable and comes first, so somebody who knows the answer types it and
/// presses Enter without ever hearing the list.
fn ask_about_word(
    parent: &Dialog,
    finding: &crate::application::spell_session::Finding,
) -> SpellChoice {
    use crate::application::spell_session::Problem;

    let asker = Dialog::builder(parent, "Check Spelling")
        .with_style(DialogStyle::DefaultDialogStyle)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let heading = StaticText::builder(&asker)
        .with_label(&format!("{}: {}", finding.problem.spoken(), finding.word))
        .build();
    sizer.add(&heading, 0, SizerFlag::All, 8);

    // A repeated word is fixed by deleting it, so the field starts empty and
    // the button says Delete. Offering "change it to itself" would be a
    // question with no useful answer.
    let repeated = finding.problem == Problem::Repeated;
    let label = StaticText::builder(&asker)
        .with_label(if repeated {
            "&Replace with (leave empty to delete):"
        } else {
            "Change &to:"
        })
        .build();
    let replacement = TextCtrl::builder(&asker)
        .with_value(if repeated {
            ""
        } else {
            finding.suggestions.first().map_or("", String::as_str)
        })
        .build();
    set_accessible_name(&replacement, "Change to");
    sizer.add(
        &label,
        0,
        SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        8,
    );
    sizer.add(&replacement, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let suggestions_label = StaticText::builder(&asker)
        .with_label("&Suggestions:")
        .build();
    let suggestions = ListBox::builder(&asker).build();
    for suggestion in &finding.suggestions {
        suggestions.append(suggestion);
    }
    if !finding.suggestions.is_empty() {
        suggestions.set_selection(0, true);
    }
    set_accessible_name(&suggestions, "Suggestions");
    sizer.add(
        &suggestions_label,
        0,
        SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        8,
    );
    sizer.add(&suggestions, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // Arrowing the list fills the field, so what will happen is always in one
    // place rather than depending on which control was touched last.
    suggestions.on_selection_changed(move |_| {
        if let Some(picked) = suggestions.get_string_selection() {
            replacement.set_value(&picked);
        }
    });

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let add_button = |id: Id, text: &str, name: &str| {
        let button = Button::builder(&asker).with_id(id).with_label(text).build();
        set_accessible_name(&button, name);
        button.on_click(move |_| asker.end_modal(id));
        buttons.add(&button, 0, SizerFlag::All, 4);
    };
    add_button(
        ID_SPELL_CHANGE,
        if repeated { "&Delete" } else { "&Change" },
        if repeated { "Delete" } else { "Change" },
    );
    if !repeated {
        add_button(ID_SPELL_CHANGE_ALL, "Change &All", "Change all");
    }
    add_button(ID_SPELL_IGNORE, "&Ignore", "Ignore");
    add_button(ID_SPELL_IGNORE_ALL, "I&gnore All", "Ignore all");
    if !repeated {
        add_button(ID_SPELL_ADD, "A&dd to Dictionary", "Add to dictionary");
    }
    add_button(ID_CANCEL, "&Close", "Close");
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 4);

    asker.set_sizer_and_fit(sizer, true);
    // Focus starts in the field rather than on a button: it holds the answer,
    // and Enter from there is the common case.
    replacement.set_focus();

    let answer = asker.show_modal();
    let typed = replacement.get_value();
    // Read first, then destroy: the field belongs to the dialog. This is the
    // worst of the leaks, because it is one per word rather than one per
    // window, so a message with thirty misspellings left thirty behind.
    asker.destroy();
    match answer {
        ID_SPELL_CHANGE => SpellChoice::Change(typed),
        ID_SPELL_CHANGE_ALL => SpellChoice::ChangeAll(typed),
        ID_SPELL_IGNORE => SpellChoice::Ignore,
        ID_SPELL_IGNORE_ALL => SpellChoice::IgnoreAll,
        ID_SPELL_ADD => SpellChoice::Add,
        _ => SpellChoice::Stop,
    }
}

/// Ask for a table's shape and put one in the message.
///
/// Rows, columns, and whether the first row is headers. That last one is the
/// question worth asking: a header row is what makes the table navigable for
/// whoever receives the message, and defaulting it on is the accessible
/// default rather than a preference.
fn insert_table(
    dialog: &Dialog,
    body_editor: WebView,
    a11y: &std::sync::Arc<crate::presentation::accessibility::Accessibility>,
) {
    use crate::presentation::accessibility::announcements::Priority;

    let asker = Dialog::builder(dialog, "Insert Table")
        .with_style(DialogStyle::DefaultDialogStyle)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(6)
        .with_hgap(8)
        .build();

    let rows_label = StaticText::builder(&asker).with_label("&Rows:").build();
    let rows = SpinCtrl::builder(&asker)
        .with_range(1, editor_document::MAX_TABLE_ROWS as i32)
        .with_initial_value(3)
        .build();
    set_accessible_name(&rows, "Rows");
    fields.add(
        &rows_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&rows, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let columns_label = StaticText::builder(&asker).with_label("&Columns:").build();
    let columns = SpinCtrl::builder(&asker)
        .with_range(1, editor_document::MAX_TABLE_COLUMNS as i32)
        .with_initial_value(3)
        .build();
    set_accessible_name(&columns, "Columns");
    fields.add(
        &columns_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&columns, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let header = CheckBox::builder(&asker)
        .with_label("First row is column &headers")
        .build();
    header.set_value(true);
    set_accessible_name(&header, "First row is column headers");
    sizer.add(&header, 0, SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&asker)
        .with_id(ID_OK)
        .with_label("&Insert")
        .build();
    let cancel = Button::builder(&asker)
        .with_id(ID_CANCEL)
        .with_label("Cancel")
        .build();
    buttons.add(&ok, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 4);

    asker.set_sizer_and_fit(sizer, true);
    rows.set_focus();
    let answer = asker.show_modal();
    let (rows, columns, header) = (
        rows.value().max(0) as usize,
        columns.value().max(0) as usize,
        header.get_value(),
    );
    asker.destroy();
    if answer != ID_OK {
        return;
    }
    match editor_document::insert_table_script(rows, columns, header) {
        Some(script) => {
            run_in_editor(&body_editor, &script);
            let _ = a11y.announce(
                &editor_document::table_spoken(rows, columns, header),
                Priority::Normal,
            );
        }
        // Refused out loud rather than silently doing nothing, which reads as
        // the command being broken.
        None => {
            let _ = a11y.announce(
                &format!(
                    "A table can be up to {} rows by {} columns.                      More than that cannot be read a cell at a time.",
                    editor_document::MAX_TABLE_ROWS,
                    editor_document::MAX_TABLE_COLUMNS
                ),
                Priority::High,
            );
        }
    }
}

fn insert_link(
    dialog: &Dialog,
    body_editor: WebView,
    a11y: &std::sync::Arc<crate::presentation::accessibility::Accessibility>,
) {
    use crate::presentation::accessibility::announcements::Priority;

    let entry = TextEntryDialog::builder(dialog, "Address to link to:", "Insert Link").build();
    let answer = entry.show_modal();
    let typed = entry.get_value();
    entry.destroy();
    if answer != ID_OK {
        return;
    }
    let Some(typed) = typed else {
        return;
    };
    if typed.trim().is_empty() {
        return;
    }
    match editor_document::link_script(&typed) {
        Some(script) => {
            run_in_editor(&body_editor, &script);
            let _ = a11y.announce("Link added", Priority::Normal);
        }
        // Refused out loud, naming what was refused. A link that silently does
        // not appear reads as the command being broken.
        None => {
            let _ = a11y.announce(
                &format!("{typed} is not an address this can link to"),
                Priority::High,
            );
        }
    }
}

// ── Preview Before Send ─────────────────────────────────────────────────────

enum PreviewDecision {
    ConfirmSend,
    GoBack,
}

// Clear of the formatting run, which reaches ID_HIGHEST + 149.
const ID_CONFIRM_SEND: Id = ID_HIGHEST + 180;
const ID_GO_BACK: Id = ID_HIGHEST + 181;

/// Show a read-only preview of the composed email before sending.
fn show_send_preview(
    parent: &Dialog,
    data: &ComposeData,
    account_names: &[String],
) -> PreviewDecision {
    let dlg = Dialog::builder(parent, "Preview Before Send")
        .with_size(650, 500)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Header summary
    let hdr = FlexGridSizer::builder(0, 2)
        .with_vgap(2)
        .with_hgap(8)
        .build();
    hdr.add_growable_col(1, 1);

    let from_display = data
        .account_index
        .and_then(|i| account_names.get(i as usize))
        .cloned()
        .unwrap_or_else(|| "(default account)".to_string());

    for (label, value) in [
        ("From:", from_display.as_str()),
        ("To:", &data.to),
        ("CC:", &data.cc),
        ("BCC:", &data.bcc),
        ("Subject:", &data.subject),
    ] {
        if (label == "CC:" || label == "BCC:") && value.is_empty() {
            continue;
        }
        let lbl = StaticText::builder(&dlg).with_label(label).build();
        let val = StaticText::builder(&dlg).with_label(value).build();
        hdr.add(&lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 2);
        hdr.add(&val, 1, SizerFlag::Expand | SizerFlag::All, 2);
    }

    sizer.add_sizer(&hdr, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // Separator line
    let sep = StaticText::builder(&dlg)
        .with_label("────────────────────────────────────────")
        .build();
    sizer.add(
        &sep,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right,
        8,
    );

    // Body preview (WebView, renders HTML formatting)
    let body_preview = WebView::builder(&dlg)
        .with_backend(WebViewBackend::Edge)
        .build();
    set_accessible_name(&body_preview, "Message preview");
    body_preview.set_name("Message preview");
    body_preview.enable_context_menu(false);
    body_preview.enable_access_to_dev_tools(false);
    let renderer = HtmlRenderer::new();
    // The editor's own markup coming back for a last look before it goes.
    let html = renderer.wrap_body(&MessageBody::Html(data.body.clone()));
    body_preview.set_page(&html, "about:blank");

    // Block navigation in preview: open links in default browser
    body_preview.on_navigating(|event: WebViewEventData| {
        if let Some(url) = event.get_string()
            && !url.is_empty()
            && url != "about:blank"
            && !url.starts_with("about:")
            && !url.starts_with("data:")
        {
            event.event.event.veto();
            if let Some(safe) = crate::presentation::HtmlRenderer::safe_external_url(&url) {
                let _ = open::that(&safe);
            } else {
                tracing::warn!("Refused to open unsafe URL from message: {}", url);
            }
        }
    });

    sizer.add(&body_preview, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // Buttons
    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let back_btn = Button::builder(&dlg)
        .with_label("&Go Back && Edit")
        .with_id(ID_GO_BACK)
        .build();
    let send_btn = Button::builder(&dlg)
        .with_label("Confirm &Send")
        .with_id(ID_CONFIRM_SEND)
        .build();
    btn_sizer.add(&back_btn, 0, SizerFlag::All, 4);
    btn_sizer.add_spacer(16);
    btn_sizer.add(&send_btn, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dlg.set_sizer(sizer, true);

    send_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CONFIRM_SEND);
        }
    });
    back_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_GO_BACK);
        }
    });

    let answer = dlg.show_modal();
    // With the browser control the preview renders into. One per message sent,
    // and the compose window holds another.
    dlg.destroy();
    if answer == ID_CONFIRM_SEND {
        PreviewDecision::ConfirmSend
    } else {
        PreviewDecision::GoBack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_formatting_run_fits_the_range_kept_for_it() {
        // The run is one id per Format::ALL, and Format::ALL grows. Without
        // this, adding a fourteenth command silently walks into whatever is
        // next.
        let last = ID_FORMAT_FIRST + editor_document::Format::ALL.len() as Id - 1;

        assert!(
            last <= ID_FORMAT_LAST,
            "the formatting run reaches {last} and only {ID_FORMAT_LAST} is kept for it"
        );
    }

    #[test]
    fn test_nothing_else_takes_an_id_the_formatting_run_needs() {
        // ID_CONFIRM_SEND was numerically identical to Bold and ID_GO_BACK to
        // Italic. It never fired, because the preview's buttons raise one kind
        // of event and the formatting handler listens for another, and nothing
        // anywhere enforced that. The run grew by thirteen entries in a single
        // change with nobody checking what was already in the range.
        for (name, id) in [
            ("ID_BOLD", ID_BOLD),
            ("ID_ITALIC", ID_ITALIC),
            ("ID_UNDERLINE", ID_UNDERLINE),
            ("ID_INSERT_LINK", ID_INSERT_LINK),
            ("ID_INSERT_TABLE", ID_INSERT_TABLE),
            ("ID_FORMAT_MENU", ID_FORMAT_MENU),
            ("ID_SPELL_CHECK", ID_SPELL_CHECK),
            ("ID_SPELL_CHANGE", ID_SPELL_CHANGE),
            ("ID_SPELL_CHANGE_ALL", ID_SPELL_CHANGE_ALL),
            ("ID_SPELL_IGNORE", ID_SPELL_IGNORE),
            ("ID_SPELL_IGNORE_ALL", ID_SPELL_IGNORE_ALL),
            ("ID_SPELL_ADD", ID_SPELL_ADD),
            ("ID_SEND", ID_SEND),
            ("ID_SAVE_DRAFT", ID_SAVE_DRAFT),
            ("ID_DISCARD", ID_DISCARD),
            ("ID_ATTACH", ID_ATTACH),
            ("ID_UNDO", ID_UNDO),
            ("ID_REDO", ID_REDO),
            ("ID_CONFIRM_SEND", ID_CONFIRM_SEND),
            ("ID_GO_BACK", ID_GO_BACK),
        ] {
            assert!(
                !(ID_FORMAT_FIRST..=ID_FORMAT_LAST).contains(&id),
                "{name} sits inside the range kept for the formatting run"
            );
        }
    }

    #[test]
    fn test_reply_subject_prepends_re() {
        assert_eq!(format_reply_subject("Hello"), "Re: Hello");
    }

    #[test]
    fn test_reply_subject_no_double_re() {
        assert_eq!(format_reply_subject("Re: Hello"), "Re: Hello");
    }

    #[test]
    fn test_forward_subject_prepends_fwd() {
        assert_eq!(format_forward_subject("Hello"), "Fwd: Hello");
    }

    #[test]
    fn test_forward_subject_no_double_fwd() {
        assert_eq!(format_forward_subject("Fwd: Hello"), "Fwd: Hello");
    }

    #[test]
    fn test_a_message_with_anything_in_it_is_worth_keeping() {
        // Escape reaches the compose dialog from anywhere in the window and
        // used to throw the message away with nothing asked and nothing kept.
        // Each of these on its own is somebody's work.
        for touched in [
            |d: &mut ComposeData| d.to = "sam@example.com".into(),
            |d: &mut ComposeData| d.cc = "ada@example.com".into(),
            |d: &mut ComposeData| d.bcc = "grace@example.com".into(),
            |d: &mut ComposeData| d.subject = "Tomorrow".into(),
            |d: &mut ComposeData| d.body_plain = "See you at nine.".into(),
        ] {
            let mut data = written("", "");
            data.to = String::new();
            data.subject = String::new();
            data.body_plain = String::new();
            touched(&mut data);

            assert!(worth_keeping(&data), "{data:?}");
        }
    }

    #[test]
    fn test_a_message_nobody_has_touched_is_not_kept() {
        // A blank row in the drafts list is one that announces nothing, on a
        // list somebody navigates by ear.
        let mut nothing = written("", "");
        nothing.to = String::new();
        nothing.subject = String::new();
        nothing.body_plain = String::new();

        assert!(!worth_keeping(&nothing));
    }

    #[test]
    fn test_markup_with_no_words_in_it_is_still_nothing() {
        // The editor hands back an empty paragraph for a message nobody has
        // typed into, so asking about the markup would keep every window ever
        // opened. The plain text half is what is asked.
        let mut empty = written("<p><br></p>", "");
        empty.to = String::new();
        empty.subject = String::new();

        assert!(!worth_keeping(&empty));
    }

    /// A message with both halves as the editor would hand them over.
    fn written(body: &str, body_plain: &str) -> ComposeData {
        ComposeData {
            to: "sam@example.com".to_string(),
            cc: String::new(),
            bcc: String::new(),
            subject: "Tomorrow".to_string(),
            body: body.to_string(),
            body_plain: body_plain.to_string(),
            html_mode: true,
            account_index: None,
            attachments: Vec::new(),
        }
    }

    #[test]
    fn test_the_send_check_reads_the_text_and_not_the_tags() {
        // The editor holds HTML, so the moment somebody presses Enter for a
        // second line the body is "Hello Sam<div>See you tomorrow</div>". A
        // spell checker has no idea what a tag is: it reports "Sam<div>See"
        // and "tomorrow</div", and a message with nothing wrong in it asks
        // "send anyway?".
        //
        // Which is the confirmation this feature exists not to become. Its own
        // doc comment says so: one that appears every time is one people learn
        // to dismiss without reading, and then it is not there the time it
        // mattered.
        let data = written(
            "Hello Sam<div>See you tomorrow</div>",
            "Hello Sam\nSee you tomorrow",
        );
        let speller = crate::service::spellcheck::SpellChecker::new();

        // Asserted against the markup rather than against the dictionary. What
        // is in the built-in word list is not the point and changes; a tag
        // reaching the question is the defect, whatever else it says. ("Sam"
        // is reported either way: it is a name, which is a separate argument.)
        let said = spelling_question(&data, &speller).unwrap_or_default();
        assert!(!said.contains("div"), "a tag was checked as a word: {said}");
        assert!(!said.contains('<'), "markup reached the question: {said}");
    }

    #[test]
    fn test_a_word_actually_spelled_wrong_still_stops_the_send() {
        // The other half. Reading the right field must not turn the check off.
        let data = written("<p>Please recieve this</p>", "Please recieve this");
        let speller = crate::service::spellcheck::SpellChecker::new();

        let said = spelling_question(&data, &speller).expect("a question");
        assert!(said.contains("recieve"), "{said}");
    }

    #[test]
    fn test_reply_body_adds_quote_header() {
        let body = format_reply_body(&MessageBody::Plain("Original text".into()));
        let MessageBody::Plain(text) = &body else {
            panic!("plain in, plain out, or the editor escapes the wrong half: {body:?}");
        };
        assert!(text.contains("--- Original Message ---"));
        assert!(text.contains("Original text"));
        assert!(text.starts_with("\n\n"));
    }

    #[test]
    fn test_forward_body_adds_forward_header() {
        let body = format_forward_body(&MessageBody::Plain("Forwarded text".into()));
        let MessageBody::Plain(text) = &body else {
            panic!("plain in, plain out: {body:?}");
        };
        assert!(text.contains("---------- Forwarded message ----------"));
        assert!(text.contains("Forwarded text"));
    }

    #[test]
    fn test_quoting_an_html_original_keeps_it_html() {
        // The kind has to survive the join. Deciding a body is text because a
        // marker was glued to the front of it is how the editor came to run a
        // stranger's markup through the wrong branch.
        let body = format_reply_body(&MessageBody::Html("<p>Original</p>".into()));
        let MessageBody::Html(html) = &body else {
            panic!("html in, html out: {body:?}");
        };
        assert!(html.contains("<p>--- Original Message ---</p>"), "{html}");
        assert!(html.contains("<p>Original</p>"), "{html}");
    }

    #[test]
    fn test_quoting_a_multipart_original_takes_the_markup() {
        // Both halves arrived. The editor is a live DOM, so the markup is the
        // one that keeps the sender's headings and link text.
        let body = format_forward_body(&MessageBody::Multipart {
            plain: "Original".into(),
            html: "<h2>Original</h2>".into(),
        });
        assert!(matches!(body, MessageBody::Html(ref h) if h.contains("<h2>Original</h2>")));
    }

    #[test]
    fn test_compose_title_for_each_mode() {
        assert_eq!(compose_title(&ComposeMode::New), "Compose New Message");
        assert_eq!(
            compose_title(&ComposeMode::Reply {
                to: String::new(),
                subject: String::new(),
                quoted_body: MessageBody::Plain(String::new()),
            }),
            "Reply"
        );
        assert_eq!(
            compose_title(&ComposeMode::ReplyAll {
                to: String::new(),
                cc: String::new(),
                subject: String::new(),
                quoted_body: MessageBody::Plain(String::new()),
            }),
            "Reply All"
        );
        assert_eq!(
            compose_title(&ComposeMode::Forward {
                subject: String::new(),
                body: MessageBody::Plain(String::new()),
            }),
            "Forward"
        );
        assert_eq!(
            compose_title(&ComposeMode::Draft(CompositionData::default())),
            "Edit Draft"
        );
    }

    #[test]
    fn test_reply_subject_empty_string() {
        assert_eq!(format_reply_subject(""), "Re: ");
    }

    #[test]
    fn test_forward_subject_empty_string() {
        assert_eq!(format_forward_subject(""), "Fwd: ");
    }
}
