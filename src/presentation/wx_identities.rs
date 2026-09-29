//! Other Addresses to Send From: the manager and its address window (#59,
//! GAP-10, 13-33).
//!
//! The manager opens from the Account Manager for one saved account, on the
//! loop every other manager uses, so its letters are the loop's own: Add,
//! Edit, Delete, Move Up, Move Down and Close. The address window asks for an
//! address and the name people see beside it. What may be kept is decided in
//! `application::identities`; this file reads, asks and says, and the caller
//! writes what the manager hands back when it closes.

use crate::application::identities::{
    self, Identity, NOT_TRIED_WITH_A_PROVIDER, Refused, TheBox, what_stops_an_address_being_kept,
};
use crate::application::reordering::{Move, Moved};
use crate::data::account::Account;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::{name_from_label, set_accessible_name};
use crate::presentation::manager_words;
use crate::presentation::status_line::said_and_shown;
use crate::presentation::text_history_keys::{keep_a_history, set_anew};
use crate::presentation::theme;
use crate::presentation::wx_managers::{
    ManagedRow, ManagerChrome, make_shell, nothing_stops_this_closing, run_manager_loop,
};
use std::sync::Arc;
use wxdragon::prelude::*;

/// The address window's OK, an id of its own rather than `ID_OK`, so a
/// refused save keeps the window open rather than wxWidgets closing it.
const ID_KEEP_THE_ADDRESS: Id = ID_HIGHEST + 330;

/// How wide a sentence in these windows runs before it wraps, in pixels.
const SENTENCE_WIDTH: i32 = 480;

impl ManagedRow for Identity {
    /// The order is the From list's, so the person decides it, with the
    /// gesture accounts and labels use.
    fn moved(rows: &[(String, String)], which: &str, direction: Move) -> Option<Moved> {
        Some(identities::moved(rows, which, direction))
    }
}

/// What the manager's title says for an account.
fn the_managers_title(account: &Account) -> String {
    format!("Other Addresses to Send From: {}", account.name)
}

/// The manager's window, built and filled without being shown.
pub struct IdentityManagerWidgets {
    pub dialog: Dialog,
    pub sizer: BoxSizer,
    pub list: ListCtrl,
    pub status: StaticText,
}

/// Build the manager over an account's other addresses, in their order, and
/// fill its list. Split from [`show_identity_manager`] so a test can read the
/// rows a live list holds.
///
/// The sentence saying no provider has been asked yet is a line of its own
/// above the list, rather than on the status line every answer is written
/// over.
pub fn build_identity_manager(
    parent: &dyn WxWidget,
    account: &Account,
    rows: &[Identity],
    palette: Option<theme::Palette>,
) -> IdentityManagerWidgets {
    let (dialog, sizer, list, status) = make_shell(
        parent,
        &the_managers_title(account),
        "Other addresses",
        560,
        400,
        palette,
    );
    let untried = StaticText::builder(&dialog)
        .with_label(NOT_TRIED_WITH_A_PROVIDER)
        .build();
    untried.wrap(SENTENCE_WIDTH);
    sizer.add(&untried, 0, SizerFlag::All, 8);

    list.insert_column(0, "Address", ListColumnFormat::Left, 240);
    list.insert_column(1, "Name people see", ListColumnFormat::Left, 200);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);
    populate_identities(&list, rows);

    IdentityManagerWidgets {
        dialog,
        sizer,
        list,
        status,
    }
}

/// What the name column says for an address kept with no name.
///
/// Said rather than left empty: an empty cell has no name on UI Automation,
/// which the scan found on pull request 135, and it reads as a row that did
/// not load.
const NO_NAME: &str = "No name";

/// Fill the manager's list, one row per address in its order.
pub fn populate_identities(list: &ListCtrl, rows: &[Identity]) {
    list.delete_all_items();
    for (at, row) in rows.iter().enumerate() {
        let at = at as i64;
        list.insert_item(at, &row.address, None);
        let name = match row.sender_name.as_str() {
            "" => NO_NAME,
            name => name,
        };
        list.set_item_text_by_column(at, 1, name);
    }
}

