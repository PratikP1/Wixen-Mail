//! Asking for the passphrase of a locked PGP key (#49, 13-17.1).
//!
//! Not written yet: the dialog below is a stand-in the red half of 13-17.1
//! measures against.

use wxdragon::prelude::*;

/// The dialog's title.
pub const TITLE: &str = "Unlock a PGP Key";

/// The line said first when the dialog asks again after a wrong passphrase.
pub const THAT_DID_NOT_OPEN_IT: &str = "That passphrase did not open the key. Try again.";

/// What the dialog says about the key it is asking for.
pub fn what_it_asks(whose: &str) -> String {
    format!("The key for {whose} is locked.")
}

/// The dialog, built and not yet shown.
pub struct PassphraseDialog {
    /// The dialog itself.
    pub dialog: Dialog,
    /// Where the passphrase is typed or pasted.
    pub field: TextCtrl,
}

impl PassphraseDialog {
    /// What the dialog answers when it closes with `pressed`.
    pub fn answer(&self, _pressed: i32) -> Option<String> {
        None
    }
}

/// Build the dialog, without showing it.
pub fn build<W: WxWidget>(parent: &W, _whose: &str, _said: Option<&str>) -> PassphraseDialog {
    let dialog = Dialog::builder(parent, TITLE).build();
    let field = TextCtrl::builder(&dialog)
        .with_style(TextCtrlStyle::ReadOnly)
        .build();
    PassphraseDialog { dialog, field }
}

/// Ask for the passphrase of the key for `whose`, saying `said` first when
/// asking again. The typed text on OK, `None` on Cancel.
pub fn ask<W: WxWidget>(parent: &W, whose: &str, said: Option<&str>) -> Option<String> {
    let asking = build(parent, whose, said);
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
}
