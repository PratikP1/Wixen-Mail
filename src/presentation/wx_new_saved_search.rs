//! New Saved Search: the name of a saved search made from nothing, and where
//! it looks (#58, GAP-09, 13-39).
//!
//! The first of two windows. This one asks the name and the place; the
//! conditions window, which Edit Conditions also opens, asks every or any and
//! the conditions. Nothing is written until the second closes with at least
//! one condition, so this window keeps nothing of its own.

use crate::application::saved_searches::{Naming, name_for};
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::{name_from_label, set_accessible_name};
use crate::presentation::text_history_keys::keep_a_history;
use crate::presentation::theme;
use std::sync::Arc;
use wxdragon::prelude::*;

/// The window's title, and the caption of its refusal.
const TITLE: &str = "New Saved Search";

/// N and L: the window's only letters. OK and Cancel carry none, as the
/// filter editor's do.
const THE_NAME_LABEL: &str = "&Name for this search:";
const THE_PLACE_LABEL: &str = "&Look in:";

/// Said before OK, so pressing it is not a surprise: a second window follows.
const WHAT_COMES_NEXT: &str = "Next, add the conditions a message has to meet.";

/// The New Saved Search window's controls, returned so a test and a scan
/// target can build it without the modal loop.
#[derive(Clone, Copy)]
pub struct NewSavedSearchWindow {
    pub dialog: Dialog,
    pub name: TextCtrl,
    pub look_in: Choice,
    pub ok: Button,
    pub cancel: Button,
}

/// Build the New Saved Search window over the places a search can look in,
/// the first chosen, without showing it.
///
/// `places` is what `saved_searches::where_a_search_can_look` answers: the
/// words the choice shows, and the folder each stores.
pub fn build_new_saved_search_dialog(
    parent: &dyn WxWidget,
    places: &[(String, Option<String>)],
    palette: Option<theme::Palette>,
) -> NewSavedSearchWindow {
    let dialog = Dialog::builder(parent, TITLE)
        .with_style(DialogStyle::DefaultDialogStyle)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    let name_label = StaticText::builder(&dialog)
        .with_label(THE_NAME_LABEL)
        .build();
    let name = TextCtrl::builder(&dialog)
        .with_size(Size::new(280, -1))
        .build();
    // Several steps of Undo, as every box a person types into has, rather
    // than Windows' single step.
    keep_a_history(&name);
    // Each label is a separate control, which wxWidgets never associates with
    // the field beside it, so without a name the field announces as just
    // "edit" or "combo box".
    set_accessible_name(&name, &name_from_label(THE_NAME_LABEL));
    fields.add(
        &name_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&name, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let place_label = StaticText::builder(&dialog)
        .with_label(THE_PLACE_LABEL)
        .build();
    let look_in = Choice::builder(&dialog)
        .with_choices(places.iter().map(|(said, _)| said.clone()).collect())
        .build();
    set_accessible_name(&look_in, &name_from_label(THE_PLACE_LABEL));
    if !places.is_empty() {
        look_in.set_selection(0);
    }
    fields.add(
        &place_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&look_in, 1, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let next = StaticText::builder(&dialog)
        .with_label(WHAT_COMES_NEXT)
        .build();
    sizer.add(
        &next,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right,
        12,
    );

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dialog)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    ok.set_default();
    let cancel = Button::builder(&dialog)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    buttons.add(&ok, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dialog.set_sizer_and_fit(sizer, true);

    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
        theme::paint(&name, palette.main_surface());
    }

    NewSavedSearchWindow {
        dialog,
        name,
        look_in,
        ok,
        cancel,
    }
}

/// Ask the name of a saved search made from nothing and where it looks.
///
/// The tidied name and the chosen place's folder, `None` for everywhere in the
/// account; `None` altogether when it was cancelled. A name `name_for` refuses
/// is refused here, aloud and in a message box, and the window stays open with
/// what was typed still in it (3.3.7), so the caller never opens it twice.
pub fn ask(
    parent: &dyn WxWidget,
    a11y: &Arc<Accessibility>,
    places: &[(String, Option<String>)],
    already_used: &[String],
) -> Option<(String, Option<String>)> {
    let window = build_new_saved_search_dialog(parent, places, theme::current_from_stored_config());
    window.ok.on_click({
        let dialog = window.dialog;
        let typed = window.name;
        let a11y = Arc::clone(a11y);
        let already_used = already_used.to_vec();
        move |event| {
            // Consuming the click is what makes the refusal stick.
            event.event.skip(false);
            if let Some(why) = name_for(&typed.get_value(), &already_used).why_not() {
                let _ = a11y.announce(&why, Priority::High);
                MessageDialog::builder(&dialog, &why, TITLE)
                    .build()
                    .show_modal();
                typed.set_focus();
                return;
            }
            dialog.end_modal(ID_OK);
        }
    });

    window.name.set_focus();
    let pressed = window.dialog.show_modal();
    // Read first, then destroy: the controls belong to the dialog, and
    // wxWidgets does not free one when the Rust value goes.
    let named = name_for(&window.name.get_value(), already_used);
    let place = window
        .look_in
        .get_selection()
        .and_then(|chosen| places.get(chosen as usize))
        .and_then(|(_, folder)| folder.clone());
    window.dialog.destroy();

    match (pressed == ID_OK, named) {
        (true, Naming::Accepted(name)) => Some((name, place)),
        _ => None,
    }
}
