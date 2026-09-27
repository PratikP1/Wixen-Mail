//! The PGP key manager: every key on this computer, and what can be done with
//! one (GAP-03, #49).

use crate::application::pgp_keys::Imported;
use crate::common::Result;
use crate::data::message_cache::MessageCache;
use crate::presentation::accessibility::Accessibility;
use crate::service::pgp::KeyListing;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

/// What the window is called.
pub const TITLE: &str = "PGP Keys";

/// The label over the sentence saying what keys can do in this build.
pub const THE_LIMITS_LABEL: &str = "What these keys can do &here:";
/// The label over the list.
pub const THE_LIST_LABEL: &str = "&Keys:";
/// The button that reads keys in from a file.
pub const IMPORT_FROM_A_FILE: &str = "Import from &File...";
/// The button that reads a key in from pasted text.
pub const PASTE_A_KEY: &str = "&Paste a Key...";
/// The button that writes the chosen key's public half to a file.
pub const EXPORT_THE_PUBLIC_KEY: &str = "E&xport Public Key...";
/// The button that puts the chosen key's public half on the clipboard.
pub const COPY_THE_PUBLIC_KEY: &str = "&Copy Public Key";
/// The button that removes the chosen key, after asking.
pub const REMOVE_THE_KEY: &str = "&Remove...";
/// The button that leaves.
pub const CLOSE: &str = "Cl&ose";

/// Every label in the window, in the order Tab reaches what each names.
pub const LABELS: [&str; 8] = [
    THE_LIMITS_LABEL,
    THE_LIST_LABEL,
    IMPORT_FROM_A_FILE,
    PASTE_A_KEY,
    EXPORT_THE_PUBLIC_KEY,
    COPY_THE_PUBLIC_KEY,
    REMOVE_THE_KEY,
    CLOSE,
];

/// The label over the box a key is pasted into.
pub const THE_KEY_TEXT_LABEL: &str = "&Key text:";

/// What the line every answer is written to is called.
pub const WHAT_HAPPENED: &str = "What happened";

/// The letter a label binds to Alt, lower case, or `None` when it binds none.
pub fn the_letter(label: &str) -> Option<char> {
    let _ = label;
    None
}

/// Every letter more than one of these labels claims.
pub fn letters_claimed_twice(labels: &[&str]) -> Vec<char> {
    let _ = labels;
    Vec::new()
}

/// Where focus goes when the window opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereFocusGoes {
    /// The first key in the list.
    TheFirstKey,
    /// Import from File, because there is no key to land on.
    ImportFromAFile,
}

/// Where focus should go when the window opens, given how many keys it lists.
pub fn where_focus_goes(rows: usize) -> WhereFocusGoes {
    let _ = rows;
    WhereFocusGoes::ImportFromAFile
}

/// The key a press acts on, out of what the list is showing.
pub fn the_row_chosen(showing: &[KeyListing], selected: i64) -> Option<&KeyListing> {
    let _ = (showing, selected);
    None
}

/// Reading every key here.
pub type ReadingTheKeys = Box<dyn Fn() -> Result<Vec<KeyListing>>>;
/// Importing every key a text holds.
pub type ImportingKeys = Box<dyn Fn(&str) -> Vec<Imported>>;
/// Removing the key with a fingerprint.
pub type RemovingAKey = Box<dyn Fn(&str) -> Result<bool>>;
/// The public half of the key with a fingerprint.
pub type ExportingAPublicKey = Box<dyn Fn(&str) -> Result<Option<String>>>;

/// Everything the window does to keys, and nothing else.
pub struct TheKeysUnderneath {
    /// Every key here.
    pub every_key: ReadingTheKeys,
    /// Import every key a text holds.
    pub import: ImportingKeys,
    /// Remove the key with this fingerprint.
    pub remove: RemovingAKey,
    /// The public half of the key with this fingerprint.
    pub export_public: ExportingAPublicKey,
}

