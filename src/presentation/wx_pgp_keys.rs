//! The PGP key manager: every key on this computer, and what can be done with
//! one (GAP-03, #49).
//!
//! A tester on 2026-09-15: "While there is an import PGP key option, there is
//! no way to manage these keys." This is the way. It lists every key here,
//! private and public, one row per key read as a person before a fingerprint;
//! imports from a file or from pasted text; hands a key's public half to a
//! file or the clipboard; and removes a key only after a question that names
//! it and says what will stop working.
//!
//! Nothing here decides anything about keys. What a row says, the question
//! before a removal, the sentence saying what keys can do in this build and
//! every answer are [`crate::application::pgp_keys`]'s, where a test can read
//! them without a window, and every key operation goes through
//! [`TheKeysUnderneath`], which [`show`] fills from that module.
//!
//! # Reaching it and leaving it
//!
//! File, PGP Keys, on K, where File, Import PGP Private Key was until 13-17
//! (Pratik, 2026-09-24). Modal, and opened once in a while, so no chord of its
//! own. No control is ever disabled: a button pressed with no key chosen says
//! so, because a greyed button is skipped by Tab and never says why.
//!
//! # What has never been checked
//!
//! Nobody has heard this, and no real key has been through it. How a row
//! sounds, whether the question is read in full, and whether the letters land
//! are in the broken windows ledger as `unrun-verify`.

use crate::application::pgp_keys::{self, Imported};
use crate::application::status_sentences::nothing_chosen_named;
use crate::common::Result;
use crate::common::how_the_machine_writes_dates::WhichLocale;
use crate::data::message_cache::MessageCache;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::status_line::said_and_shown;
use crate::presentation::text_history_keys::keep_a_history;
use crate::presentation::theme;
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
///
/// One set of letters for the whole window, checked by
/// `test_every_label_claims_a_letter_and_no_two_claim_the_same`: two labels
/// on one letter is a key that lands on whichever comes first.
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

/// The list's headings, in the order the columns are built: the person first.
pub const THE_COLUMNS: [&str; 7] = [
    "Name and address",
    "Kind",
    "Key id",
    "Fingerprint",
    "Created",
    "Expires",
    "Can",
];

/// The letter a label binds to Alt, lower case, or `None` when it binds none.
///
/// `&&` is a literal ampersand and binds nothing, which is how Windows reads
/// it too.
pub fn the_letter(label: &str) -> Option<char> {
    let mut letters = label.chars();
    while let Some(letter) = letters.next() {
        if letter != '&' {
            continue;
        }
        match letters.next() {
            Some('&') => continue,
            Some(bound) if bound.is_alphanumeric() => return Some(bound.to_ascii_lowercase()),
            _ => return None,
        }
    }
    None
}

/// Every letter more than one of these labels claims, in the order the second
/// claim is met.
pub fn letters_claimed_twice(labels: &[&str]) -> Vec<char> {
    let mut claimed = Vec::new();
    let mut twice = Vec::new();
    for letter in labels.iter().filter_map(|label| the_letter(label)) {
        if claimed.contains(&letter) && !twice.contains(&letter) {
            twice.push(letter);
        }
        claimed.push(letter);
    }
    twice
}

