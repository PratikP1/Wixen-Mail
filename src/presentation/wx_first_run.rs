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
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::first_run::{Choice, INTRODUCTION, READ_MORE, TESTING_PAGE, TITLE};
use crate::presentation::help_page;
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

    // One radio button per choice, each followed by what it costs.
    //
    // The explanation is both the button's accessible description and a label
    // on screen. The description is what a screen reader reads on focus, so it
    // is heard by the person deciding; the label is what a sighted person
    // reads. It is not part of the accessible name, because a name is repeated
    // in every announcement about the control and a four-sentence one is read
    // in full each time somebody arrows past.
    //
    // The visible label is marked as belonging to the button rather than
    // named in its own right, so it is not read twice: once as the button's
    // description and again as the text underneath it.
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
        set_accessible_name_and_description(&button, choice.label(), choice.explanation());
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
        // Converted to HTML and opened in a browser, rather than handed over
        // as Markdown. Windows gives a `.md` file to a text editor or to
        // nothing at all, and Markdown source read aloud is hash signs and
        // square brackets, with the headings that make a long page navigable
        // reduced to punctuation.
        //
        // A browser rather than a window of our own: it already has heading
        // navigation, find and zoom, and every screen reader knows it well.
        if let Err(e) = help_page::open(TESTING_PAGE) {
            tracing::warn!("Could not open the testing page: {e}");
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