impl TheKeysUnderneath {
    /// The keys on this computer, through `application::pgp_keys`.
    pub fn kept_in(cache: Arc<MessageCache>) -> Self {
        let _ = cache;
        TheKeysUnderneath {
            every_key: Box::new(|| Ok(Vec::new())),
            import: Box::new(|_| Vec::new()),
            remove: Box::new(|_| Ok(false)),
            export_public: Box::new(|_| Ok(None)),
        }
    }
}

/// Asking a yes or no question over the window.
pub type AskingYesOrNo = Box<dyn Fn(&Dialog, &str) -> bool>;
/// Putting text on the clipboard, and whether it took it.
pub type PuttingOnTheClipboard = Box<dyn Fn(&str) -> bool>;

/// What the window needs from the desktop around it.
pub struct TheDesktop {
    /// Ask a question with Yes and No, No the default.
    pub ask: AskingYesOrNo,
    /// Put text on the clipboard.
    pub clipboard: PuttingOnTheClipboard,
}

impl TheDesktop {
    /// Windows' own question box and clipboard.
    pub fn this_one() -> Self {
        TheDesktop {
            ask: Box::new(|_, _| false),
            clipboard: Box::new(|_| false),
        }
    }
}

/// The window, built and not yet shown.
#[derive(Clone)]
pub struct KeyManager {
    /// The dialog itself.
    pub dialog: Dialog,
    /// The line every answer is written to.
    pub status: StaticText,
    showing: Rc<RefCell<Vec<KeyListing>>>,
}

impl KeyManager {
    /// Put the list's cursor on a row, as a person arrowing to it does.
    pub fn choose(&self, row: usize) {
        let _ = row;
    }

    /// What Remove does: ask, and remove the chosen key on a yes.
    pub fn remove_the_chosen_key(&self) {}

    /// What Copy Public Key does.
    pub fn copy_the_chosen_key(&self) {}

    /// Import every key a text holds, and say what became of each.
    pub fn import_text(&self, text: &str) {
        let _ = text;
    }

    /// How many keys the list shows.
    pub fn rows(&self) -> usize {
        self.showing.borrow().len()
    }
}

/// Build the key manager over these keys, without showing it.
pub fn build<W: WxWidget>(
    parent: &W,
    underneath: TheKeysUnderneath,
    desktop: TheDesktop,
    a11y: &Arc<Accessibility>,
) -> KeyManager {
    let _ = (underneath, desktop, a11y);
    let dialog = Dialog::builder(parent, TITLE).build();
    let status = StaticText::builder(&dialog).with_label("").build();
    KeyManager {
        dialog,
        status,
        showing: Rc::new(RefCell::new(Vec::new())),
    }
}

/// The box a key is pasted into, built and not yet shown.
pub struct PasteDialog {
    /// The dialog itself.
    pub dialog: Dialog,
    /// Where the key's text goes.
    pub text: TextCtrl,
}

/// Build the Paste a Key dialog, without showing it.
pub fn build_the_paste_dialog<W: WxWidget>(parent: &W) -> PasteDialog {
    let dialog = Dialog::builder(parent, "Paste a Key").build();
    let text = TextCtrl::builder(&dialog).build();
    PasteDialog { dialog, text }
}