/// Open the manager for a saved account and hand back its addresses as the
/// person left them, or `None` when nothing changed.
pub fn show_identity_manager(
    parent: &dyn WxWidget,
    account: &Account,
    rows: &[Identity],
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> Option<Vec<Identity>> {
    let IdentityManagerWidgets {
        dialog,
        sizer,
        list,
        status,
    } = build_identity_manager(parent, account, rows, palette);

    let mut working = rows.to_vec();
    let changed = run_manager_loop(
        ManagerChrome {
            dialog: &dialog,
            main_sizer: &sizer,
            list: &list,
            status_text: &status,
            a11y: a11y.clone(),
        },
        manager_words::ADDRESS,
        &mut working,
        populate_identities,
        |over, existing, rows| ask_for_an_address(over, account, existing, rows, a11y, palette),
        |row| row.address.clone(),
        nothing_stops_this_closing,
    );
    changed.then_some(working)
}

/// The address window, built without being shown.
#[derive(Clone, Copy)]
pub struct AddressWindowWidgets {
    pub dialog: Dialog,
    pub address: TextCtrl,
    pub name: TextCtrl,
    pub status: StaticText,
    pub ok: Button,
    pub cancel: Button,
}

/// Build the address window, for a new address or one being edited.
///
/// Two boxes, each with a letter of its own, A and N, and OK and Cancel with
/// none. Focus opens on the address.
pub fn build_address_window(
    parent: &dyn WxWidget,
    existing: Option<&Identity>,
    palette: Option<theme::Palette>,
) -> AddressWindowWidgets {
    let titled = match existing {
        Some(_) => "Edit Other Address",
        None => "Add Other Address",
    };
    // Not resizable: it fits itself to what it holds, and a resize border
    // puts a nameless size grip into the tree beside the two boxes.
    let dialog = Dialog::builder(parent, titled)
        .with_style(DialogStyle::DefaultDialogStyle)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(6)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);
    let a_box = |label: &str| -> TextCtrl {
        let shown = StaticText::builder(&dialog).with_label(label).build();
        let field = TextCtrl::builder(&dialog)
            .with_size(Size::new(300, -1))
            .build();
        keep_a_history(&field);
        set_accessible_name(&field, &name_from_label(label));
        fields.add(
            &shown,
            0,
            SizerFlag::AlignCenterVertical | SizerFlag::All,
            4,
        );
        fields.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
        if let Some(palette) = palette {
            theme::paint(&field, palette.main_surface());
        }
        field
    };
    let address = a_box("&Address:");
    let name = a_box("The &name people see:");
    sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Empty until a save is refused.
    let status = StaticText::builder(&dialog).with_label("").build();
    sizer.add(
        &status,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right,
        8,
    );

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dialog)
        .with_label("OK")
        .with_id(ID_KEEP_THE_ADDRESS)
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

    if let Some(existing) = existing {
        set_anew(&address, &existing.address);
        set_anew(&name, &existing.sender_name);
    }
    cancel.on_click(move |_| dialog.end_modal(ID_CANCEL));
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }
    address.set_focus();

    AddressWindowWidgets {
        dialog,
        address,
        name,
        status,
        ok,
        cancel,
    }
}

/// Wire the window's OK to the rules for an address: closed when it can be
/// kept, or left open with the sentence said and shown and focus on the box
/// it is about. `held` is the account's other addresses apart from the one
/// being edited.
pub fn wire_the_address_window(
    w: &AddressWindowWidgets,
    account: &Account,
    held: &[Identity],
    a11y: &Arc<Accessibility>,
) {
    let w = *w;
    let account = account.clone();
    let held = held.to_vec();
    let a11y = Arc::clone(a11y);
    w.ok.on_click(move |_| {
        match what_stops_an_address_being_kept(
            &w.address.get_value(),
            &w.name.get_value(),
            &account,
            &held,
        ) {
            None => w.dialog.end_modal(ID_OK),
            Some(Refused { said, at }) => {
                said_and_shown(&w.status, &a11y, &said, Priority::High);
                w.status.wrap(SENTENCE_WIDTH);
                w.dialog.fit();
                match at {
                    TheBox::Address => w.address.set_focus(),
                    TheBox::Name => w.name.set_focus(),
                }
            }
        }
    });
}

/// Ask for an other address, new or edited, and hand it back once it can be
/// kept.
fn ask_for_an_address(
    over: &Dialog,
    account: &Account,
    existing: Option<&Identity>,
    rows: &[Identity],
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> Option<Identity> {
    let w = build_address_window(over, existing, palette);
    wire_the_address_window(&w, account, &identities::the_others(rows, existing), a11y);
    let kept = (w.dialog.show_modal() == ID_OK).then(|| {
        let id = existing
            .map(|row| row.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Identity::typed(&id, &w.address.get_value(), &w.name.get_value())
    });
    w.dialog.destroy();
    kept
}
