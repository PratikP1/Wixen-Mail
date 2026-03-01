//! wxdragon Composition Dialog
//!
//! Provides a modal dialog for composing, replying to, and forwarding emails.
//! Uses RichTextCtrl for the message body with formatting toolbar support.

use crate::presentation::ui_types::CompositionData;
use wxdragon::prelude::*;

// ── Formatting toolbar IDs ──────────────────────────────────────────────────

// Button IDs: used as return codes via end_modal() or in button .with_id()
const ID_BOLD: Id = ID_HIGHEST + 100;
const ID_ITALIC: Id = ID_HIGHEST + 101;
const ID_UNDERLINE: Id = ID_HIGHEST + 102;
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
    pub body: String,
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
        quoted_body: String,
    },
    /// Reply to all recipients
    ReplyAll {
        to: String,
        cc: String,
        subject: String,
        quoted_body: String,
    },
    /// Forward a message
    Forward {
        subject: String,
        body: String,
    },
    /// Edit an existing draft
    Draft(CompositionData),
}

/// Show the composition dialog modally and return the user's action.
///
/// This creates a Dialog with:
/// - To/CC/BCC/Subject text fields
/// - Account selector (Choice dropdown)
/// - RichTextCtrl body editor with B/I/U formatting buttons
/// - Send, Save Draft, Discard action buttons
pub fn show_compose_dialog(
    parent: &Frame,
    mode: ComposeMode,
    account_names: &[String],
    active_account_index: u32,
) -> ComposeResult {
    show_compose_dialog_with_options(parent, mode, account_names, active_account_index, true)
}