/// The words a label says: without the marker that makes a letter its key,
/// which a reader taking a name literally says as "ampersand", and without
/// the colon that ends a label over a field.
fn what_the_label_says(label: &str) -> String {
    label.replace('&', "").trim_end_matches(':').to_string()
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
///
/// Focus in a list with no rows answers every arrow key with silence, which
/// somebody working by ear cannot tell from a window that failed to load. The
/// sentence above the list and the first thing to do are what an empty
/// window offers.
pub fn where_focus_goes(rows: usize) -> WhereFocusGoes {
    if rows == 0 {
        WhereFocusGoes::ImportFromAFile
    } else {
        WhereFocusGoes::TheFirstKey
    }
}

/// The key a press acts on, out of what the list is showing.
///
/// `wxListCtrl` answers -1 when nothing is chosen, which is why this takes the
/// control's own signed answer.
pub fn the_row_chosen(showing: &[KeyListing], selected: i64) -> Option<&KeyListing> {
    usize::try_from(selected)
        .ok()
        .and_then(|at| showing.get(at))
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
///
/// Closures rather than a database handle, on the Blocked Senders window's
/// pattern, so the window can be built over keys held anywhere. The test that
/// reads it over MSAA builds it over keys in memory, because the real ones
/// live in the credential store of whoever runs it.
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
        let listing = Arc::clone(&cache);
        let importing = Arc::clone(&cache);
        let removing = Arc::clone(&cache);
        let exporting = cache;
        TheKeysUnderneath {
            every_key: Box::new(move || pgp_keys::every_key_here(&listing)),
            import: Box::new(move |text| pgp_keys::import(&importing, text)),
            remove: Box::new(move |fingerprint| pgp_keys::remove(&removing, fingerprint)),
            export_public: Box::new(move |fingerprint| {
                pgp_keys::export_public(&exporting, fingerprint)
            }),
        }
    }
}

/// Asking a yes or no question over the window.
pub type AskingYesOrNo = Box<dyn Fn(&Dialog, &str) -> bool>;
/// Putting text on the clipboard, and whether it took it.
pub type PuttingOnTheClipboard = Box<dyn Fn(&str) -> bool>;

/// What the window needs from the desktop around it.
///
/// Handed in so the MSAA test can answer the question and read the clipboard
/// without a person to press No and without overwriting whatever the person
/// running it had copied.
pub struct TheDesktop {
    /// Ask a question with Yes and No, No the answer Enter gives.
    pub ask: AskingYesOrNo,
    /// Put text on the clipboard.
    pub clipboard: PuttingOnTheClipboard,
}

impl TheDesktop {
    /// Windows' own question box and clipboard.
    pub fn this_one() -> Self {
        TheDesktop {
            ask: Box::new(|over, question| {
                MessageDialog::builder(over, question, "Remove a PGP Key")
                    .with_style(crate::presentation::asking::yes_no_where_enter_answers_no())
                    .build()
                    .show_modal()
                    == ID_YES
            }),
            clipboard: Box::new(|text| Clipboard::get().set_text(text)),
        }
    }
}

/// The window, built and not yet shown.
#[derive(Clone)]
pub struct KeyManager {
    /// The dialog itself.
    pub dialog: Dialog,
    /// The keys, one row each.
    pub list: ListCtrl,
    /// The line every answer is written to.
    pub status: StaticText,
    import_from_a_file: Button,
    showing: Rc<RefCell<Vec<KeyListing>>>,
    underneath: Rc<TheKeysUnderneath>,
    desktop: Rc<TheDesktop>,
    a11y: Arc<Accessibility>,
}

impl KeyManager {
    /// Put the list's cursor on a row, as a person arrowing to it does.
    pub fn choose(&self, row: usize) {
        land_the_row_cursor(&self.list, Some(row));
    }

    /// How many keys the list shows.
    pub fn rows(&self) -> usize {
        self.showing.borrow().len()
    }

    /// The key the list's cursor is on, or a sentence saying to choose one.
    fn the_chosen_key(&self) -> Option<KeyListing> {
        let chosen = the_row_chosen(
            &self.showing.borrow(),
            self.list.get_first_selected_item().into(),
        )
        .cloned();
        if chosen.is_none() {
            self.say(&nothing_chosen_named("key"), Priority::High);
        }
        chosen
    }

