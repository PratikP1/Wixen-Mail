//! The window that asks whether a change is meant for one day or all of them.
//!
//! The words are in [`crate::application::calendar`], where they can be read by
//! a test. This is the window around them: the event and how often it comes
//! round, one radio button per answer with what that answer will do, and
//! Continue.
//!
//! # Why it is a set of radio buttons and not three push buttons
//!
//! It was three push buttons, and a push button cannot be ticked. So the
//! question opened with nothing chosen and focus on Cancel, and the answer that
//! would be taken was nowhere in what a screen reader read out. A radio group
//! can say which answer is already chosen, and a group with a label of its own
//! puts the question itself in front of the answers rather than leaving it as a
//! line of text somebody has to go looking for.
//!
//! Focus lands on the answer that is ticked, so the first thing heard is the
//! answer that will be taken. That is the rule the first-run screen already
//! keeps, and it broke there once: focus on one answer and the tick on another
//! means somebody hears one thing and gets another.
//!
//! # Why Enter does not answer it
//!
//! One of the answers changes every day of somebody's series and the other
//! writes two entries to their calendar. Cancel is the default button, so Enter
//! pressed partway through hearing the question changes nothing. Continue is
//! one Tab away, and it takes whatever is ticked.

use crate::application::calendar::{
    EditMeans, WhatIsBeingDone, WhatTheCalendarAllows, a_repeat_kept_here_only, asking_is_needed,
    what_it_will_do,
};
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use wxdragon::prelude::*;

/// What the group of answers is called, and the question itself.
const THE_QUESTION: &str = "Which days do you mean?";

/// Ask whether a change is meant for one day or for the whole series.
///
/// `Some` with the answer, or `None` if it was called off. An event that does
/// not repeat is not asked about at all: there is only one day, so there is
/// nothing to choose between, and a question with one true answer is a question
/// nobody should be made to read.
///
/// `done` is which door somebody came through. A delete keeps nothing, so
/// describing it in the words written for an edit told somebody taking one day
/// off that the day would be kept as an appointment of its own.
pub fn which_days_are_meant(
    parent: &dyn WxWidget,
    summary: &str,
    repeats: &str,
    done: WhatIsBeingDone,
    allows: &WhatTheCalendarAllows,
) -> Option<EditMeans> {
    if !asking_is_needed(repeats) {
        return Some(EditMeans::WholeSeries);
    }

    let dialog = Dialog::builder(parent, "This event repeats")
        .with_size(560, 420)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let opening = format!("\"{summary}\" repeats: {repeats}.");
    let heading = StaticText::builder(&dialog).with_label(&opening).build();
    set_accessible_name(&heading, &opening);
    sizer.add(&heading, 0, SizerFlag::Expand | SizerFlag::All, 10);

    // Said before the answers, because it changes what both of them mean. An
    // event that repeats only on this computer is one appointment at the
    // calendar it is filed in, so a question about which days somebody means is
    // being asked over a series nothing else can see.
    if let Some(only_here) = a_repeat_kept_here_only(allows.goes) {
        let warning = StaticText::builder(&dialog).with_label(only_here).build();
        set_accessible_name(&warning, only_here);
        sizer.add(&warning, 0, SizerFlag::Expand | SizerFlag::All, 10);
    }

    // The question is the group's label, so a screen reader reads it when focus
    // enters the group and it does not have to be found as loose text.
    let group =
        StaticBoxSizerBuilder::new_with_label(Orientation::Vertical, &dialog, THE_QUESTION).build();

    // What each answer will do is both the button's accessible description and
    // a label on screen, for the reason the first-run screen sets out at
    // length: a description is what a screen reader working through Microsoft
    // Active Accessibility reads on focus, and the words on screen are the only
    // copy a sighted reader and a reader working through UI Automation get.
    // Dropping either takes the sentence away from somebody.
    let mut buttons = Vec::new();
    for (index, means) in EditMeans::AS_OFFERED.iter().enumerate() {
        let will_do = what_it_will_do(done, *means, allows);
        let button = RadioButton::builder(&dialog)
            .with_label(means.label())
            .with_style(if index == 0 {
                // Marks the start of the group, so the two behave as one set and
                // the arrow keys move between them rather than off to the next
                // control.
                RadioButtonStyle::GroupStart
            } else {
                RadioButtonStyle::Default
            })
            .build();
        set_accessible_name_and_description(&button, &means.spoken(), &will_do);
        button.set_value(*means == EditMeans::PRESELECTED);
        group.add(&button, 0, SizerFlag::All, 6);

        let explanation = StaticText::builder(&dialog).with_label(&will_do).build();
        set_accessible_name(&explanation, &will_do);
        group.add(&explanation, 0, SizerFlag::Expand | SizerFlag::Left, 24);

        buttons.push((*means, button));
    }
    sizer.add_sizer(&group, 1, SizerFlag::Expand | SizerFlag::All, 10);

    let carry_on = Button::builder(&dialog)
        .with_label("&Continue")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dialog)
        .with_label("&Cancel")
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&carry_on, "Continue");
    set_accessible_name(&cancel, "Cancel, and change nothing");
    let along = BoxSizer::builder(Orientation::Horizontal).build();
    along.add(&carry_on, 0, SizerFlag::All, 4);
    along.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&along, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(sizer, true);
    dialog.centre();

    carry_on.on_click({
        let closing = dialog;
        move |_| closing.end_modal(ID_OK)
    });
    cancel.on_click({
        let closing = dialog;
        move |_| closing.end_modal(ID_CANCEL)
    });
    // Enter answers the question with the one answer that touches nobody's
    // calendar. Both of the others act, and one of them acts on every day of a
    // series.
    cancel.set_default();

    // Focus on the answer that is ticked, read back from the buttons rather
    // than worked out a second time, so what is heard and what Continue takes
    // cannot come apart.
    if let Some((_, ticked)) = buttons.iter().find(|(_, button)| button.get_value()) {
        ticked.set_focus();
    }

    let answered = dialog.show_modal();
    // Read off the buttons, and only when Continue was pressed. Anything else
    // is somebody leaving the question alone, which is not an answer.
    let chosen = (answered == ID_OK).then(|| {
        buttons
            .iter()
            .find(|(_, button)| button.get_value())
            .map_or(EditMeans::PRESELECTED, |(means, _)| *means)
    });
    dialog.destroy();
    chosen
}
