//! The window that asks for an address book's address.
//!
//! Widgets only. What somebody typed becomes an address, whether that address
//! can be used, what a server's list becomes and what to say when it fails are
//! all in [`crate::application::address_book_source`], where they can be
//! tested. A window needs a display and a running application, so anything
//! decided in here is decided where nothing can check it.
//!
//! # Two windows rather than one that changes under somebody
//!
//! This one asks. The caller then asks the server what it has, away from the
//! thread that draws the window, with a small window saying so and offering a
//! way to stop, and opens the ordinary list-picking window for the answer with
//! the count in its label. The alternative, one window with a Find button that
//! fills a list in place, moves focus into a list that appeared, announces a
//! count that arrived, and enables a button that was disabled, all of which is
//! new behaviour on exactly the axis a screen reader user notices.
//!
//! `wx_add_calendar` is the model and this is deliberately the same shape, so
//! that somebody who has added a calendar here meets no new arrangement.
//!
//! # The mnemonics in this window
//!
//! d for the address, u for the user name, p for the password, a for Add and c
//! for Cancel. Checked by hand, and then found to be checked:
//! `test_no_two_controls_in_one_dialog_claim_the_same_alt_key` in
//! `tests/wired.rs` reads every builder in this folder and would refuse two of
//! these claiming one letter. `wx_add_calendar`'s header says nothing can check
//! them, and that has not been true since that check was written.
//!
//! What it cannot see is everything else about them. Whether Alt and the letter
//! really move focus to the box beside the label rather than to something else,
//! whether Windows or a screen reader takes the letter first, and whether the
//! letter is one somebody can find at all are three questions no reading of
//! this source answers. Those are the screen reader pass's.
//!
//! There is no kind to choose, unlike the calendar's window: an address book is
//! always signed in to and always written back to, so the two radio buttons and
//! the name box that window carries have nothing to say here. That also means
//! nothing is ever greyed out and re-enabled under somebody moving by keyboard.
//!
//! **Not confirmed with a screen reader.** Every control is named the only way
//! that reaches NVDA, and whether it is actually usable is a thing only a
//! screen reader run answers.

use crate::application::address_book_source::NOT_TRIED_FOR_REAL;
use crate::presentation::accessibility::names::{name_from_label, set_accessible_name};
use crate::presentation::text_history_keys::keep_a_history;
use crate::presentation::theme;
use wxdragon::prelude::*;

/// What somebody asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    /// What was typed, exactly as typed. Checked by the application layer.
    pub address: String,
    pub user_name: String,
    pub password: String,
}

/// Ask for an address book's address and the sign-in.
///
/// `None` when somebody changed their mind, which must leave everything as it
/// was.
pub fn ask_for_an_address_book(parent: &Frame) -> Option<Asked> {
    let widgets = build_add_address_book_dialog(parent, theme::current_from_stored_config());

    let answer = widgets.dialog.show_modal();
    let asked = (answer == ID_OK).then(|| Asked {
        address: widgets.address.get_value(),
        user_name: widgets.user_name.get_value(),
        password: widgets.password.get_value(),
    });
    widgets.dialog.destroy();
    asked
}

/// The Add Address Book dialog's widgets, returned so a test can build it
/// without a human closing a live modal and so `ask_for_an_address_book` can
/// read every field back after a real `.show_modal()`.
pub struct AddAddressBookWidgets {
    pub dialog: Dialog,
    pub address: TextCtrl,
    pub user_name: TextCtrl,
    pub password: TextCtrl,
}

/// Build the Add Address Book dialog without showing it.
pub fn build_add_address_book_dialog(
    parent: &Frame,
    palette: Option<theme::Palette>,
) -> AddAddressBookWidgets {
    let dialog = Dialog::builder(parent, "Add an address book by its address")
        .with_size(620, 420)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // First in the window, so it is met before the password box rather than
    // after it. Said here and not only in a report: somebody adding an address
    // book here is the first person this has ever run for.
    let warning = StaticText::builder(&dialog)
        .with_label(NOT_TRIED_FOR_REAL)
        .build();
    set_accessible_name(&warning, NOT_TRIED_FOR_REAL);
    sizer.add(&warning, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let fields = FlexGridSizer::builder(3, 2)
        .with_hgap(6)
        .with_vgap(6)
        .build();
    fields.add_growable_col(1, 1);
    let address = labelled_field(&dialog, &fields, "A&ddress", TextCtrlStyle::Default);
    let user_name = labelled_field(&dialog, &fields, "&User name", TextCtrlStyle::Default);
    let password = labelled_field(&dialog, &fields, "&Password", TextCtrlStyle::Password);
    sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let help = StaticText::builder(&dialog)
        .with_label(
            "An address book on a server is named by the server, and you choose which of \
             its address books to add after signing in.",
        )
        .build();
    sizer.add(&help, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let add = Button::builder(&dialog)
        .with_label("&Add")
        .with_id(ID_OK)
        .build();
    set_accessible_name(&add, "Add");
    let cancel = Button::builder(&dialog)
        .with_label("&Cancel")
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&cancel, "Cancel");
    buttons.add(&add, 0, SizerFlag::All, 6);
    buttons.add(&cancel, 0, SizerFlag::All, 6);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dialog.set_sizer(sizer, true);

    add.on_click(move |_| dialog.end_modal(ID_OK));
    cancel.on_click(move |_| dialog.end_modal(ID_CANCEL));

    // On the address, which is the thing somebody came here to type.
    address.set_focus();

    // Painted last. `None` means high contrast is on, or the system is set up
    // in a way this application should not paint over, so nothing is set here
    // and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
        theme::paint(&address, palette.main_surface());
        theme::paint(&user_name, palette.main_surface());
        theme::paint(&password, palette.main_surface());
    }

    AddAddressBookWidgets {
        dialog,
        address,
        user_name,
        password,
    }
}

/// A label and the box beside it, with the box named the way NVDA reads.
///
/// The visible label is a control of its own and wxWidgets never joins the two,
/// so without the name the box announces as "edit" and nothing else.
///
/// The box keeps a history of several steps, unless it is a password box: a
/// password is never held in memory as steps.
fn labelled_field(
    dialog: &Dialog,
    sizer: &FlexGridSizer,
    label: &str,
    style: TextCtrlStyle,
) -> TextCtrl {
    let text = StaticText::builder(dialog).with_label(label).build();
    let field = TextCtrl::builder(dialog).with_style(style).build();
    if !style.contains(TextCtrlStyle::Password) {
        keep_a_history(&field);
    }
    set_accessible_name(&field, &name_from_label(label));
    sizer.add(&text, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    sizer.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    field
}