    /// What Remove does: ask, and remove the chosen key on a yes.
    pub fn remove_the_chosen_key(&self) {
        let Some(key) = self.the_chosen_key() else {
            return;
        };
        if !(self.desktop.ask)(&self.dialog, &pgp_keys::removal_question(&key)) {
            self.say(pgp_keys::NOTHING_WAS_REMOVED, Priority::Normal);
            return;
        }
        let removed = (self.underneath.remove)(&key.fingerprint);
        let was_on = self.list.get_first_selected_item();
        self.read_the_keys_again();
        land_the_row_cursor(&self.list, the_row_after_removing(was_on, self.rows()));
        let how_loud = if removed.is_ok() {
            Priority::Normal
        } else {
            Priority::High
        };
        self.say(&pgp_keys::what_removing_did(&key, &removed), how_loud);
    }

    /// The chosen key's public half, or a sentence saying why there is none.
    fn the_chosen_public_half(&self) -> Option<(KeyListing, String)> {
        let key = self.the_chosen_key()?;
        match (self.underneath.export_public)(&key.fingerprint) {
            Ok(Some(armour)) => Some((key, armour)),
            Ok(None) => {
                self.say(
                    "That key is no longer here, so there is nothing to export.",
                    Priority::High,
                );
                None
            }
            Err(problem) => {
                self.say(
                    &format!("The key could not be read: {problem}."),
                    Priority::High,
                );
                None
            }
        }
    }

    /// What Copy Public Key does.
    pub fn copy_the_chosen_key(&self) {
        let Some((key, armour)) = self.the_chosen_public_half() else {
            return;
        };
        if (self.desktop.clipboard)(&armour) {
            self.say(&pgp_keys::on_the_clipboard(&key), Priority::Normal);
        } else {
            self.say(
                "The clipboard would not take it. Another program may be holding it.",
                Priority::High,
            );
        }
    }

    /// What Export Public Key does: ask where, then write the public half.
    fn export_the_chosen_key(&self) {
        let Some((key, armour)) = self.the_chosen_public_half() else {
            return;
        };
        let picker = FileDialog::builder(&self.dialog)
            .with_message("Export a public key")
            .with_default_file(&format!("{}.asc", key.key_id))
            .with_wildcard("PGP key files (*.asc)|*.asc|All files (*.*)|*.*")
            .with_style(FileDialogStyle::Save | FileDialogStyle::OverwritePrompt)
            .build();
        if picker.show_modal() != ID_OK {
            return;
        }
        let Some(path) = picker.get_path() else {
            self.say("No file name was chosen.", Priority::High);
            return;
        };
        match std::fs::write(&path, armour) {
            Ok(()) => self.say(&pgp_keys::saved_to(&key, &path), Priority::Normal),
            Err(problem) => self.say(
                &format!("The public key could not be saved to {path}: {problem}."),
                Priority::High,
            ),
        }
    }

    /// What Import from File does.
    fn import_from_a_file(&self) {
        let picker = FileDialog::builder(&self.dialog)
            .with_message("Import PGP keys")
            .with_wildcard(
                "PGP key files (*.asc;*.key;*.gpg)|*.asc;*.key;*.gpg|All files (*.*)|*.*",
            )
            .with_style(FileDialogStyle::Open | FileDialogStyle::FileMustExist)
            .build();
        if picker.show_modal() != ID_OK {
            return;
        }
        let Some(chosen) = picker.get_path() else {
            self.say("No file was chosen.", Priority::High);
            return;
        };
        // The path and never the contents: a key file's bytes are the highest
        // value secret this program handles.
        match std::fs::read_to_string(&chosen) {
            Ok(text) => self.import_text(&text),
            Err(_) => self.say(
                "That file could not be read. A PGP key exported as text is what this wants.",
                Priority::High,
            ),
        }
    }

    /// What Paste a Key does.
    fn paste_a_key(&self) {
        let paste = build_the_paste_dialog(&self.dialog);
        let pasted = (paste.dialog.show_modal() == ID_OK).then(|| paste.text.get_value());
        paste.dialog.destroy();
        if let Some(text) = pasted {
            self.import_text(&text);
        }
    }