/// Show the composition dialog with configurable preview-before-send.
pub fn show_compose_dialog_with_options(
    parent: &Frame,
    mode: ComposeMode,
    account_names: &[String],
    active_account_index: u32,
    preview_before_send: bool,
) -> ComposeResult {
    // ── Create Dialog ────────────────────────────────────────────────────
    let title = match &mode {
        ComposeMode::New => "Compose New Message",
        ComposeMode::Reply { .. } => "Reply",
        ComposeMode::ReplyAll { .. } => "Reply All",
        ComposeMode::Forward { .. } => "Forward",
        ComposeMode::Draft(_) => "Edit Draft",
    };

    let dialog = Dialog::builder(parent, title)
        .with_size(850, 700)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder | DialogStyle::MaximizeBox)
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
    let account_label = StaticText::builder(&dialog)
        .with_label("&From:")
        .build();
    let account_choice = Choice::builder(&dialog)
        .with_choices(account_names.iter().map(|s| s.to_string()).collect())
        .with_selection(Some(active_account_index))
        .build();
    fields_sizer.add(&account_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields_sizer.add(&account_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // To field
    let to_label = StaticText::builder(&dialog).with_label("&To:").build();
    let to_field = TextCtrl::builder(&dialog).build();
    fields_sizer.add(&to_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields_sizer.add(&to_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // CC field
    let cc_label = StaticText::builder(&dialog).with_label("&CC:").build();
    let cc_field = TextCtrl::builder(&dialog).build();
    fields_sizer.add(&cc_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields_sizer.add(&cc_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // BCC field
    let bcc_label = StaticText::builder(&dialog).with_label("&BCC:").build();
    let bcc_field = TextCtrl::builder(&dialog).build();
    fields_sizer.add(&bcc_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields_sizer.add(&bcc_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Subject field
    let subject_label = StaticText::builder(&dialog).with_label("Su&bject:").build();
    let subject_field = TextCtrl::builder(&dialog).build();
    fields_sizer.add(&subject_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields_sizer.add(&subject_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    main_sizer.add_sizer(&fields_sizer, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // -- Compose toolbar --
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();

    // Prominent Send button (Outlook-style — first in toolbar)
    let send_toolbar_btn = Button::builder(&dialog)
        .with_label("&Send")
        .with_id(ID_SEND)
        .with_size(Size::new(72, 30))
        .build();
    toolbar_sizer.add(&send_toolbar_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Undo / Redo
    let undo_btn = Button::builder(&dialog)
        .with_label("&Undo")
        .with_id(ID_UNDO)
        .with_size(Size::new(52, 28))
        .build();
    let redo_btn = Button::builder(&dialog)
        .with_label("&Redo")
        .with_id(ID_REDO)
        .with_size(Size::new(52, 28))
        .build();
    toolbar_sizer.add(&undo_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&redo_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Formatting: Bold, Italic, Underline
    let bold_btn = Button::builder(&dialog)
        .with_label("B")
        .with_id(ID_BOLD)
        .with_size(Size::new(32, 28))
        .build();
    let italic_btn = Button::builder(&dialog)
        .with_label("I")
        .with_id(ID_ITALIC)
        .with_size(Size::new(32, 28))
        .build();
    let underline_btn = Button::builder(&dialog)
        .with_label("U")
        .with_id(ID_UNDERLINE)
        .with_size(Size::new(32, 28))
        .build();
    toolbar_sizer.add(&bold_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&italic_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add(&underline_btn, 0, SizerFlag::All, 2);
    toolbar_sizer.add_spacer(12);

    // Attach
    let attach_btn = Button::builder(&dialog)
        .with_label("A&ttach...")
        .with_id(ID_ATTACH)
        .build();
    toolbar_sizer.add(&attach_btn, 0, SizerFlag::All, 2);

    main_sizer.add_sizer(&toolbar_sizer, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right, 8);

    // -- Rich text body --
    let body_editor = RichTextCtrl::builder(&dialog)
        .with_style(RichTextCtrlStyle::MultiLine | RichTextCtrlStyle::WordWrap)
        .build();

    main_sizer.add(&body_editor, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Attachment list (initially hidden) --
    let attachment_label = StaticText::builder(&dialog)
        .with_label("No attachments")
        .build();
    main_sizer.add(&attachment_label, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right, 8);

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
        .with_label("&Cancel")
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
            let subj = if subject.starts_with("Re: ") {
                subject.clone()
            } else {
                format!("Re: {}", subject)
            };
            subject_field.set_value(&subj);
            body_editor.set_value(&format!("\n\n--- Original Message ---\n{}", quoted_body));
            body_editor.set_insertion_point(0);
        }
        ComposeMode::ReplyAll {
            to,
            cc,
            subject,
            quoted_body,
        } => {
            to_field.set_value(to);
            cc_field.set_value(cc);
            let subj = if subject.starts_with("Re: ") {
                subject.clone()
            } else {
                format!("Re: {}", subject)
            };
            subject_field.set_value(&subj);
            body_editor.set_value(&format!("\n\n--- Original Message ---\n{}", quoted_body));
            body_editor.set_insertion_point(0);
        }
        ComposeMode::Forward { subject, body } => {
            let subj = if subject.starts_with("Fwd: ") {
                subject.clone()
            } else {
                format!("Fwd: {}", subject)
            };
            subject_field.set_value(&subj);
            body_editor.set_value(&format!(
                "\n\n---------- Forwarded message ----------\n{}",
                body
            ));
            body_editor.set_insertion_point(0);
            // Focus the To field since user needs to fill it
            to_field.set_focus();
        }
        ComposeMode::Draft(data) => {
            to_field.set_value(&data.to);
            cc_field.set_value(&data.cc);
            bcc_field.set_value(&data.bcc);
            subject_field.set_value(&data.subject);
            body_editor.set_value(&data.body);
        }
    }

    // ── Wire formatting button events ────────────────────────────────────
    bold_btn.on_click({
        let body_editor = body_editor;
        move |_| {
            body_editor.apply_bold_to_selection();
        }
    });

    italic_btn.on_click({
        let body_editor = body_editor;
        move |_| {
            body_editor.apply_italic_to_selection();
        }
    });

    underline_btn.on_click({
        let body_editor = body_editor;
        move |_| {
            body_editor.apply_underline_to_selection();
        }
    });

    // Send button (in toolbar) closes dialog with ID_SEND
    send_toolbar_btn.on_click({
        let dialog = dialog;
        move |_| {
            dialog.end_modal(ID_SEND);
        }
    });

    // Undo / Redo
    undo_btn.on_click({
        let body_editor = body_editor;
        move |_| {
            body_editor.undo();
        }
    });
    redo_btn.on_click({
        let body_editor = body_editor;
        move |_| {
            body_editor.redo();
        }
    });

    // Save Draft
    draft_btn.on_click({
        let dialog = dialog;
        move |_| {
            dialog.end_modal(ID_SAVE_DRAFT);
        }
    });

    // Discard
    discard_btn.on_click({
        let dialog = dialog;
        move |_| {
            dialog.end_modal(ID_DISCARD);
        }
    });

    // Cancel
    cancel_btn.on_click({
        let dialog = dialog;
        move |_| {
            dialog.end_modal(ID_CANCEL);
        }
    });

    // ── Show dialog modally (loop for preview-then-send) ───────────────
    loop {
        let result = dialog.show_modal();

        let data = ComposeData {
            to: to_field.get_value(),
            cc: cc_field.get_value(),
            bcc: bcc_field.get_value(),
            subject: subject_field.get_value(),
            body: body_editor.get_value(),
            html_mode: true, // RichTextCtrl is always rich text
            account_index: account_choice.get_selection(),
        };

        match result {
            _ if result == ID_SEND => {
                if data.to.trim().is_empty() {
                    tracing::warn!("Send attempted with empty To field");
                    return ComposeResult::Cancelled;
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
            _ if result == ID_SAVE_DRAFT => return ComposeResult::SaveDraft(data),
            _ => return ComposeResult::Cancelled,
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
    let hdr = FlexGridSizer::builder(0, 2).with_vgap(2).with_hgap(8).build();
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
    sizer.add(&sep, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right, 8);

    // Body preview (read-only)
    let body_preview = TextCtrl::builder(&dlg)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
        .with_value(&data.body)
        .build();
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
        move |_| { d.end_modal(ID_CONFIRM_SEND); }
    });
    back_btn.on_click({
        let d = dlg;
        move |_| { d.end_modal(ID_GO_BACK); }
    });

    if dlg.show_modal() == ID_CONFIRM_SEND {
        PreviewDecision::ConfirmSend
    } else {
        PreviewDecision::GoBack
    }
}
