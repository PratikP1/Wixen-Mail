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
//! wave it away. Continue is always available, though, so nobody is stuck.
//! What it takes is the second of the three, which leaves mail alone and sends
//! changes to tasks, contacts and the calendar up to a provider, so pressing
//! Enter immediately is a real decision rather than a safe one.

use crate::application::allowed::Allowed;
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::first_run::{Choice, INTRODUCTION, READ_MORE, TESTING_PAGE, TITLE};
use crate::presentation::help_page;
use crate::presentation::theme;
use wxdragon::prelude::*;

const ID_CONTINUE: Id = ID_HIGHEST + 300;
const ID_READ_MORE: Id = ID_HIGHEST + 301;

/// Ask what Wixen Mail may change, and give back the answer.
///
/// Modal, and it does not come back until somebody has chosen.
pub fn ask_what_is_allowed(parent: &Frame) -> Allowed {
    let (dialog, buttons) = build_first_run_dialog(parent, theme::current_from_stored_config());

    dialog.show_modal();
    let chosen = buttons
        .iter()
        .find(|(_, button)| button.get_value())
        .map_or(Choice::DEFAULT, |(choice, _)| *choice);
    dialog.destroy();

    chosen.allows()
}

/// Build the first-run "what may Wixen Mail change" dialog without showing
/// it.
///
/// Everything `ask_what_is_allowed` used to do up to its own `.show_modal()`
/// call, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// Returns the radio buttons alongside the dialog, the same way the caller
/// needs them after a real `.show_modal()`: to read which one is ticked.
pub fn build_first_run_dialog(
    parent: &Frame,
    palette: Option<theme::Palette>,
) -> (Dialog, Vec<(Choice, RadioButton)>) {
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
    // It is in the accessibility tree twice, and nothing here can take one
    // of them out. There is no way through this binding to say that a piece
    // of text belongs to a control: what `set_accessible_name` writes is a
    // name, and a static text's name is its own words whether anything sets
    // one or not. So what each answer costs is heard as the button's
    // description when the button takes focus, and again as the text under
    // it when a screen reader reads the window from the top.
    //
    // Both are kept on purpose. The description is the copy that reaches a
    // reader working through Microsoft Active Accessibility at the moment
    // the decision is being made, and it was missing once: three buttons
    // that read correctly and three explanations of what each one costs that
    // nobody ever heard. The words on screen are the only copy a sighted
    // reader gets, and the only copy a reader working through UI Automation
    // gets, because a description set here never reaches that tree at all.
    // Dropping either takes the sentence away from somebody, so what it
    // costs is hearing it twice on the way down the window.
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

    // Focus on the answer that is ticked, rather than on Continue, so what is
    // heard is the decision rather than the way past it and it is the answer
    // Continue will take.
    //
    // Read back from the buttons rather than worked out a second time, so the
    // two cannot come apart. They had: the screen read out one answer and
    // ticked another, so somebody using a screen reader heard the cautious one,
    // pressed Enter on the first thing they heard, and switched on writing to
    // their real address book, calendar and tasks.
    if let Some((_, ticked)) = buttons.iter().find(|(_, button)| button.get_value()) {
        ticked.set_focus();
    }

    // Painted last. `None` means high contrast is on, or the system is set
    // up in a way this application should not paint over, so nothing is set
    // here and Windows decides. No `TextCtrl`, `ListCtrl` or `TreeCtrl`
    // anywhere in this dialog, so the dialog itself is the only site: the
    // radio buttons, like every checkbox and button elsewhere in this round,
    // are left to Windows.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }

    (dialog, buttons)
}
