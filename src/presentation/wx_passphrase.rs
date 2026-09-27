//! Asking for the passphrase of a locked PGP key (#49, 13-17.1).
//!
//! A reader window opens this when a message it is about to show was
//! encrypted to a key here that a passphrase is holding shut, and nobody has
//! typed that passphrase since Wixen Mail started. The preview never does: a
//! dialog that appears while somebody arrows through a list is a trap.
//!
//! # Where the words go
//!
//! What is typed is handed back once, to
//! [`crate::application::pgp_keys::unlock`], which turns it into the
//! OpenPGP crate's own passphrase type at once. This window keeps nothing,
//! and it is destroyed as soon as it answers. Nothing here logs it.
//!
//! # Paste
//!
//! The field is Windows' own password box, which takes a paste and refuses a
//! copy. A password manager pastes, and WCAG 3.3.8 asks that nobody has to
//! type a secret from memory, so nothing here stands between the box and a
//! paste. No puzzle and no code to copy either: the one thing asked for is the
//! passphrase itself.
//!
//! # Letters
//!
//! One: `P` for the field. OK and Cancel are Enter and Escape.

use crate::presentation::accessibility::names::{name_from_label, set_accessible_name};
use crate::presentation::theme;
use wxdragon::prelude::*;

/// The dialog's title.
pub const TITLE: &str = "Unlock a PGP Key";

/// The label beside the field, and through it the field's name.
///
/// No colon, as the password fields of Add Calendar and Add Address Book have
/// none: `name_from_label` turns a trailing colon into a comma, and the name
/// here is the one word.
pub const THE_PASSPHRASE_LABEL: &str = "&Passphrase";

/// The line said first when the dialog asks again after a wrong passphrase.
pub const THAT_DID_NOT_OPEN_IT: &str = "That passphrase did not open the key. Try again.";

/// What the dialog says about the key it is asking for: whose, what to do,
/// and what becomes of the passphrase.
///
/// Whose is the key's own first name and address, from the credential store,
/// and never anything a message says, so a stranger's mail cannot word a
/// prompt for a secret.
pub fn what_it_asks(whose: &str) -> String {
    format!(
        "The key for {whose} is locked with a passphrase. Type it to open this message. It is \
         remembered until Wixen Mail closes and never saved."
    )
}

/// What the dialog says when the composer asks at Send, for a key that is to
/// sign the message rather than open one.
pub fn what_signing_asks(whose: &str) -> String {
    what_it_asks(whose)
}

/// The dialog, built and not yet shown.
pub struct PassphraseDialog {
    /// The dialog itself.
    pub dialog: Dialog,
    /// Where the passphrase is typed or pasted.
    pub field: TextCtrl,
}

impl PassphraseDialog {
    /// What the dialog answers when it closes with `pressed`: what was typed
    /// on OK, and nothing on anything else.
    pub fn answer(&self, pressed: i32) -> Option<String> {
        (pressed == ID_OK).then(|| self.field.get_value())
    }
}

/// Build the dialog, without showing it.
///
/// `said` is the line said first when asking again, so it is heard before the
/// sentence it qualifies rather than after it.
pub fn build<W: WxWidget>(parent: &W, whose: &str, said: Option<&str>) -> PassphraseDialog {
    let dialog = Dialog::builder(parent, TITLE)
        .with_style(DialogStyle::DefaultDialogStyle)
        .build();
    if let Some(palette) = theme::current_from_stored_config() {
        theme::paint(&dialog, palette.main_surface());
    }
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    if let Some(said) = said {
        let again = StaticText::builder(&dialog).with_label(said).build();
        again.wrap(440);
        sizer.add(&again, 0, SizerFlag::All, 8);
    }
    let asks = StaticText::builder(&dialog)
        .with_label(&what_it_asks(whose))
        .build();
    asks.wrap(440);
    sizer.add(&asks, 0, SizerFlag::All, 8);

    let row = BoxSizer::builder(Orientation::Horizontal).build();
    let label = StaticText::builder(&dialog)
        .with_label(THE_PASSPHRASE_LABEL)
        .build();
    row.add(
        &label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    // A password box keeps no history of ours: a passphrase is never held in
    // memory as steps.
    let field = TextCtrl::builder(&dialog)
        .with_style(TextCtrlStyle::Password)
        .with_size(Size::new(280, -1))
        .build();
    set_accessible_name(&field, &name_from_label(THE_PASSPHRASE_LABEL));
    row.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add_sizer(&row, 0, SizerFlag::Expand | SizerFlag::All, 4);

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
    PassphraseDialog { dialog, field }
}

/// Ask for the passphrase of the key for `whose`, saying `said` first when
/// asking again. What was typed on OK, `None` on Cancel.
pub fn ask<W: WxWidget>(parent: &W, whose: &str, said: Option<&str>) -> Option<String> {
    let asking = build(parent, whose, said);
    asking.field.set_focus();
    let pressed = asking.dialog.show_modal();
    let answer = asking.answer(pressed);
    asking.dialog.destroy();
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_says_whose_key_it_is_and_what_becomes_of_the_passphrase() {
        // Whose, from the key rather than the message; what to do; and where
        // the passphrase goes, which is the question somebody typing a secret
        // into a mail program has a right to have answered first.
        assert_eq!(
            what_it_asks("Ada Lovelace <ada@example.com>"),
            "The key for Ada Lovelace <ada@example.com> is locked with a passphrase. Type it \
             to open this message. It is remembered until Wixen Mail closes and never saved."
        );
    }

    #[test]
    fn test_asked_at_send_it_says_the_passphrase_is_to_sign_the_message() {
        // The composer asks for the same passphrase at Send, and "to open this
        // message" there would name a message that is not the one being sent.
        assert_eq!(
            what_signing_asks("Ada Lovelace <ada@example.com>"),
            "The key for Ada Lovelace <ada@example.com> is locked with a passphrase. Type it \
             to sign this message. It is remembered until Wixen Mail closes and never saved."
        );
    }
}
