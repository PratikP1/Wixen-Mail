//! The window that asks for a calendar's address.
//!
//! Widgets only. What somebody typed becomes an address, whether that address
//! can be used, what a server's list becomes and what to say when it fails are
//! all in [`crate::application::calendar_source`], where they can be tested. A
//! window needs a display and a running application, so anything decided in
//! here is decided where nothing can check it.
//!
//! # Two windows rather than one that changes under somebody
//!
//! This one asks. If it is a calendar server, the caller then asks the server
//! what it has, away from the thread that draws the window, with a small window
//! saying so and offering a way to stop, and opens the ordinary list-picking
//! window for the answer with the count in its label. The
//! alternative, one window with a Find button that fills a list in place, moves
//! focus into a list that appeared, announces a count that arrived, and enables
//! a button that was disabled, all of which is new behaviour on exactly the
//! axis a screen reader user notices.
//!
//! # The mnemonics in this window
//!
//! Checked by hand, because nothing can check them: s for a calendar on a
//! server, f for a feed, d for the address, n for the name, u for the user
//! name, p for the password, a for Add and c for Cancel. No two are the same,
//! so every control here can be reached with Alt and one letter.
//!
//! **Not confirmed with a screen reader.** Every control is named the only way
//! that reaches NVDA, and whether it is actually usable is a thing only a
//! screen reader run answers.

use crate::application::calendar_source::{NOT_TRIED_FOR_REAL, Source};
use crate::presentation::accessibility::names::{name_from_label, set_accessible_name};
use wxdragon::prelude::*;

/// What somebody asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    /// A calendar server or a published feed.
    pub source: Source,
    /// What was typed, exactly as typed. Checked by the application layer.
    pub address: String,
    /// What to call a feed. A server names its own calendars.
    pub name: String,
    pub user_name: String,
    pub password: String,
}

/// Ask for a calendar's address, and the sign-in if it needs one.
///
/// `None` when somebody changed their mind, which must leave everything as it
/// was.
pub fn ask_for_a_calendar(parent: &Frame) -> Option<Asked> {
    let dialog = Dialog::builder(parent, "Add a calendar by its address")
        .with_size(620, 480)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Said in the window, not only in a report. Somebody adding a calendar here
    // is the first person this has ever run for.
    let warning = StaticText::builder(&dialog)
        .with_label(NOT_TRIED_FOR_REAL)
        .build();
    set_accessible_name(&warning, NOT_TRIED_FOR_REAL);
    sizer.add(&warning, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let kind = StaticBoxSizerBuilder::new_with_label(
        Orientation::Vertical,
        &dialog,
        "What kind of calendar is it?",
    )
    .build();
    let on_a_server = RadioButton::builder(&dialog)
        .with_label("A calendar on a &server, which you sign in to")
        .with_style(RadioButtonStyle::GroupStart)
        .build();
    set_accessible_name(
        &on_a_server,
        &name_from_label("A calendar on a &server, which you sign in to"),
    );
    on_a_server.set_value(true);
    kind.add(&on_a_server, 0, SizerFlag::All, 6);

    let from_a_feed = RadioButton::builder(&dialog)
        .with_label("A calendar &feed, which is only ever read")
        .build();
    set_accessible_name(
        &from_a_feed,
        &name_from_label("A calendar &feed, which is only ever read"),
    );
    kind.add(&from_a_feed, 0, SizerFlag::All, 6);
    sizer.add_sizer(&kind, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let fields = FlexGridSizer::builder(4, 2)
        .with_hgap(6)
        .with_vgap(6)
        .build();
    fields.add_growable_col(1, 1);
    let address = labelled_field(&dialog, &fields, "A&ddress", TextCtrlStyle::Default);
    let name = labelled_field(
        &dialog,
        &fields,
        "&Name for this calendar",
        TextCtrlStyle::Default,
    );
    let user_name = labelled_field(&dialog, &fields, "&User name", TextCtrlStyle::Default);
    let password = labelled_field(&dialog, &fields, "&Password", TextCtrlStyle::Password);
    sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let help = StaticText::builder(&dialog)
        .with_label(
            "A feed needs no sign-in, and it takes its name from what you type here. A \
             calendar on a server is named by the server, and you choose which of its \
             calendars to add after signing in.",
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

    // Greyed rather than hidden, both ways round, so the order of the controls
    // never changes under somebody who is moving through them by keyboard.
    let show_for = move |server: bool| {
        user_name.enable(server);
        password.enable(server);
        name.enable(!server);
    };
    show_for(true);
    on_a_server.on_selected(move |_| show_for(true));
    from_a_feed.on_selected(move |_| show_for(false));

    add.on_click(move |_| dialog.end_modal(ID_OK));
    cancel.on_click(move |_| dialog.end_modal(ID_CANCEL));

    // On the address, which is the thing somebody came here to type. The
    // kind is already answered with the commoner of the two.
    address.set_focus();

    let answer = dialog.show_modal();
    let asked = (answer == ID_OK).then(|| Asked {
        source: if from_a_feed.get_value() {
            Source::Feed
        } else {
            Source::Server
        },
        address: address.get_value(),
        name: name.get_value(),
        user_name: user_name.get_value(),
        password: password.get_value(),
    });
    dialog.destroy();
    asked
}

/// A label and the box beside it, with the box named the way NVDA reads.
///
/// The visible label is a control of its own and wxWidgets never joins the two,
/// so without the name the box announces as "edit" and nothing else.
fn labelled_field(
    dialog: &Dialog,
    sizer: &FlexGridSizer,
    label: &str,
    style: TextCtrlStyle,
) -> TextCtrl {
    let text = StaticText::builder(dialog).with_label(label).build();
    let field = TextCtrl::builder(dialog).with_style(style).build();
    set_accessible_name(&field, &name_from_label(label));
    sizer.add(&text, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    sizer.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    field
}