    /// Import every key a text holds, and say what became of each.
    pub fn import_text(&self, text: &str) {
        let answers = (self.underneath.import)(text);
        self.read_the_keys_again();
        let first = answers
            .iter()
            .find_map(|answer| answer.listing.as_ref())
            .and_then(|key| {
                self.showing
                    .borrow()
                    .iter()
                    .position(|shown| shown.fingerprint == key.fingerprint)
            });
        land_the_row_cursor(&self.list, first.or((self.rows() > 0).then_some(0)));
        let said = answers
            .into_iter()
            .map(|answer| answer.said)
            .collect::<Vec<String>>()
            .join(" ");
        self.say(&said, Priority::Normal);
    }

    /// Read the keys again and put them in the list, saying so if they could
    /// not be read.
    fn read_the_keys_again(&self) {
        let keys = match (self.underneath.every_key)() {
            Ok(keys) => keys,
            Err(problem) => {
                self.say(
                    &format!("The keys here could not be read: {problem}."),
                    Priority::High,
                );
                Vec::new()
            }
        };
        fill(&self.list, &keys);
        *self.showing.borrow_mut() = keys;
    }

    /// Show an answer on the line and say it, from the one call that does both.
    fn say(&self, sentence: &str, how_loud: Priority) {
        said_and_shown(&self.status, &self.a11y, sentence, how_loud);
    }
}

/// Build the key manager over these keys, without showing it.
pub fn build<W: WxWidget>(
    parent: &W,
    underneath: TheKeysUnderneath,
    desktop: TheDesktop,
    a11y: &Arc<Accessibility>,
) -> KeyManager {
    let dialog = Dialog::builder(parent, TITLE)
        .with_size(860, 560)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    if let Some(palette) = theme::current_from_stored_config() {
        theme::paint(&dialog, palette.main_surface());
    }
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // What keys can do in this build, first, where a person reads it before
    // anything else (#49). A read-only box rather than a line of text, so Tab
    // reaches it and a screen reader can read it again line by line.
    let limits_label = StaticText::builder(&dialog)
        .with_label(THE_LIMITS_LABEL)
        .build();
    sizer.add(&limits_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let limits = TextCtrl::builder(&dialog)
        .with_value(pgp_keys::WHAT_KEYS_CAN_DO_HERE)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly | TextCtrlStyle::WordWrap)
        .with_size(Size::new(-1, 90))
        .build();
    set_accessible_name(&limits, &what_the_label_says(THE_LIMITS_LABEL));
    sizer.add(&limits, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let list_label = StaticText::builder(&dialog)
        .with_label(THE_LIST_LABEL)
        .build();
    sizer.add(&list_label, 0, SizerFlag::Left, 8);
    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&list, &what_the_label_says(THE_LIST_LABEL));
    for (at, heading) in THE_COLUMNS.iter().enumerate() {
        // The name widest, then the kind, which says "Private key, locked
        // with a passphrase" for a locked key and would be cut off at the
        // width of the rest.
        let width = match at {
            0 => 240,
            1 => 230,
            _ => 110,
        };
        list.insert_column(at as i64, heading, ListColumnFormat::Left, width);
    }
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // Nothing raises a notification when a line of text changes, so an answer
    // only shown here is an answer nobody working by ear gets; every answer
    // goes through `said_and_shown`.
    let status = StaticText::builder(&dialog).with_label("").build();
    set_accessible_name(&status, WHAT_HAPPENED);
    sizer.add(&status, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let a_button = |label: &str| {
        let button = Button::builder(&dialog).with_label(label).build();
        set_accessible_name(&button, &what_the_label_says(label));
        button
    };
    let import_from_a_file = a_button(IMPORT_FROM_A_FILE);
    let paste = a_button(PASTE_A_KEY);
    let export = a_button(EXPORT_THE_PUBLIC_KEY);
    let copy = a_button(COPY_THE_PUBLIC_KEY);
    let remove = a_button(REMOVE_THE_KEY);
    let close = Button::builder(&dialog)
        .with_label(CLOSE)
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&close, &what_the_label_says(CLOSE));
    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    for button in [&import_from_a_file, &paste, &export, &copy, &remove, &close] {
        buttons.add(button, 0, SizerFlag::All, 4);
    }
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dialog.set_sizer(sizer, true);

    let manager = KeyManager {
        dialog,
        list,
        status,
        import_from_a_file,
        showing: Rc::new(RefCell::new(Vec::new())),
        underneath: Rc::new(underneath),
        desktop: Rc::new(desktop),
        a11y: Arc::clone(a11y),
    };
    manager.read_the_keys_again();
    land_the_row_cursor(&manager.list, (manager.rows() > 0).then_some(0));

    import_from_a_file.on_click({
        let manager = manager.clone();
        move |_| manager.import_from_a_file()
    });
    paste.on_click({
        let manager = manager.clone();
        move |_| manager.paste_a_key()
    });
    export.on_click({
        let manager = manager.clone();
        move |_| manager.export_the_chosen_key()
    });
    copy.on_click({
        let manager = manager.clone();
        move |_| manager.copy_the_chosen_key()
    });
    remove.on_click({
        let manager = manager.clone();
        move |_| manager.remove_the_chosen_key()
    });
    close.on_click(move |_| dialog.end_modal(ID_CANCEL));
    manager
}

