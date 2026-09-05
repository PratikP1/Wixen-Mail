//! The window that asks which copy of a contact or a calendar item to keep.
//!
//! Both copies are shown, each introduced by which copy it is, with the fields
//! they disagree about named. Nothing here decides anything: the wording, the
//! comparison and what a press means are all in
//! [`crate::application::conflict_choice`], where they can be read back by a
//! test without a running window.
//!
//! # Reaching it and leaving it
//!
//! Modal, and it is not modal for the reason the reminder window is. A reminder
//! interrupts; this does not. It is modal because two copies of one contact
//! being edited from two windows at once is a question about a question, and
//! because the sync that raised it says plainly where to go.
//!
//! Closing it is an answer and the answer is "not yet". The hold stays, nothing
//! is written and nothing is sent, so somebody who opened it by accident, or
//! who wants to look at the two copies somewhere else first, loses nothing.
//!
//! # What has never been checked
//!
//! Nobody has heard this. Whether the labelled pair is understood by ear,
//! whether announcing each version as focus reaches it is better than reading
//! both on opening, and whether the list of differing fields is useful or is a
//! sentence somebody stops hearing, are all in the broken windows ledger as
//! `unrun-verify` rather than claimed here.

use crate::application::conflict_choice::{BothCopies, WhatWasPressed, WhichCopy};
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::theme;
use wxdragon::prelude::*;

const ID_KEEP_WHAT_IS_HERE: Id = ID_HIGHEST + 431;
const ID_KEEP_THEIRS: Id = ID_HIGHEST + 432;
const ID_LEAVE_IT: Id = ID_HIGHEST + 433;

/// What the button that keeps this computer's copy says.
///
/// A label that says what pressing it does rather than naming a side. "This
/// computer" alone is a noun, and a button whose label is a noun leaves
/// somebody working by ear to work out what the verb was.
pub const KEEP_WHAT_IS_HERE: &str = "&Keep what is on this computer";

/// What the button that keeps the provider's copy says, with a hole for whose
/// copy it is. One string with a hole rather than two, for the same reason the
/// sentences are built that way.
pub const KEEP_THEIRS: &str = "Keep what {} has";

/// What the button that answers "not yet" says.
pub const LEAVE_IT: &str = "&Decide later";

/// The heading over the list of what one copy holds.
pub const THE_FIELDS_COLUMN: &str = "Field";

/// The heading over the values.
pub const THE_VALUES_COLUMN: &str = "Value";

/// Ask which copy to keep, and wait for an answer.
pub fn ask_which_copy_to_keep(parent: &Frame, copies: &BothCopies) -> Option<WhichCopy> {
    let dialog = build_the_choosing_dialog(parent, copies, theme::current_from_stored_config());
    let answer = dialog.show_modal();
    dialog.destroy();
    crate::application::conflict_choice::what_that_means(match answer {
        id if id == ID_KEEP_WHAT_IS_HERE => WhatWasPressed::KeepWhatIsHere,
        id if id == ID_KEEP_THEIRS => WhatWasPressed::KeepTheirs,
        // Escape, the close box, and the button that says so, all one answer.
        // Falling through to either copy winning would be a choice made by
        // whoever wrote this match rather than by the person.
        _ => WhatWasPressed::LeftWithoutChoosing,
    })
}

/// Build the window without showing it, so a test can read back what a live
/// control really holds without a modal loop.
///
/// The same split `wx_reminder_alert::build_reminder_alert_dialog` makes, and
/// for the same reason.
pub fn build_the_choosing_dialog(
    parent: &Frame,
    copies: &BothCopies,
    palette: Option<theme::Palette>,
) -> Dialog {
    let dialog = Dialog::builder(parent, "Which copy do you want to keep?")
        .with_size(640, 520)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // The one sentence said on opening: what is being asked and how many
    // fields disagree. Built in the application layer so it can be read back
    // without a window.
    let asked = copies.what_is_being_asked();
    let question = StaticText::builder(&dialog).with_label(&asked).build();
    question.set_name(&asked);
    sizer.add(&question, 0, SizerFlag::Expand | SizerFlag::All, 12);

    for which in [WhichCopy::Here, WhichCopy::TheProviders] {
        let label = copies.label_for(which);
        let heading = StaticText::builder(&dialog).with_label(&label).build();
        heading.set_name(&label);
        sizer.add(&heading, 0, SizerFlag::Left | SizerFlag::All, 8);

        let values = ListCtrl::builder(&dialog)
            .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
            .build();
        values.set_name(&label);
        values.insert_column(0, THE_FIELDS_COLUMN, ListColumnFormat::Left, 160);
        values.insert_column(1, THE_VALUES_COLUMN, ListColumnFormat::Left, 400);
        for (row, field) in copies.values_in(which).iter().enumerate() {
            values.insert_item(row as i64, &field.called, None);
            values.set_item_text_by_column(row as i64, 1, &field.value);
        }
        sizer.add(&values, 1, SizerFlag::Expand | SizerFlag::All, 8);
    }

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let keep_here = Button::builder(&dialog)
        .with_label(KEEP_WHAT_IS_HERE)
        .with_id(ID_KEEP_WHAT_IS_HERE)
        .build();
    keep_here.set_name(KEEP_WHAT_IS_HERE);
    let theirs_label = KEEP_THEIRS.replace("{}", copies.other_copy.called());
    let keep_theirs = Button::builder(&dialog)
        .with_label(&theirs_label)
        .with_id(ID_KEEP_THEIRS)
        .build();
    keep_theirs.set_name(&theirs_label);
    let leave_it = Button::builder(&dialog)
        .with_label(LEAVE_IT)
        .with_id(ID_LEAVE_IT)
        .build();
    leave_it.set_name(LEAVE_IT);
    buttons.add(&keep_here, 0, SizerFlag::All, 4);
    buttons.add(&keep_theirs, 0, SizerFlag::All, 4);
    buttons.add(&leave_it, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(sizer, true);
    // The question first, so what is being asked is the first thing read, and
    // so nothing is the default answer.
    let _ = set_accessible_name;
    leave_it.set_focus();
    dialog
}