/// Show the key manager over the keys on this computer.
pub fn show(parent: &Frame, cache: Arc<MessageCache>, a11y: &Arc<Accessibility>) {
    let manager = build(
        parent,
        TheKeysUnderneath::kept_in(cache),
        TheDesktop::this_one(),
        a11y,
    );
    manager.dialog.show_modal();
    manager.dialog.destroy();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::service::pgp::for_tests::{
        ALICES_FINGERPRINT, CAROLS_FINGERPRINT, alices_private_key, carols_public_key,
    };
    use crate::service::secret_store;

    #[test]
    fn test_every_label_claims_a_letter_and_no_two_claim_the_same() {
        let letters: Vec<Option<char>> = LABELS.iter().map(|label| the_letter(label)).collect();

        assert_eq!(
            letters,
            ['h', 'k', 'f', 'p', 'x', 'c', 'r', 'o']
                .into_iter()
                .map(Some)
                .collect::<Vec<_>>()
        );
        assert_eq!(letters_claimed_twice(&LABELS), Vec::<char>::new());
    }

    #[test]
    fn test_a_letter_two_labels_claim_is_refused() {
        // The companion: a check that never finds a duplicate would pass the
        // case above whatever the labels said.
        let planted = [THE_LIST_LABEL, "&Keep", IMPORT_FROM_A_FILE, "Save &As"];

        assert_eq!(letters_claimed_twice(&planted), vec!['k']);
    }

    #[test]
    fn test_a_doubled_marker_is_a_literal_ampersand_and_claims_nothing() {
        assert_eq!(the_letter("Fish && Chips"), None);
        assert_eq!(the_letter("Fish && &Chips"), Some('c'));
    }

    #[test]
    fn test_focus_opens_on_the_first_key_and_on_import_when_there_is_none() {
        assert_eq!(where_focus_goes(2), WhereFocusGoes::TheFirstKey);
        assert_eq!(where_focus_goes(0), WhereFocusGoes::ImportFromAFile);
    }

    fn two_keys(cache: &MessageCache) -> Vec<KeyListing> {
        crate::application::pgp_keys::import(
            cache,
            &format!("{}\n{}", alices_private_key(), carols_public_key()),
        );
        crate::application::pgp_keys::every_key_here(cache).expect("the keys")
    }

    /// A cache held the way the main window hands it to [`show`].
    ///
    /// `MessageCache` wraps a rusqlite connection and is not `Sync`, so the
    /// `Arc` buys sharing rather than thread safety, as it does in production;
    /// the same allow `managers`' `test_cache_arc` carries for the same reason.
    #[allow(clippy::arc_with_non_send_sync)]
    fn a_cache(what_for: &str) -> TempHome<Arc<MessageCache>> {
        secret_store::allow();
        TempHome::named(what_for, |dir| {
            Arc::new(MessageCache::new(dir.to_path_buf(), None).expect("a cache to open"))
        })
    }

    #[test]
    fn test_the_row_a_press_acts_on_is_the_one_chosen_and_nothing_is_not_a_row() {
        let cache = a_cache("pgp-window-rows");
        let rows = two_keys(&cache);

        assert_eq!(the_row_chosen(&rows, 0), Some(&rows[0]));
        assert_eq!(the_row_chosen(&rows, 1), Some(&rows[1]));
        assert_eq!(the_row_chosen(&rows, -1), None);
        assert_eq!(the_row_chosen(&rows, 2), None);
    }

    #[test]
    fn test_the_keys_underneath_are_the_ones_the_application_layer_keeps() {
        let cache = a_cache("pgp-window-underneath");
        let underneath = TheKeysUnderneath::kept_in(Arc::clone(&cache));

        let said: Vec<String> = (underneath.import)(&format!(
            "{}\n{}",
            alices_private_key(),
            carols_public_key()
        ))
        .into_iter()
        .map(|imported| imported.said)
        .collect();
        assert_eq!(said.len(), 2, "{said:?}");

        let listed: Vec<(String, bool)> = (underneath.every_key)()
            .expect("the keys")
            .into_iter()
            .map(|key| (key.fingerprint, key.private))
            .collect();
        assert_eq!(
            listed,
            vec![
                (ALICES_FINGERPRINT.to_string(), true),
                (CAROLS_FINGERPRINT.to_string(), false),
            ]
        );

        let exported = (underneath.export_public)(ALICES_FINGERPRINT)
            .expect("read")
            .expect("Alice's public half");
        let described = crate::service::pgp::describe(&exported);
        assert_eq!(described.len(), 1, "{described:?}");
        assert!(!described[0].private, "the private half was exported");

        assert!((underneath.remove)(CAROLS_FINGERPRINT).expect("removed"));
        assert_eq!((underneath.every_key)().expect("the keys").len(), 1);
    }
}
