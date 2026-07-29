//! The window that asks, once, what Wixen Mail may change.
//!
//! The words are in [`crate::presentation::first_run`], where they can be read
//! by a test. This is the window around them: a heading, the introduction, one
//! radio button per choice with its explanation, a button that opens the
//! testing page, and Continue.
//!
//! # Why it cannot be dismissed without answering
//!
//! There is no Cancel and Escape does not close it. The whole point is that
//! somebody decided, and a dialog that can be waved away teaches people to
//! wave it away. Continue is always available, though, and starts on the safe
//! answer, so nobody is stuck: pressing Enter immediately is a valid decision
//! and a cautious one.

use crate::application::allowed::Allowed;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::first_run::{Choice, INTRODUCTION, READ_MORE, TESTING_PAGE, TITLE};
use wxdragon::prelude::*;

const ID_CONTINUE: Id = ID_HIGHEST + 300;
const ID_READ_MORE: Id = ID_HIGHEST + 301;

/// Ask what Wixen Mail may change, and give back the answer.
///
/// Modal, and it does not come back until somebody has chosen.
pub fn ask_what_is_allowed(parent: &Frame) -> Allowed {
    let dialog = Dialog::builder(parent, TITLE)
        .with_size(620, 560)
        .with_style(DialogStyle::Caption | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // A heading rather than only a window title, so it is reachable by the
    // heading key and read as the top of the content.
    let heading = StaticText::builder(&dialog).with_label(TITLE).build();
    set_accessible_name(&heading, TITLE);
    sizer.add(&heading, 0, SizerFlag::All, 10);

    let introduction = StaticText::builder(&dialog)
        .with_label(INTRODUCTION)
        .build();
    set_accessible_name(&introduction, INTRODUCTION);
    sizer.add(&introduction, 0, SizerFlag::Expand | SizerFlag::All, 10);

    // One radio button per choice, each followed by what it costs. The
    // explanation is a separate label rather than part of the button, because
    // a radio button whose name is four sentences long is read out in full
    // every time somebody arrows past it.
    let group = StaticBoxSizerBuilder::new_with_label(
        Orientation::Vertical,
        &dialog,
        "What may Wixen Mail change?",
    )
    .build();

    let mut buttons = Vec::new();
    for (index, choice) in Choice::ALL.iter().enumerate() {
        let button = RadioButton::builder(&dialog)
            .with_label(choice.label())
            .with_style(if index == 0 {
                // Marks the start of the group, so the three behave as one set
                // and the arrow keys move between them rather than off to the
                // next control.
                RadioButtonStyle::GroupStart
            } else {
                RadioButtonStyle::Default
            })
            .build();
        set_accessible_name(&button, choice.label());
        button.set_value(*choice == Choice::DEFAULT);
        group.add(&button, 0, SizerFlag::All, 6);

        let explanation = StaticText::builder(&dialog)
            .with_label(choice.explanation())
            .build();
        set_accessible_name(&explanation, choice.explanation());
        group.add(&explanation, 0, SizerFlag::Expand | SizerFlag::Left, 24);

        buttons.push((*choice, button));
    }
    sizer.add_sizer(&group, 1, SizerFlag::Expand | SizerFlag::All, 10);

    let read_more = Button::builder(&dialog)
        .with_label(READ_MORE)
        .with_id(ID_READ_MORE)
        .build();
    set_accessible_name(&read_more, READ_MORE);
    sizer.add(&read_more, 0, SizerFlag::All, 10);

    let carry_on = Button::builder(&dialog)
        .with_label("&Continue")
        .with_id(ID_CONTINUE)
        .build();
    set_accessible_name(&carry_on, "Continue");
    sizer.add(&carry_on, 0, SizerFlag::AlignRight | SizerFlag::All, 10);

    dialog.set_sizer(sizer, true);

    read_more.on_click(move |_| {
        // The page ships beside the program. Opening it in whatever reads
        // markdown beats rendering it here: it is long, and somebody may want
        // to keep it open next to the application.
        if let Err(e) = open::that(TESTING_PAGE) {
            tracing::warn!("Could not open {TESTING_PAGE}: {e}");
        }
    });
    carry_on.on_click(move |_| dialog.end_modal(ID_CONTINUE));

    // Focus on the first choice rather than on Continue, so the first thing
    // heard is the decision rather than the way past it.
    if let Some((_, first)) = buttons.first() {
        first.set_focus();
    }

    dialog.show_modal();
    let chosen = buttons
        .iter()
        .find(|(_, button)| button.get_value())
        .map_or(Choice::DEFAULT, |(choice, _)| *choice);
    dialog.destroy();

    chosen.allows()
}