/// Which row to put the cursor on once one has gone: the row that moved up
/// into the gap, or the last one when the gap was at the end.
fn the_row_after_removing(was_on: i32, left: usize) -> Option<usize> {
    if left == 0 {
        return None;
    }
    Some(usize::try_from(was_on).unwrap_or(0).min(left - 1))
}

/// Put the cursor on a row, selected and focused together, so a screen
/// reader's own cursor goes with it.
fn land_the_row_cursor(list: &ListCtrl, at: Option<usize>) {
    let Some(at) = at else { return };
    let at = at as i64;
    list.set_item_state(
        at,
        ListItemState::Selected | ListItemState::Focused,
        ListItemState::Selected | ListItemState::Focused,
    );
    list.ensure_visible(at);
}

/// Put the keys into the control, one row each.
fn fill(list: &ListCtrl, keys: &[KeyListing]) {
    list.delete_all_items();
    for (at, key) in keys.iter().enumerate() {
        let row = pgp_keys::what_a_row_says(key, WhichLocale::ThisComputer);
        // Paired with `THE_COLUMNS` by position and by length, so a heading
        // added without a cell does not compile rather than being silence.
        let cells: [String; THE_COLUMNS.len()] = [
            row.name,
            row.kind,
            row.key_id,
            row.fingerprint,
            row.created,
            row.expires,
            row.can,
        ];
        list.insert_item(at as i64, &cells[0], None);
        for (column, cell) in cells.iter().enumerate().skip(1) {
            list.set_item_text_by_column(at as i64, column as i32, cell);
        }
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
    let dialog = Dialog::builder(parent, "Paste a Key")
        .with_size(640, 420)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    if let Some(palette) = theme::current_from_stored_config() {
        theme::paint(&dialog, palette.main_surface());
    }
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let label = StaticText::builder(&dialog)
        .with_label(THE_KEY_TEXT_LABEL)
        .build();
    sizer.add(&label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let text = TextCtrl::builder(&dialog)
        .with_style(TextCtrlStyle::MultiLine)
        .build();
    set_accessible_name(&text, &what_the_label_says(THE_KEY_TEXT_LABEL));
    keep_a_history(&text);
    sizer.add(&text, 1, SizerFlag::Expand | SizerFlag::All, 8);
    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dialog)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dialog)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    buttons.add(&ok, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dialog.set_sizer(sizer, true);
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
    match where_focus_goes(manager.rows()) {
        WhereFocusGoes::TheFirstKey => {
            manager.list.set_focus();
            land_the_row_cursor(&manager.list, Some(0));
        }
        WhereFocusGoes::ImportFromAFile => manager.import_from_a_file.set_focus(),
    }
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
