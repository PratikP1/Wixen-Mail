//! wxdragon Composition Dialog
//!
//! Provides a modal dialog for composing, replying to, and forwarding emails.
//! The message body is a `WebView` holding a contenteditable page, built by
//! [`crate::presentation::editor_document`].

use crate::common::types::MessageBody;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::editor_document;
use crate::presentation::html_renderer::HtmlRenderer;
use crate::presentation::ui_types::CompositionData;
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

/// Format a reply subject line — prepends "Re: " unless already present.
fn format_reply_subject(subject: &str) -> String {
    if subject.starts_with("Re: ") {
        subject.to_string()
    } else {
        format!("Re: {}", subject)
    }
}

/// Format a forward subject line — prepends "Fwd: " unless already present.
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

    // Account selector
    let account_label = StaticText::builder(&dialog).with_label("&From:").build();
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
    let to_label = StaticText::builder(&dialog).with_label("&To:").build();
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
    let cc_label = StaticText::builder(&dialog).with_label("&CC:").build();
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
    let bcc_label = StaticText::builder(&dialog).with_label("&BCC:").build();
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
    let subject_label = StaticText::builder(&dialog).with_label("&Subject:").build();
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

    // Prominent Send button (Outlook-style — first in toolbar)
    let send_toolbar_btn = Button::builder(&dialog)
        .with_label("Se&nd")
        .with_id(ID_SEND)
        .with_size(Size::new(72, 30))
        .build();
    send_toolbar_btn.set_name("Send message (Ctrl+Enter)");
    toolbar_sizer.add(&send_toolbar_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Undo / Redo
    let undo_btn = Button::builder(&dialog)
        .with_label("&Undo")
        .with_id(ID_UNDO)
        .with_size(Size::new(52, 28))
        .build();
    undo_btn.set_name("Undo (Ctrl+Z)");
    let redo_btn = Button::builder(&dialog)
        .with_label("&Redo")
        .with_id(ID_REDO)
        .with_size(Size::new(52, 28))
        .build();
    redo_btn.set_name("Redo (Ctrl+Y)");
    toolbar_sizer.add(&undo_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&redo_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Formatting: Bold, Italic, Underline — accessible labels for screen readers
    let bold_btn = Button::builder(&dialog)
        .with_label("B")
        .with_id(ID_BOLD)
        .with_size(Size::new(32, 28))
        .build();
    bold_btn.set_name("Bold (Ctrl+B)");
    let italic_btn = Button::builder(&dialog)
        .with_label("I")
        .with_id(ID_ITALIC)
        .with_size(Size::new(32, 28))
        .build();
    italic_btn.set_name("Italic (Ctrl+I)");
    let underline_btn = Button::builder(&dialog)
        .with_label("U")
        .with_id(ID_UNDERLINE)
        .with_size(Size::new(32, 28))
        .build();
    underline_btn.set_name("Underline (Ctrl+U)");
    // The other eight commands, and the link. Named as a word rather than a
    // symbol: B, I and U are recognisable letters, and there is no glyph for
    // "heading level 2" that anybody reads the same way.
    let format_btn = Button::builder(&dialog)
        .with_label("F&ormat...")
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
        .with_label("&Spelling")
        .with_id(ID_SPELL_CHECK)
        .build();
    set_accessible_name(&spell_btn, "Check spelling, F7");
    toolbar_sizer.add(&spell_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Attach
    let attach_btn = Button::builder(&dialog)
        .with_label("Attach F&ile...")
        .with_id(ID_ATTACH)
        .build();
    attach_btn.set_name("Attach file");
    toolbar_sizer.add(&attach_btn, 0, SizerFlag::All, 2);

    // Remove all toolbar buttons from tab order — accessible via keyboard shortcuts
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

    // -- Attachment list (initially hidden) --
    let attachment_label = StaticText::builder(&dialog)
        .with_label("No attachments")
        .build();
    main_sizer.add(
        &attachment_label,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right,
        8,
    );

    // -- Action buttons (Send is in toolbar above) --
    let button_sizer = BoxSizer::builder(Orientation::Horizontal).build();

    let draft_btn = Button::builder(&dialog)
        .with_label("Save &Draft")
        .with_id(ID_SAVE_DRAFT)
        .build();
    let discard_btn = Button::builder(&dialog)
        .with_label("Disc&ard")
        .with_id(ID_DISCARD)
        .build();
    let cancel_btn = Button::builder(&dialog)
        .with_label("Cance&l")
        .with_id(ID_CANCEL)
        .build();

    button_sizer.add_spacer(0); // Push buttons right
    button_sizer.add(&draft_btn, 0, SizerFlag::All, 4);
    button_sizer.add(&discard_btn, 0, SizerFlag::All, 4);
    button_sizer.add(&cancel_btn, 0, SizerFlag::All, 4);

    main_sizer.add_sizer(&button_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(main_sizer, true);

    // ── Pre-populate fields based on mode ────────────────────────────────
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

    format_btn.on_click(move |_| {
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

    // F7 typed in the page arrives inside the web view's own callback, and the
    // check it starts opens modal dialogs. Made once here, because a timer has
    // to outlive the callback that starts it, and started on demand below.
    let spell_later = std::rc::Rc::new(Timer::new(&dialog));
    spell_later.on_tick({
        let a11y = a11y.clone();
        move |_| check_spelling(&dialog, body_editor, &a11y)
    });

    body_editor.on_script_message_received({
        let a11y = a11y.clone();
        let typing_speller = typing_speller.clone();
        let spell_later = spell_later.clone();
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
                    spell_later.start(1, true);
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
    let read_compose_data = move || {
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
        })
    };

    // Held for the life of the dialog: dropping the timer stops it, and the
    // dialog is what it belongs to.
    let _autosave_timer = autosave.interval().map(|every| {
        let timer = Timer::new(&dialog);
        timer.on_tick(move |_| {
            // Could not read it, so there is nothing to save that is better
            // than what is already saved. Silent to the user on purpose: this
            // fires every couple of minutes and the message is untouched, so
            // there is nothing for them to do. The log is where it belongs.
            let Some(data) = read_compose_data() else {
                tracing::warn!("Autosave skipped: the editor did not answer");
                return;
            };
            // Nothing typed yet is not worth a row in the drafts list, and
            // it would be one that reads as blank.
            if data.to.trim().is_empty()
                && data.subject.trim().is_empty()
                && data.body.trim().is_empty()
            {
                return;
            }
            on_autosave(&data);
        });
        // Milliseconds, and the cast is safe: the range stops at ten minutes.
        timer.start(every.as_millis() as i32, false);
        timer
    });

    // ── Show dialog modally (loop for preview-then-send) ───────────────
    loop {
        let result = dialog.show_modal();

        // Nothing to read for the ways out that discard the message.
        if result != ID_SEND && result != ID_SAVE_DRAFT {
            return ComposeResult::Cancelled;
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
                    return ComposeResult::Cancelled;
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
                        PreviewDecision::ConfirmSend => return ComposeResult::Send(data),
                        PreviewDecision::GoBack => continue, // re-show compose dialog
                    }
                } else {
                    return ComposeResult::Send(data);
                }
            }
            // The only other way here: the guard above returns for every
            // result that is neither of these two.
            _ => return ComposeResult::SaveDraft(data),
        }
    }
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
    let answer = MessageDialog::builder(
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
    .build()
    .show_modal();

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
    if asker.show_modal() != ID_OK {
        return;
    }

    let (rows, columns, header) = (
        rows.value().max(0) as usize,
        columns.value().max(0) as usize,
        header.get_value(),
    );
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
    if entry.show_modal() != ID_OK {
        return;
    }
    let Some(typed) = entry.get_value() else {
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

const ID_CONFIRM_SEND: Id = ID_HIGHEST + 120;
const ID_GO_BACK: Id = ID_HIGHEST + 121;

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

    // Body preview (WebView — renders HTML formatting)
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

    // Block navigation in preview — open links in default browser
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

    if dlg.show_modal() == ID_CONFIRM_SEND {
        PreviewDecision::ConfirmSend
    } else {
        PreviewDecision::GoBack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
