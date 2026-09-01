//! wxdragon Manager Dialogs
//!
//! Contact, Filter, Tag, and Signature managers sharing a generic modal loop.
//!
//! Every answer these windows give is shown on a line of text above their
//! buttons and said out loud, from one call. Nothing raises a notification for
//! such a line and it is not somewhere anybody navigating by ear goes, so an
//! answer that was only shown was an answer nobody got. Being told to select
//! something first is said above the ordinary run of outcomes, because it is
//! the answer to the key just pressed.

use crate::application::filters::{
    A_FIELD_A_RULE_MAY_NAME, A_WAY_A_RULE_MAY_MATCH, a_way_of_matching_compares_against_nothing,
    the_field_those_words_name, the_way_of_matching_those_words_name, the_words_for_a_field,
    the_words_for_a_way_of_matching,
};
use crate::application::saved_searches::Question;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::{
    name_from_label, set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::manager_words;
use crate::presentation::status_line::said_and_shown;
use crate::presentation::theme;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

// ── Shared Button IDs ──────────────────────────────────────────────────────

const ID_MGR_ADD: Id = ID_HIGHEST + 300;
const ID_MGR_EDIT: Id = ID_HIGHEST + 301;
const ID_MGR_DELETE: Id = ID_HIGHEST + 302;
const ID_MGR_SYNC: Id = ID_HIGHEST + 303;

// ── Shared Helpers ─────────────────────────────────────────────────────────

/// Get selected item index from a ListCtrl (-1 means none).
pub(crate) fn get_selected(list: &ListCtrl) -> Option<usize> {
    let sel = list.get_first_selected_item();
    if sel >= 0 { Some(sel as usize) } else { None }
}

/// Add a label + TextCtrl row to a FlexGridSizer. Returns the TextCtrl.
fn add_field(parent: &Dialog, sizer: &FlexGridSizer, label: &str) -> TextCtrl {
    let lbl = StaticText::builder(parent).with_label(label).build();
    let field = TextCtrl::builder(parent).build();
    // The visible label is a separate control, which wxWidgets never associates
    // with the field, so without this the field announces as just "edit".
    set_accessible_name(&field, &name_from_label(label));
    sizer.add(&lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    sizer.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    field
}

/// Select a Choice item by matching its string value.
fn select_choice_by_string(choice: &Choice, value: &str) {
    let count = choice.get_count();
    for i in 0..count {
        if choice.get_string(i).as_deref() == Some(value) {
            choice.set_selection(i);
            return;
        }
    }
}

/// Get the currently selected string from a Choice.
fn get_choice_string(choice: &Choice) -> Option<String> {
    choice.get_string_selection()
}

// ── Generic Manager Dialog Loop ────────────────────────────────────────────

/// The chrome every manager dialog shares: the dialog, its sizer, the item
/// list, the line of text every answer is written on, and the way those
/// sentences are said out loud.
///
/// This used to say the line announced changes, and it never did: for as long
/// as the description existed, every answer these windows gave was written
/// there and said nowhere. Both go through one call now, so the line and the
/// ear cannot come apart again.
struct ManagerChrome<'a> {
    dialog: &'a Dialog,
    main_sizer: &'a BoxSizer,
    list: &'a ListCtrl,
    status_text: &'a StaticText,
    /// Owned rather than borrowed: Delete's own click (see [`delete_selected`])
    /// runs from a button handler that must outlive this function call, and a
    /// borrowed `&Accessibility` cannot go into one. Cloning an `Arc` is a
    /// refcount bump, not a copy of the accessibility layer itself.
    a11y: Arc<Accessibility>,
}

/// The rows a manager dialog's Add/Edit/Delete loop is working on, and
/// whether anything has changed yet. Bundled together, and the bundle
/// shared once as `Rc<RefCell<...>>`, because Delete's own click (see
/// [`delete_selected`]) reaches both from outside the loop that used to be
/// the only place either one changed.
pub struct ManagerState<T> {
    pub working: Vec<T>,
    pub changed: bool,
}

/// Delete the selected row, immediately: remove it from the working rows,
/// repaint the list, and say what happened. Runs directly from the Delete
/// button's own click, never through `end_modal`, because deleting a row
/// never needed to leave this dialog in the first place.
///
/// Moved here, verbatim, from what used to be `run_manager_loop`'s own
/// `ID_MGR_DELETE` arm. `end_modal` immediately followed by another
/// `show_modal()` on the same dialog, with nothing yielded to the Windows
/// message pump in between, is how NVDA lost both a button's own
/// announcement and the dialog reappearing (confirmed live against Account
/// Manager's Sign In Again, the one button among these a screen reader has
/// actually been run against: NVDA jumped straight from the button's own
/// name to its own generic "Wixen Mail, unavailable"). Add and Edit still
/// end this dialog's own modal loop, because they genuinely need to: each
/// opens a real nested Add/Edit dialog of its own.
///
/// Public so a test can call it directly with a real list and a real line
/// of text, without a human closing a live modal.
/// Which row to put the cursor on after removing one, out of however many
/// are left.
///
/// Repainting a list leaves nothing selected. So deleting a rule took the
/// cursor away, and pressing Delete again said "select a filter to delete",
/// which sounds like a refusal and is really the repaint; getting back meant
/// tabbing to the list and arrowing down to where you already were.
///
/// The row that moved up into the gap, which is the next one along and the
/// one somebody deleting several in a row wants. At the end of the list there
/// is no such row, so it is the new last one instead.
fn the_row_to_select_after_removing(removed: usize, left: usize) -> Option<usize> {
    if left == 0 {
        return None;
    }
    Some(removed.min(left - 1))
}

/// Put the cursor back on a list that has just been repainted.
///
/// Selected and focused together, and made visible: selecting alone leaves
/// the screen reader's own cursor where it was, so the person hears nothing
/// move.
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

pub fn delete_selected<T: Clone>(
    state: &Rc<RefCell<ManagerState<T>>>,
    list: &ListCtrl,
    status_text: &StaticText,
    a11y: &Accessibility,
    kind: &str,
    populate: impl Fn(&ListCtrl, &[T]),
    name_fn: impl Fn(&T) -> String,
) {
    if let Some(idx) = get_selected(list) {
        let name = name_fn(&state.borrow().working[idx]);
        let mut s = state.borrow_mut();
        s.working.remove(idx);
        s.changed = true;
        drop(s);
        let left = state.borrow().working.len();
        populate(list, &state.borrow().working);
        // Back on the row that moved up into the gap. Without this the
        // repaint leaves nothing selected, so a second Delete says to select
        // something and sounds like a refusal.
        land_the_row_cursor(list, the_row_to_select_after_removing(idx, left));
        said_and_shown(
            status_text,
            a11y,
            &manager_words::deleted(kind, &name, left),
            Priority::Normal,
        );
    } else {
        said_and_shown(
            status_text,
            a11y,
            &manager_words::nothing_selected(kind, "delete"),
            Priority::High,
        );
    }
}

/// What a list of these still needs before its window can close: nothing.
///
/// A filter list, a tag list or a signature list with nothing in it is a list
/// somebody emptied on purpose, and there is no reason to hold the window open
/// over it. Named rather than written as `|_| None` at three call sites, so
/// the decision reads as a decision that was made.
fn nothing_stops_this_closing<T>(_rows: &[T]) -> Option<&'static str> {
    None
}

/// Run the standard Add/Edit/Delete modal loop shared by all manager dialogs.
///
/// `kind` is the word this window's rows are, "filter", "tag" or
/// "signature", asked of [`manager_words`] for what to say about a row
/// somebody just changed. Without it, "Deleted: Jane Smith" never said
/// whether a contact, a filter, a tag or a signature had gone, and read
/// identically to the mail path's own sentence for a message leaving a
/// server besides.
///
/// Delete runs from its own button click and never reaches this loop at
/// all; see [`delete_selected`]. Add and Edit still do, because each opens
/// a nested dialog of its own and genuinely needs `end_modal` to get there.
///
/// `open_one` is the sub-dialog both Add and Edit reach, with `None` for a new
/// row and the row itself for an existing one. One function rather than two,
/// because every window on this loop passed the same call twice with only that
/// argument differing, and two closures over one dialog is how the two come to
/// open it differently.
///
/// `what_it_still_needs` is asked on the way out, and a sentence back from it
/// refuses the Close and keeps the window open. Four of the five windows on
/// this loop answer `None` to everything, because a filter list or a tag list
/// with nothing in it is a list somebody emptied on purpose. A condition list
/// is not: a saved search that asks nothing about a message is refused by the
/// store as well, and a window is where somebody can be told why.
///
/// Returns `true` if any changes were made.
fn run_manager_loop<T: Clone + 'static>(
    chrome: ManagerChrome<'_>,
    kind: &str,
    working: &mut Vec<T>,
    populate: impl Fn(&ListCtrl, &[T]) + Copy + 'static,
    open_one: impl Fn(&Dialog, Option<&T>) -> Option<T>,
    name_fn: impl Fn(&T) -> String + Copy + 'static,
    what_it_still_needs: impl Fn(&[T]) -> Option<&'static str> + 'static,
) -> bool {
    let ManagerChrome {
        dialog,
        main_sizer,
        list,
        status_text,
        a11y,
    } = chrome;

    // Create and attach buttons
    let add_btn = Button::builder(dialog)
        .with_label("&Add...")
        .with_id(ID_MGR_ADD)
        .build();
    let edit_btn = Button::builder(dialog)
        .with_label("&Edit...")
        .with_id(ID_MGR_EDIT)
        .build();
    let del_btn = Button::builder(dialog)
        .with_label("&Delete")
        .with_id(ID_MGR_DELETE)
        .build();
    let close_btn = Button::builder(dialog)
        .with_label("&Close")
        .with_id(ID_OK)
        .build();

    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add(&add_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&edit_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&del_btn, 0, SizerFlag::All, 4);
    btn_sizer.add_spacer(16);
    btn_sizer.add(&close_btn, 0, SizerFlag::All, 4);

    main_sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    main_sizer.add(status_text, 0, SizerFlag::Expand | SizerFlag::All, 4);
    dialog.set_sizer(*main_sizer, true);

    // The working rows and the "did anything change" flag, shared between
    // this loop and Delete's own button click below. Delete mutates both
    // from outside this loop entirely now (see `delete_selected`), so a
    // plain `&mut Vec<T>` and a plain `bool` local, each only ever good for
    // the length of this call, are no longer enough to hold them.
    let state: Rc<RefCell<ManagerState<T>>> = Rc::new(RefCell::new(ManagerState {
        working: std::mem::take(working),
        changed: false,
    }));

    add_btn.on_click({
        let d = *dialog;
        move |_| {
            d.end_modal(ID_MGR_ADD);
        }
    });
    edit_btn.on_click({
        let d = *dialog;
        move |_| {
            d.end_modal(ID_MGR_EDIT);
        }
    });
    del_btn.on_click({
        let list = *list;
        let status_text = *status_text;
        let a11y = a11y.clone();
        let kind = kind.to_string();
        let state = state.clone();
        move |_| {
            delete_selected(&state, &list, &status_text, &a11y, &kind, populate, name_fn);
        }
    });
    close_btn.on_click({
        let d = *dialog;
        let state = state.clone();
        move |event| {
            // Consuming the click is what makes a refusal stick, the same way
            // the condition editor's empty-pattern refusal does. Called in
            // both branches, because a Close that ends the modal itself does
            // not want the default handler closing it a second time.
            event.event.skip(false);
            if let Some(needed) = what_it_still_needs(&state.borrow().working) {
                a_sub_dialog_needs(&d, "Not closed", needed);
                return;
            }
            d.end_modal(ID_OK);
        }
    });

    populate(list, &state.borrow().working);

    loop {
        match dialog.show_modal() {
            r if r == ID_MGR_ADD => {
                if let Some(item) = open_one(dialog, None) {
                    let name = name_fn(&item);
                    let mut s = state.borrow_mut();
                    s.working.push(item);
                    s.changed = true;
                    drop(s);
                    let left = state.borrow().working.len();
                    populate(list, &state.borrow().working);
                    said_and_shown(
                        status_text,
                        &a11y,
                        &manager_words::added(kind, &name, left),
                        Priority::Normal,
                    );
                }
            }
            r if r == ID_MGR_EDIT => {
                if let Some(idx) = get_selected(list) {
                    let current = state.borrow().working[idx].clone();
                    if let Some(edited) = open_one(dialog, Some(&current)) {
                        let name = name_fn(&edited);
                        let mut s = state.borrow_mut();
                        s.working[idx] = edited;
                        s.changed = true;
                        drop(s);
                        let left = state.borrow().working.len();
                        populate(list, &state.borrow().working);
                        said_and_shown(
                            status_text,
                            &a11y,
                            &manager_words::updated(kind, &name, left),
                            Priority::Normal,
                        );
                    }
                } else {
                    said_and_shown(
                        status_text,
                        &a11y,
                        &manager_words::nothing_selected(kind, "edit"),
                        Priority::High,
                    );
                }
            }
            _ => break,
        }
    }

    *working = state.borrow().working.clone();
    // Taken down once the loop is out. wxWidgets does not free a dialog when
    // the Rust value goes, so every manager window ever opened stayed for the
    // life of the session; `wx_compose` hit the same thing and says so where
    // it fixed it. After the state is read, because the list belongs to the
    // dialog.
    dialog.destroy();
    state.borrow().changed
}

/// Create the standard manager dialog shell: dialog + sizer + list + status.
///
/// Shared by the Filter, Tag and Signature managers' own list windows, so
/// painting the dialog and the list once here reaches all three the moment
/// each caller forwards a palette into this call. The status line is left to
/// Windows: it carries no row content of its own, matching the rule this
/// round follows throughout for a `StaticText` that is not one. `None` means
/// high contrast is on, or the system is set up in a way this application
/// should not paint over, so nothing is set here and Windows decides.
///
/// `holds` is what the list is called when it is read out. It used to be
/// `"Items"` for every manager window, which is the generic word rather than
/// an answer: somebody landing on the list heard the same thing whether it was
/// holding filters, tags, signatures or the conditions of a saved search. The
/// count and the position in the set come from Windows' own provider for a
/// native list in report mode, on both channels, so this is the one thing
/// about the list this code has to say.
pub fn make_shell(
    parent: &Frame,
    title: &str,
    holds: &str,
    w: i32,
    h: i32,
    palette: Option<theme::Palette>,
) -> (Dialog, BoxSizer, ListCtrl, StaticText) {
    let dialog = Dialog::builder(parent, title)
        .with_size(w, h)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&list, holds);
    let status = StaticText::builder(&dialog).with_label(" ").build();

    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
        theme::paint(&list, palette.main_surface());
    }

    (dialog, sizer, list, status)
}

// ══════════════════════════════════════════════════════════════════════════════
// Contact Manager: Comprehensive Google Contacts-style fields
// ══════════════════════════════════════════════════════════════════════════════

/// Phone number with type label
#[derive(Debug, Clone, PartialEq)]
pub struct PhoneItem {
    pub label: String,
    pub number: String,
}

/// Email address with type label
#[derive(Debug, Clone, PartialEq)]
pub struct EmailItem {
    pub label: String,
    pub address: String,
}

/// Structured physical address with type label
#[derive(Debug, Clone, PartialEq)]
pub struct AddressItem {
    pub label: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

/// User-defined custom field
#[derive(Debug, Clone, PartialEq)]
pub struct CustomFieldItem {
    pub label: String,
    pub value: String,
}

/// Compared whole, because that comparison is what decides whether a contact
/// is written back. Every field the editor can show is in here, so two of these
/// being equal is the whole of "nobody changed this row", and a field added to
/// the editor later joins the comparison without anybody remembering to.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactEntry {
    pub id: String,
    // ── Name & Identity ─────────────────────────────────────────────────
    pub name: String,
    /// The first part of the person's name, shown in a box of its own so it
    /// can be corrected. Filled from what an address book recorded, or from
    /// one guess at the whole name when nothing recorded the parts.
    pub given_name: String,
    /// The other part, kept whole however many spaces it carries.
    pub family_name: String,
    pub nickname: String,
    // ── Organization ────────────────────────────────────────────────────
    pub company: String,
    pub department: String,
    pub job_title: String,
    // ── Multi-value contact info ────────────────────────────────────────
    pub emails: Vec<EmailItem>,
    pub phones: Vec<PhoneItem>,
    pub addresses: Vec<AddressItem>,
    // ── Other standard fields ───────────────────────────────────────────
    pub birthday: String,
    pub website: String,
    pub relationship: String,
    pub notes: String,
    // ── Custom fields ───────────────────────────────────────────────────
    pub custom_fields: Vec<CustomFieldItem>,
    // ── Avatar ──────────────────────────────────────────────────────────
    pub avatar_url: String,
    // ── Flags ───────────────────────────────────────────────────────────
    pub favorite: bool,
}

impl ContactEntry {
    /// Primary email (first in list, or empty)
    pub fn primary_email(&self) -> &str {
        self.emails
            .first()
            .map(|e| e.address.as_str())
            .unwrap_or("")
    }
    /// Primary phone (first in list, or empty)
    fn primary_phone(&self) -> &str {
        self.phones.first().map(|p| p.number.as_str()).unwrap_or("")
    }
}

#[derive(Debug, Clone)]
pub enum ContactManagerAction {
    None,
    Updated(Vec<ContactEntry>),
    SyncRequested,
}

// ── Label constants for dropdowns ────────────────────────────────────────────

const EMAIL_LABELS: &[&str] = &["Personal", "Work", "Other"];
const PHONE_LABELS: &[&str] = &[
    "Mobile", "Home", "Work", "Work Fax", "Home Fax", "Pager", "Other",
];
const ADDRESS_LABELS: &[&str] = &["Home", "Work", "Other"];

// ── Country Data ────────────────────────────────────────────────────────────

/// Comprehensive country list for address entry (alphabetical)
const COUNTRIES: &[&str] = &[
    "Afghanistan",
    "Albania",
    "Algeria",
    "Andorra",
    "Angola",
    "Argentina",
    "Armenia",
    "Australia",
    "Austria",
    "Azerbaijan",
    "Bahamas",
    "Bahrain",
    "Bangladesh",
    "Barbados",
    "Belarus",
    "Belgium",
    "Belize",
    "Bolivia",
    "Bosnia and Herzegovina",
    "Botswana",
    "Brazil",
    "Brunei",
    "Bulgaria",
    "Cambodia",
    "Cameroon",
    "Canada",
    "Chile",
    "China",
    "Colombia",
    "Costa Rica",
    "Croatia",
    "Cuba",
    "Cyprus",
    "Czech Republic",
    "Denmark",
    "Dominican Republic",
    "Ecuador",
    "Egypt",
    "El Salvador",
    "Estonia",
    "Ethiopia",
    "Finland",
    "France",
    "Georgia",
    "Germany",
    "Ghana",
    "Greece",
    "Guatemala",
    "Honduras",
    "Hong Kong",
    "Hungary",
    "Iceland",
    "India",
    "Indonesia",
    "Iran",
    "Iraq",
    "Ireland",
    "Israel",
    "Italy",
    "Jamaica",
    "Japan",
    "Jordan",
    "Kazakhstan",
    "Kenya",
    "Kuwait",
    "Latvia",
    "Lebanon",
    "Libya",
    "Lithuania",
    "Luxembourg",
    "Malaysia",
    "Mexico",
    "Moldova",
    "Monaco",
    "Mongolia",
    "Morocco",
    "Mozambique",
    "Myanmar",
    "Nepal",
    "Netherlands",
    "New Zealand",
    "Nicaragua",
    "Nigeria",
    "North Korea",
    "Norway",
    "Oman",
    "Pakistan",
    "Panama",
    "Paraguay",
    "Peru",
    "Philippines",
    "Poland",
    "Portugal",
    "Qatar",
    "Romania",
    "Russia",
    "Saudi Arabia",
    "Senegal",
    "Serbia",
    "Singapore",
    "Slovakia",
    "Slovenia",
    "South Africa",
    "South Korea",
    "Spain",
    "Sri Lanka",
    "Sudan",
    "Sweden",
    "Switzerland",
    "Syria",
    "Taiwan",
    "Tanzania",
    "Thailand",
    "Tunisia",
    "Turkey",
    "Uganda",
    "Ukraine",
    "United Arab Emirates",
    "United Kingdom",
    "United States",
    "Uruguay",
    "Uzbekistan",
    "Venezuela",
    "Vietnam",
    "Yemen",
    "Zambia",
    "Zimbabwe",
];

/// Get the default country based on the system locale.
fn get_default_country() -> &'static str {
    let lang = Locale::get_system_language();
    if let Some(canonical) = Locale::get_language_canonical_name(lang) {
        // canonical is like "en_US", "ja_JP", "de_DE"
        if let Some(code) = canonical.split('_').nth(1) {
            return match code {
                "US" => "United States",
                "GB" | "UK" => "United Kingdom",
                "CA" => "Canada",
                "AU" => "Australia",
                "NZ" => "New Zealand",
                "JP" => "Japan",
                "DE" => "Germany",
                "AT" => "Austria",
                "CH" => "Switzerland",
                "FR" => "France",
                "ES" => "Spain",
                "IT" => "Italy",
                "BR" => "Brazil",
                "MX" => "Mexico",
                "IN" => "India",
                "CN" => "China",
                "KR" => "South Korea",
                "RU" => "Russia",
                "SE" => "Sweden",
                "NO" => "Norway",
                "DK" => "Denmark",
                "FI" => "Finland",
                "NL" => "Netherlands",
                "BE" => "Belgium",
                "PT" => "Portugal",
                "PL" => "Poland",
                "IE" => "Ireland",
                "ZA" => "South Africa",
                "SG" => "Singapore",
                "PH" => "Philippines",
                "IL" => "Israel",
                "AE" => "United Arab Emirates",
                "SA" => "Saudi Arabia",
                "AR" => "Argentina",
                "CL" => "Chile",
                "CO" => "Colombia",
                "EG" => "Egypt",
                "NG" => "Nigeria",
                "KE" => "Kenya",
                "TW" => "Taiwan",
                "HK" => "Hong Kong",
                "TH" => "Thailand",
                "ID" => "Indonesia",
                "MY" => "Malaysia",
                "VN" => "Vietnam",
                "PK" => "Pakistan",
                "BD" => "Bangladesh",
                "TR" => "Turkey",
                "UA" => "Ukraine",
                "CZ" => "Czech Republic",
                "HU" => "Hungary",
                "RO" => "Romania",
                "GR" => "Greece",
                _ => "United States",
            };
        }
    }
    "United States"
}

/// Get country-specific address field labels (with accelerators).
///
/// Returns (region_label, code_label).
/// Accelerators avoid conflicts with &Country(C), &Type(T), &Street(S), C&ity(I).
/// Country-aware address field labels for contact editor.
pub(crate) fn get_address_field_labels(country: &str) -> (&'static str, &'static str) {
    match country {
        "United States" => ("St&ate:", "&ZIP Code:"),
        "United Kingdom" => ("Co&unty:", "&Postcode:"),
        "Canada" => ("Pro&vince:", "&Postal Code:"),
        "Australia" => ("St&ate:", "&Postcode:"),
        "Japan" => ("Pre&fecture:", "&Postal Code:"),
        "Germany" | "Austria" | "Switzerland" => ("St&ate/Land:", "&PLZ:"),
        "France" => ("Re&gion:", "Code &Postal:"),
        "Brazil" => ("St&ate:", "&CEP:"),
        "India" => ("St&ate:", "&PIN Code:"),
        "South Korea" | "China" => ("Pro&vince:", "&Postal Code:"),
        "Italy" => ("Pro&vince:", "&CAP:"),
        "Spain" => ("Pro&vince:", "&Postal Code:"),
        "Mexico" => ("St&ate:", "&Postal Code:"),
        "Ireland" => ("Co&unty:", "&Eircode:"),
        "Netherlands" => ("Pro&vince:", "&Postcode:"),
        _ => ("St&ate/Province:", "&Postal Code:"),
    }
}

// ── Contact Manager: Custom Loop with Live Search ──────────────────────────

/// What `show_contact_manager_dialog`'s own loop still needs after
/// construction: the controls to read from and, for the search field and the
/// list, to repaint whenever a search narrows what they show, plus the
/// shared state the live search, Delete's own click, and the Add/Edit arms
/// still inside the loop all reach into together.
pub struct ContactManagerDialogHandles {
    pub dialog: Dialog,
    pub search: TextCtrl,
    pub list: ListCtrl,
    pub status: StaticText,
    working: Rc<RefCell<Vec<ContactEntry>>>,
    index_map: Rc<RefCell<Vec<usize>>>,
    changed: Rc<RefCell<bool>>,
}

/// Build the Contact Manager's own list window without showing it.
///
/// Everything `show_contact_manager_dialog` used to do up to its own modal
/// loop, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// Takes `a11y` because Delete's own click (see [`delete_selected_contact`])
/// answers immediately, from inside this function's own button wiring,
/// rather than from `show_contact_manager_dialog`'s loop the way it used to.
pub fn build_contact_manager_dialog(
    parent: &Frame,
    contacts: &[ContactEntry],
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> ContactManagerDialogHandles {
    let dialog = Dialog::builder(parent, "Contact Manager")
        .with_size(700, 500)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // ── Search row: first in tab order for accessibility ────────────
    let search_row = BoxSizer::builder(Orientation::Horizontal).build();
    let search_lbl = StaticText::builder(&dialog).with_label("&Search:").build();
    let search_f = TextCtrl::builder(&dialog).build();
    set_accessible_name(&search_f, "Search");
    search_row.add(
        &search_lbl,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    search_row.add(&search_f, 1, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add_sizer(
        &search_row,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        4,
    );

    // ── Contact list ────────────────────────────────────────────────
    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&list, "Contacts");
    list.insert_column(0, "Name", ListColumnFormat::Left, 160);
    list.insert_column(1, "Email", ListColumnFormat::Left, 200);
    list.insert_column(2, "Phone", ListColumnFormat::Left, 130);
    list.insert_column(3, "Company", ListColumnFormat::Left, 140);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // ── Buttons ─────────────────────────────────────────────────────
    let add_btn = Button::builder(&dialog)
        .with_label("&Add...")
        .with_id(ID_MGR_ADD)
        .build();
    let edit_btn = Button::builder(&dialog)
        .with_label("&Edit...")
        .with_id(ID_MGR_EDIT)
        .build();
    let del_btn = Button::builder(&dialog)
        .with_label("&Delete")
        .with_id(ID_MGR_DELETE)
        .build();
    let sync_btn = Button::builder(&dialog)
        .with_label("S&ync")
        .with_id(ID_MGR_SYNC)
        .build();
    let close_btn = Button::builder(&dialog)
        .with_label("&Close")
        .with_id(ID_OK)
        .build();
    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add(&add_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&edit_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&del_btn, 0, SizerFlag::All, 4);
    btn_sizer.add_spacer(16);
    btn_sizer.add(&sync_btn, 0, SizerFlag::All, 4);
    btn_sizer.add_spacer(16);
    btn_sizer.add(&close_btn, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 4);

    let status = StaticText::builder(&dialog).with_label(" ").build();
    sizer.add(&status, 0, SizerFlag::Expand | SizerFlag::All, 4);
    dialog.set_sizer(sizer, true);

    // ── Shared state for live search ────────────────────────────────
    let working = Rc::new(RefCell::new(contacts.to_vec()));
    let index_map: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    // Whether anything has changed yet, shared the same way `working` and
    // `index_map` already are: Delete's own click below sets this directly,
    // outside the loop that used to be the only place anything did.
    let changed: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

    // Initial population
    populate_contacts_filtered(&list, &working.borrow(), "", &mut index_map.borrow_mut());

    // ── Live search: update results as user types ──────────────────
    search_f.on_text_changed({
        let w = working.clone();
        let m = index_map.clone();
        let l = list;
        let sf = search_f;
        move |_| {
            let query = sf.get_value();
            let contacts = w.borrow();
            populate_contacts_filtered(&l, &contacts, &query, &mut m.borrow_mut());
        }
    });

    // ── Button handlers ─────────────────────────────────────────────
    add_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_MGR_ADD);
        }
    });
    edit_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_MGR_EDIT);
        }
    });
    // Delete answers immediately from its own click and never ends this
    // dialog's modal loop; see `delete_selected_contact`'s own doc comment
    // for why (the same reason `wx_managers::delete_selected` gives for the
    // Filter, Tag and Signature managers' shared loop).
    del_btn.on_click({
        let search = search_f;
        let a11y = a11y.clone();
        let working = working.clone();
        let index_map = index_map.clone();
        let changed = changed.clone();
        move |_| {
            delete_selected_contact(
                &list, &search, &status, &a11y, &working, &index_map, &changed,
            );
        }
    });
    sync_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_MGR_SYNC);
        }
    });
    close_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_OK);
        }
    });

    // Set focus to search field for accessibility
    search_f.set_focus();

    // Painted last, after the list's columns are inserted and its first
    // population has run: nothing in this codebase proves whether a native
    // list-view control keeps a manually set background colour across
    // `InsertColumn`, the same caution the Account Manager's own list takes.
    // `None` means high contrast is on, or the system is set up in a way
    // this application should not paint over, so nothing is set here and
    // Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
        theme::paint(&search_f, palette.main_surface());
        theme::paint(&list, palette.main_surface());
    }

    ContactManagerDialogHandles {
        dialog,
        search: search_f,
        list,
        status,
        working,
        index_map,
        changed,
    }
}

/// Delete the selected contact, immediately: remove it from the working
/// rows, repaint the filtered list, and say what happened. Runs directly
/// from the Delete button's own click, never through `end_modal`, because
/// deleting a contact never needed to leave this dialog in the first place.
///
/// Moved here, verbatim, from what used to be `show_contact_manager_dialog`'s
/// own `ID_MGR_DELETE` arm; see [`delete_selected`]'s own doc comment (the
/// Filter, Tag and Signature managers' shared loop) for the mechanism this
/// fixes. Add and Edit still end this dialog's own modal loop, because each
/// opens a real nested Add/Edit Contact dialog of its own.
///
/// Public so a test can call it directly with a real list and a real line
/// of text, without a human closing a live modal.
pub fn delete_selected_contact(
    list: &ListCtrl,
    search: &TextCtrl,
    status: &StaticText,
    a11y: &Accessibility,
    working: &Rc<RefCell<Vec<ContactEntry>>>,
    index_map: &Rc<RefCell<Vec<usize>>>,
    changed: &Rc<RefCell<bool>>,
) {
    if let Some(sel) = get_selected(list) {
        let found = {
            let map = index_map.borrow();
            let w = working.borrow();
            map.get(sel).map(|&idx| (idx, w[idx].name.clone()))
        };
        let Some((working_idx, name)) = found else {
            return;
        };
        working.borrow_mut().remove(working_idx);
        *changed.borrow_mut() = true;
        let query = search.get_value();
        populate_contacts_filtered(list, &working.borrow(), &query, &mut index_map.borrow_mut());
        // The row that moved up into the gap. Counted against what the list
        // is showing rather than everything held, because a search may be
        // narrowing it and the cursor belongs on a row somebody can see.
        let showing = index_map.borrow().len();
        land_the_row_cursor(list, the_row_to_select_after_removing(sel, showing));
        said_and_shown(
            status,
            a11y,
            &manager_words::deleted(manager_words::CONTACT, &name, showing),
            Priority::Normal,
        );
    } else {
        said_and_shown(
            status,
            a11y,
            &manager_words::nothing_selected(manager_words::CONTACT, "delete"),
            Priority::High,
        );
    }
}

pub fn show_contact_manager_dialog(
    parent: &Frame,
    contacts: &[ContactEntry],
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> ContactManagerAction {
    let ContactManagerDialogHandles {
        dialog,
        search: search_f,
        list,
        status,
        working,
        index_map,
        changed,
    } = build_contact_manager_dialog(parent, contacts, a11y, palette);

    // ── Modal loop ──────────────────────────────────────────────────
    // Delete no longer has an arm here: it answers from its own click (see
    // `delete_selected_contact`) and never ends this dialog to get here.
    // Sync does still end it, on purpose, unlike Delete: choosing Sync closes
    // this dialog for good and hands off to a background sync outside it
    // (`managers::manage_contacts` in the caller), so `end_modal` here is the
    // one legitimate way out, the same as Close.
    loop {
        match dialog.show_modal() {
            r if r == ID_MGR_ADD => {
                if let Some(item) = show_contact_edit(&dialog, None, palette, a11y) {
                    let name = item.name.clone();
                    working.borrow_mut().push(item);
                    *changed.borrow_mut() = true;
                    let query = search_f.get_value();
                    let w = working.borrow();
                    populate_contacts_filtered(&list, &w, &query, &mut index_map.borrow_mut());
                    let showing = index_map.borrow().len();
                    said_and_shown(
                        &status,
                        a11y,
                        &manager_words::added(manager_words::CONTACT, &name, showing),
                        Priority::Normal,
                    );
                }
            }
            r if r == ID_MGR_EDIT => {
                if let Some(sel) = get_selected(&list) {
                    let working_idx = {
                        let map = index_map.borrow();
                        match map.get(sel) {
                            Some(&idx) => idx,
                            None => continue,
                        }
                    };
                    let existing = working.borrow()[working_idx].clone();
                    if let Some(edited) = show_contact_edit(&dialog, Some(&existing), palette, a11y)
                    {
                        let name = edited.name.clone();
                        working.borrow_mut()[working_idx] = edited;
                        *changed.borrow_mut() = true;
                        let query = search_f.get_value();
                        let w = working.borrow();
                        populate_contacts_filtered(&list, &w, &query, &mut index_map.borrow_mut());
                        let showing = index_map.borrow().len();
                        said_and_shown(
                            &status,
                            a11y,
                            &manager_words::updated(manager_words::CONTACT, &name, showing),
                            Priority::Normal,
                        );
                    }
                } else {
                    said_and_shown(
                        &status,
                        a11y,
                        &manager_words::nothing_selected(manager_words::CONTACT, "edit"),
                        Priority::High,
                    );
                }
            }
            r if r == ID_MGR_SYNC => {
                // Taken down on this way out too. Syncing leaves the window,
                // and a return that skipped this would leak one per sync.
                dialog.destroy();
                return ContactManagerAction::SyncRequested;
            }
            _ => break,
        }
    }

    let result = working.borrow().clone();
    // Read first, then taken down: the list belongs to the dialog. wxWidgets
    // does not free one when the Rust value goes, so every contact manager
    // ever opened stayed for the life of the session.
    dialog.destroy();
    if *changed.borrow() {
        ContactManagerAction::Updated(result)
    } else {
        ContactManagerAction::None
    }
}

/// Populate the contact list with optional search filtering.
///
/// Updates `index_map` to map displayed row indices to positions in `contacts`.
fn populate_contacts_filtered(
    list: &ListCtrl,
    contacts: &[ContactEntry],
    query: &str,
    index_map: &mut Vec<usize>,
) {
    list.delete_all_items();
    index_map.clear();
    for (i, c) in contacts.iter().enumerate() {
        if !worth_showing(c, query) {
            continue;
        }
        let display_idx = index_map.len() as i64;
        index_map.push(i);
        let name = if c.favorite {
            format!("★ {}", c.name)
        } else {
            c.name.clone()
        };
        list.insert_item(display_idx, &name, None);
        list.set_item_text_by_column(display_idx, 1, c.primary_email());
        list.set_item_text_by_column(display_idx, 2, c.primary_phone());
        list.set_item_text_by_column(display_idx, 3, &c.company);
    }
}

/// Whether one contact answers what somebody typed in the search box.
///
/// Every address and every number the contact holds, not the first of each.
/// The list shows one address per row, so searching only that one meant that
/// the address you have for somebody, which is the address you would type,
/// found nobody whenever it was one of her others. `ContactEntry::
/// is_written_to_at` in the data layer draws the same line for the same
/// reason.
///
/// An empty box is not a filter, so everybody is worth showing.
///
/// Written apart from the list it fills because a `ListCtrl` needs a window
/// and this decision does not.
fn worth_showing(contact: &ContactEntry, query: &str) -> bool {
    let looking_for = query.trim().to_lowercase();
    if looking_for.is_empty() {
        return true;
    }
    let holds = |value: &str| value.to_lowercase().contains(&looking_for);
    holds(&contact.name)
        || holds(&contact.company)
        || holds(&contact.nickname)
        || contact.emails.iter().any(|e| holds(&e.address))
        || contact.phones.iter().any(|p| holds(&p.number))
}

// ── Contact Edit: Tabbed Dialog ─────────────────────────────────────────────

/// Button IDs for multi-value sub-lists (offset from ID_HIGHEST to avoid clashes)
const ID_ADD_EMAIL: Id = ID_HIGHEST + 400;
const ID_DEL_EMAIL: Id = ID_HIGHEST + 401;
const ID_ADD_PHONE: Id = ID_HIGHEST + 402;
const ID_DEL_PHONE: Id = ID_HIGHEST + 403;
const ID_ADD_ADDR: Id = ID_HIGHEST + 404;
const ID_DEL_ADDR: Id = ID_HIGHEST + 405;
const ID_ADD_CUSTOM: Id = ID_HIGHEST + 406;
const ID_DEL_CUSTOM: Id = ID_HIGHEST + 407;

/// Add a label + TextCtrl row to a FlexGridSizer, parent is a Panel.
fn add_panel_field(parent: &Panel, sizer: &FlexGridSizer, label: &str) -> TextCtrl {
    let lbl = StaticText::builder(parent).with_label(label).build();
    let field = TextCtrl::builder(parent).build();
    set_accessible_name(&field, &name_from_label(label));
    sizer.add(&lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    sizer.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    field
}

/// Open the Add Contact dialog directly (for File > New > Contact).
pub fn show_new_contact_dialog(parent: &Frame, a11y: &Arc<Accessibility>) -> Option<ContactEntry> {
    show_contact_edit(parent, None, theme::current_from_stored_config(), a11y)
}

/// What `show_contact_edit`'s own modal loop still needs after
/// construction: the dialog to run `.show_modal()` on, every field to read
/// back once OK is pressed, and the four lists together with the shared
/// state their own Add/Remove buttons close over.
pub struct ContactEditDialogHandles {
    pub dialog: Dialog,
    pub notebook: Notebook,
    pub basic_panel: Panel,
    pub contact_panel: Panel,
    pub addr_panel: Panel,
    pub notes_panel: Panel,
    pub name_f: TextCtrl,
    pub given_f: TextCtrl,
    pub family_f: TextCtrl,
    pub nick_f: TextCtrl,
    pub company_f: TextCtrl,
    pub dept_f: TextCtrl,
    pub title_f: TextCtrl,
    pub bday_f: TextCtrl,
    pub web_f: TextCtrl,
    pub rel_f: TextCtrl,
    pub avatar_f: TextCtrl,
    pub notes_f: TextCtrl,
    pub fav_check: CheckBox,
    pub email_list: ListCtrl,
    pub phone_list: ListCtrl,
    pub addr_list: ListCtrl,
    pub custom_list: ListCtrl,
    emails_data: Rc<RefCell<Vec<EmailItem>>>,
    phones_data: Rc<RefCell<Vec<PhoneItem>>>,
    addrs_data: Rc<RefCell<Vec<AddressItem>>>,
    custom_data: Rc<RefCell<Vec<CustomFieldItem>>>,
}

/// Build the Add/Edit Contact dialog without showing it.
///
/// Everything `show_contact_edit` used to do up to its own modal loop, split
/// out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
pub fn build_contact_edit_dialog(
    parent: &dyn WxWidget,
    existing: Option<&ContactEntry>,
    palette: Option<theme::Palette>,
    a11y: &Arc<Accessibility>,
) -> ContactEditDialogHandles {
    let title = if existing.is_some() {
        "Edit Contact"
    } else {
        "Add Contact"
    };
    let dlg = Dialog::builder(parent, title)
        .with_size(560, 580)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let root = BoxSizer::builder(Orientation::Vertical).build();

    let notebook = Notebook::builder(&dlg).build();

    // ── Tab 1: Basic Info ────────────────────────────────────────────────
    // Accelerators: N(Name), G(Given name), M(Family name), K(Nickname),
    //   C(Company), D(Department), J(Job Title), B(Birthday), W(Website),
    //   R(Relationship), A(Avatar), F(Favorite)
    let basic_panel = Panel::builder(&notebook).build();
    let basic_sizer = BoxSizer::builder(Orientation::Vertical).build();
    let basic_fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    basic_fields.add_growable_col(1, 1);

    let name_f = add_panel_field(&basic_panel, &basic_fields, "&Name:");
    // The two parts, shown so a guess at them can be corrected before it goes
    // anywhere. Nothing splits the name again after this.
    let given_f = add_panel_field(&basic_panel, &basic_fields, "&Given name:");
    let family_f = add_panel_field(&basic_panel, &basic_fields, "Fa&mily name:");
    let nick_f = add_panel_field(&basic_panel, &basic_fields, "Nic&kname:");
    let company_f = add_panel_field(&basic_panel, &basic_fields, "&Company:");
    let dept_f = add_panel_field(&basic_panel, &basic_fields, "&Department:");
    let title_f = add_panel_field(&basic_panel, &basic_fields, "&Job Title:");
    let bday_f = add_panel_field(&basic_panel, &basic_fields, "&Birthday:");
    let web_f = add_panel_field(&basic_panel, &basic_fields, "&Website:");
    let rel_f = add_panel_field(&basic_panel, &basic_fields, "&Relationship:");
    let avatar_f = add_panel_field(&basic_panel, &basic_fields, "&Avatar URL:");

    let fav_spacer = StaticText::builder(&basic_panel).with_label("").build();
    let fav_check = CheckBox::builder(&basic_panel)
        .with_label("&Favorite")
        .build();
    basic_fields.add(&fav_spacer, 0, SizerFlag::All, 4);
    basic_fields.add(&fav_check, 0, SizerFlag::All, 4);

    basic_sizer.add_sizer(&basic_fields, 1, SizerFlag::Expand | SizerFlag::All, 8);
    basic_panel.set_sizer(basic_sizer, true);
    notebook.add_page(&basic_panel, "Basic Info", true, None);

    // ── Tab 2: Email & Phone ─────────────────────────────────────────────
    // Accelerators: A(Add Email), R(Remove Email), P(Add Phone), V(Remove Phone)
    let contact_panel = Panel::builder(&notebook).build();
    let contact_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Email section
    let email_label = StaticText::builder(&contact_panel)
        .with_label("Email Addresses:")
        .build();
    contact_sizer.add(
        &email_label,
        0,
        SizerFlag::Left | SizerFlag::Top | SizerFlag::Right,
        8,
    );
    let email_list = ListCtrl::builder(&contact_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&email_list, "Email addresses");
    email_list.insert_column(0, "Type", ListColumnFormat::Left, 100);
    email_list.insert_column(1, "Address", ListColumnFormat::Left, 300);
    contact_sizer.add(&email_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let email_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_email_btn = Button::builder(&contact_panel)
        .with_label("&Add Email...")
        .with_id(ID_ADD_EMAIL)
        .build();
    let del_email_btn = Button::builder(&contact_panel)
        .with_label("&Remove Email")
        .with_id(ID_DEL_EMAIL)
        .build();
    email_btn_row.add(&add_email_btn, 0, SizerFlag::All, 4);
    email_btn_row.add(&del_email_btn, 0, SizerFlag::All, 4);
    contact_sizer.add_sizer(&email_btn_row, 0, SizerFlag::Left, 4);

    // Phone section
    let phone_label = StaticText::builder(&contact_panel)
        .with_label("Phone Numbers:")
        .build();
    contact_sizer.add(
        &phone_label,
        0,
        SizerFlag::Left | SizerFlag::Top | SizerFlag::Right,
        8,
    );
    let phone_list = ListCtrl::builder(&contact_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&phone_list, "Phone numbers");
    phone_list.insert_column(0, "Type", ListColumnFormat::Left, 100);
    phone_list.insert_column(1, "Number", ListColumnFormat::Left, 300);
    contact_sizer.add(&phone_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let phone_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_phone_btn = Button::builder(&contact_panel)
        .with_label("Add &Phone...")
        .with_id(ID_ADD_PHONE)
        .build();
    let del_phone_btn = Button::builder(&contact_panel)
        .with_label("Remo&ve Phone")
        .with_id(ID_DEL_PHONE)
        .build();
    phone_btn_row.add(&add_phone_btn, 0, SizerFlag::All, 4);
    phone_btn_row.add(&del_phone_btn, 0, SizerFlag::All, 4);
    contact_sizer.add_sizer(&phone_btn_row, 0, SizerFlag::Left, 4);

    contact_panel.set_sizer(contact_sizer, true);
    notebook.add_page(&contact_panel, "Email && Phone", false, None);

    // ── Tab 3: Addresses ─────────────────────────────────────────────────
    // Accelerators: A(Add Address), R(Remove Address)
    let addr_panel = Panel::builder(&notebook).build();
    let addr_sizer = BoxSizer::builder(Orientation::Vertical).build();
    let addr_label = StaticText::builder(&addr_panel)
        .with_label("Physical Addresses:")
        .build();
    addr_sizer.add(
        &addr_label,
        0,
        SizerFlag::Left | SizerFlag::Top | SizerFlag::Right,
        8,
    );
    let addr_list = ListCtrl::builder(&addr_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&addr_list, "Addresses");
    addr_list.insert_column(0, "Type", ListColumnFormat::Left, 80);
    addr_list.insert_column(1, "Street", ListColumnFormat::Left, 150);
    addr_list.insert_column(2, "City", ListColumnFormat::Left, 100);
    addr_list.insert_column(3, "State/Zip", ListColumnFormat::Left, 100);
    addr_list.insert_column(4, "Country", ListColumnFormat::Left, 80);
    addr_sizer.add(&addr_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let addr_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_addr_btn = Button::builder(&addr_panel)
        .with_label("&Add Address...")
        .with_id(ID_ADD_ADDR)
        .build();
    let del_addr_btn = Button::builder(&addr_panel)
        .with_label("&Remove Address")
        .with_id(ID_DEL_ADDR)
        .build();
    addr_btn_row.add(&add_addr_btn, 0, SizerFlag::All, 4);
    addr_btn_row.add(&del_addr_btn, 0, SizerFlag::All, 4);
    addr_sizer.add_sizer(&addr_btn_row, 0, SizerFlag::Left, 4);
    addr_panel.set_sizer(addr_sizer, true);
    notebook.add_page(&addr_panel, "Addresses", false, None);

    // ── Tab 4: Notes & Custom ────────────────────────────────────────────
    // Accelerators: N(Notes), A(Add Field), R(Remove Field)
    let notes_panel = Panel::builder(&notebook).build();
    let notes_sizer = BoxSizer::builder(Orientation::Vertical).build();
    let notes_label = StaticText::builder(&notes_panel)
        .with_label("&Notes, in Markdown:")
        .build();
    notes_sizer.add(&notes_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let notes_f = TextCtrl::builder(&notes_panel)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();
    set_accessible_name_and_description(
        &notes_f,
        "Notes",
        "Markdown headings and lists are read back when this is read aloud",
    );
    notes_sizer.add(&notes_f, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let custom_label = StaticText::builder(&notes_panel)
        .with_label("Custom Fields:")
        .build();
    notes_sizer.add(&custom_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let custom_list = ListCtrl::builder(&notes_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&custom_list, "Custom fields");
    custom_list.insert_column(0, "Label", ListColumnFormat::Left, 150);
    custom_list.insert_column(1, "Value", ListColumnFormat::Left, 300);
    notes_sizer.add(&custom_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let custom_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_custom_btn = Button::builder(&notes_panel)
        .with_label("&Add Field...")
        .with_id(ID_ADD_CUSTOM)
        .build();
    let del_custom_btn = Button::builder(&notes_panel)
        .with_label("&Remove Field")
        .with_id(ID_DEL_CUSTOM)
        .build();
    custom_btn_row.add(&add_custom_btn, 0, SizerFlag::All, 4);
    custom_btn_row.add(&del_custom_btn, 0, SizerFlag::All, 4);
    notes_sizer.add_sizer(&custom_btn_row, 0, SizerFlag::Left, 4);
    notes_panel.set_sizer(notes_sizer, true);
    notebook.add_page(&notes_panel, "Notes && Custom", false, None);

    root.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // Where OK says why it refused to save, on screen and, through
    // `said_and_shown` below, to the accessibility announcement queue.
    // Empty until then.
    let problem_line = StaticText::builder(&dlg).with_label("").build();
    root.add(
        &problem_line,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        8,
    );

    // ── OK / Cancel ──────────────────────────────────────────────────────
    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    root.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(root, true);

    // ── Populate from existing contact ───────────────────────────────────
    // Use thread-safe cells for mutable state shared with button callbacks
    let emails_data = Rc::new(RefCell::new(Vec::<EmailItem>::new()));
    let phones_data = Rc::new(RefCell::new(Vec::<PhoneItem>::new()));
    let addrs_data = Rc::new(RefCell::new(Vec::<AddressItem>::new()));
    let custom_data = Rc::new(RefCell::new(Vec::<CustomFieldItem>::new()));

    if let Some(c) = existing {
        name_f.set_value(&c.name);
        given_f.set_value(&c.given_name);
        family_f.set_value(&c.family_name);
        nick_f.set_value(&c.nickname);
        company_f.set_value(&c.company);
        dept_f.set_value(&c.department);
        title_f.set_value(&c.job_title);
        bday_f.set_value(&c.birthday);
        web_f.set_value(&c.website);
        rel_f.set_value(&c.relationship);
        avatar_f.set_value(&c.avatar_url);
        notes_f.set_value(&c.notes);
        fav_check.set_value(c.favorite);

        *emails_data.borrow_mut() = c.emails.clone();
        *phones_data.borrow_mut() = c.phones.clone();
        *addrs_data.borrow_mut() = c.addresses.clone();
        *custom_data.borrow_mut() = c.custom_fields.clone();
    }

    refresh_email_list(&email_list, &emails_data.borrow());
    refresh_phone_list(&phone_list, &phones_data.borrow());
    refresh_addr_list(&addr_list, &addrs_data.borrow());
    refresh_custom_list(&custom_list, &custom_data.borrow());

    // ── Button handlers (use end_modal with custom IDs) ──────────────────
    // OK checks the one thing this dialog requires before it closes at all:
    // intercepting the click (and Enter, which reaches a default button the
    // same way) ahead of wxWidgets' own default handling for `ID_OK` means
    // not calling `end_modal` is the whole of refusing to close, the same
    // reasoning `wx_item_form.rs`'s own Save handler is built on. This used
    // to be checked after the dialog had already closed, by re-showing it
    // with nothing said and nothing pumped in between: exactly the gap this
    // file's own `delete_selected_contact` was fixed for elsewhere, just
    // never caught here.
    ok.on_click({
        let d = dlg;
        let a11y = Arc::clone(a11y);
        move |event| {
            // Consuming the click is what makes the refusal below stick.
            // Left unconsumed it carries on to wxWidgets' own handler for an
            // affirmative button, which closes the dialog regardless of what
            // this decided. See `wx_item_form.rs`'s module doc comment.
            event.event.skip(false);
            if name_f.get_value().trim().is_empty() {
                said_and_shown(
                    &problem_line,
                    &a11y,
                    "Name is needed before this can be saved.",
                    Priority::High,
                );
                name_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });
    add_email_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD_EMAIL);
        }
    });
    del_email_btn.on_click({
        let held = emails_data.clone();
        let a11y = Arc::clone(a11y);
        move |event| {
            event.event.skip(false);
            remove_from_a_contact_list(
                &email_list,
                &held,
                &problem_line,
                &a11y,
                "email",
                |item: &EmailItem| item.address.clone(),
                refresh_email_list,
            );
        }
    });
    add_phone_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD_PHONE);
        }
    });
    del_phone_btn.on_click({
        let held = phones_data.clone();
        let a11y = Arc::clone(a11y);
        move |event| {
            event.event.skip(false);
            remove_from_a_contact_list(
                &phone_list,
                &held,
                &problem_line,
                &a11y,
                "phone number",
                |item: &PhoneItem| item.number.clone(),
                refresh_phone_list,
            );
        }
    });
    add_addr_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD_ADDR);
        }
    });
    del_addr_btn.on_click({
        let held = addrs_data.clone();
        let a11y = Arc::clone(a11y);
        move |event| {
            event.event.skip(false);
            remove_from_a_contact_list(
                &addr_list,
                &held,
                &problem_line,
                &a11y,
                "address",
                |item: &AddressItem| item.street.clone(),
                refresh_addr_list,
            );
        }
    });
    add_custom_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD_CUSTOM);
        }
    });
    del_custom_btn.on_click({
        let held = custom_data.clone();
        let a11y = Arc::clone(a11y);
        move |event| {
            event.event.skip(false);
            remove_from_a_contact_list(
                &custom_list,
                &held,
                &problem_line,
                &a11y,
                "custom field",
                |item: &CustomFieldItem| item.label.clone(),
                refresh_custom_list,
            );
        }
    });

    // Painted last, once every field, both panels and all four lists are
    // built and populated. The favourite CheckBox is left to Windows, the
    // same as every checkbox elsewhere in this round. `None` means high
    // contrast is on, or the system is set up in a way this application
    // should not paint over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&notebook, palette.main_surface());
        for panel in [&basic_panel, &contact_panel, &addr_panel, &notes_panel] {
            theme::paint(panel, palette.main_surface());
        }
        for field in [
            &name_f, &given_f, &family_f, &nick_f, &company_f, &dept_f, &title_f, &bday_f, &web_f,
            &rel_f, &avatar_f, &notes_f,
        ] {
            theme::paint(field, palette.main_surface());
        }
        for list in [&email_list, &phone_list, &addr_list, &custom_list] {
            theme::paint(list, palette.main_surface());
        }
    }

    ContactEditDialogHandles {
        dialog: dlg,
        notebook,
        basic_panel,
        contact_panel,
        addr_panel,
        notes_panel,
        name_f,
        given_f,
        family_f,
        nick_f,
        company_f,
        dept_f,
        title_f,
        bday_f,
        web_f,
        rel_f,
        avatar_f,
        notes_f,
        fav_check,
        email_list,
        phone_list,
        addr_list,
        custom_list,
        emails_data,
        phones_data,
        addrs_data,
        custom_data,
    }
}

fn show_contact_edit(
    parent: &dyn WxWidget,
    existing: Option<&ContactEntry>,
    palette: Option<theme::Palette>,
    a11y: &Arc<Accessibility>,
) -> Option<ContactEntry> {
    let ContactEditDialogHandles {
        dialog: dlg,
        name_f,
        given_f,
        family_f,
        nick_f,
        company_f,
        dept_f,
        title_f,
        bday_f,
        web_f,
        rel_f,
        avatar_f,
        notes_f,
        fav_check,
        email_list,
        phone_list,
        addr_list,
        custom_list,
        emails_data,
        phones_data,
        addrs_data,
        custom_data,
        ..
    } = build_contact_edit_dialog(parent, existing, palette, a11y);

    // ── Modal loop (handle sub-list actions before OK/Cancel) ────────────
    loop {
        match dlg.show_modal() {
            r if r == ID_ADD_EMAIL => {
                if let Some(item) = show_email_sub_dialog(&dlg, None, palette) {
                    emails_data.borrow_mut().push(item);
                    refresh_email_list(&email_list, &emails_data.borrow());
                }
            }
            r if r == ID_ADD_PHONE => {
                if let Some(item) = show_phone_sub_dialog(&dlg, None, palette) {
                    phones_data.borrow_mut().push(item);
                    refresh_phone_list(&phone_list, &phones_data.borrow());
                }
            }
            r if r == ID_ADD_ADDR => {
                if let Some(item) = show_address_sub_dialog(&dlg, None, palette) {
                    addrs_data.borrow_mut().push(item);
                    refresh_addr_list(&addr_list, &addrs_data.borrow());
                }
            }
            r if r == ID_ADD_CUSTOM => {
                if let Some(item) = show_custom_field_sub_dialog(&dlg, None, palette) {
                    custom_data.borrow_mut().push(item);
                    refresh_custom_list(&custom_list, &custom_data.borrow());
                }
            }
            r if r == ID_OK => {
                // Whatever this loop just closed on already held together:
                // OK's own handler, inside the dialog `show_modal` just
                // returned from, refuses to end the modal at all while the
                // name is still empty, so there is nothing left to check on
                // this side of it.
                let contact_name = name_f.get_value();
                let answer = ContactEntry {
                    id: existing
                        .map(|c| c.id.clone())
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                    name: contact_name,
                    given_name: given_f.get_value(),
                    family_name: family_f.get_value(),
                    nickname: nick_f.get_value(),
                    company: company_f.get_value(),
                    department: dept_f.get_value(),
                    job_title: title_f.get_value(),
                    birthday: bday_f.get_value(),
                    website: web_f.get_value(),
                    relationship: rel_f.get_value(),
                    avatar_url: avatar_f.get_value(),
                    notes: notes_f.get_value(),
                    favorite: fav_check.get_value(),
                    emails: emails_data.borrow().clone(),
                    phones: phones_data.borrow().clone(),
                    addresses: addrs_data.borrow().clone(),
                    custom_fields: custom_data.borrow().clone(),
                };
                // Read first, then taken down. wxWidgets does not free a
                // dialog when the Rust value goes, so every contact editor
                // ever opened stayed for the life of the session, and this
                // one holds a notebook with four panels and four lists.
                dlg.destroy();
                return Some(answer);
            }
            // Cancel or close.
            _ => {
                dlg.destroy();
                return None;
            }
        }
    }
}

// ── List refresh helpers ─────────────────────────────────────────────────────

/// Take the selected row off one of the contact editor's own lists.
///
/// Done here, with the window still open and still pumping, rather than by
/// closing the dialog and reopening it. All four Remove buttons used to close
/// it: the loop then removed the row in silence, with nothing announced and
/// no answer at all when nothing was selected, and a repaint that left
/// nothing selected meant a second press did nothing and said nothing about
/// why. That is the same shape this file's own `delete_selected` was fixed
/// for, and these four were never brought along.
fn remove_from_a_contact_list<T: Clone>(
    list: &ListCtrl,
    held: &Rc<RefCell<Vec<T>>>,
    problem_line: &StaticText,
    a11y: &Accessibility,
    what: &str,
    name_of: impl Fn(&T) -> String,
    repaint: impl Fn(&ListCtrl, &[T]),
) {
    let Some(at) = get_selected(list) else {
        said_and_shown(
            problem_line,
            a11y,
            &manager_words::nothing_selected(what, "remove"),
            Priority::High,
        );
        return;
    };
    let name = held.borrow().get(at).map(&name_of).unwrap_or_default();
    held.borrow_mut().remove(at);
    let left = held.borrow().len();
    repaint(list, &held.borrow());
    // Back on the row that moved up, so removing several in a row does not
    // mean finding the list again each time.
    land_the_row_cursor(list, the_row_to_select_after_removing(at, left));
    said_and_shown(
        problem_line,
        a11y,
        &manager_words::deleted(what, &name, left),
        Priority::Normal,
    );
}

fn refresh_email_list(list: &ListCtrl, items: &[EmailItem]) {
    list.delete_all_items();
    for (i, e) in items.iter().enumerate() {
        list.insert_item(i as i64, &e.label, None);
        list.set_item_text_by_column(i as i64, 1, &e.address);
    }
}

fn refresh_phone_list(list: &ListCtrl, items: &[PhoneItem]) {
    list.delete_all_items();
    for (i, p) in items.iter().enumerate() {
        list.insert_item(i as i64, &p.label, None);
        list.set_item_text_by_column(i as i64, 1, &p.number);
    }
}

fn refresh_addr_list(list: &ListCtrl, items: &[AddressItem]) {
    list.delete_all_items();
    for (i, a) in items.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &a.label, None);
        list.set_item_text_by_column(idx, 1, &a.street);
        list.set_item_text_by_column(idx, 2, &a.city);
        list.set_item_text_by_column(idx, 3, format!("{} {}", a.state, a.zip).trim());
        list.set_item_text_by_column(idx, 4, &a.country);
    }
}

fn refresh_custom_list(list: &ListCtrl, items: &[CustomFieldItem]) {
    list.delete_all_items();
    for (i, f) in items.iter().enumerate() {
        list.insert_item(i as i64, &f.label, None);
        list.set_item_text_by_column(i as i64, 1, &f.value);
    }
}

// ── Sub-dialogs for adding multi-value entries ───────────────────────────────

/// Build the Add Email Address dialog without showing it.
///
/// Everything `show_email_sub_dialog` used to do up to its own
/// `.show_modal()` call, split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
///
/// Returns the type choice and the address field alongside the dialog, the
/// same way `show_email_sub_dialog` still needs them after a real
/// `.show_modal()`.
/// Say what a sub-dialog needs before it can close, and keep it open.
///
/// Through a message box rather than an announcement: these four little
/// windows are built without an accessibility handle, and a message box is
/// read out by a screen reader on its own. They used to close and let the
/// caller discover the empty field, which then returned nothing at all, so
/// filling in a custom field's value and leaving its label blank destroyed
/// both with no word said.
///
/// `titled` is the caption, because a refusal is not always a refusal to add:
/// a manager window refuses to close, and a box captioned "Not added" over
/// that sentence is a caption a screen reader reads out before the sentence
/// and which contradicts it.
fn a_sub_dialog_needs(parent: &Dialog, titled: &str, said: &str) {
    let box_ = MessageDialog::builder(parent, said, titled)
        .with_style(MessageDialogStyle::OK | MessageDialogStyle::IconInformation)
        .build();
    box_.show_modal();
}

pub fn build_email_sub_dialog(
    parent: &Dialog,
    palette: Option<theme::Palette>,
) -> (Dialog, Choice, TextCtrl) {
    let dlg = Dialog::builder(parent, "Add Email Address")
        .with_size(400, 200)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Accelerators are first letters, no conflicts: T(Type), A(Address)
    let type_lbl = StaticText::builder(&dlg).with_label("&Type:").build();
    let type_choices: Vec<String> = EMAIL_LABELS.iter().map(|s| s.to_string()).collect();
    let type_choice = Choice::builder(&dlg).with_choices(type_choices).build();
    set_accessible_name(&type_choice, "Email type");
    type_choice.set_selection(0);
    fields.add(
        &type_lbl,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let addr_f = add_field(&dlg, &fields, "&Address:");
    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick; see
            // `wx_item_form.rs`'s module doc comment.
            event.event.skip(false);
            if addr_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(
                    &d,
                    "Not added",
                    "An email address is needed before this can be added.",
                );
                addr_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last. The type Choice is left to Windows, matching every other
    // Choice this round paints around. `None` means high contrast is on, or
    // the system is set up in a way this application should not paint over,
    // so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&addr_f, palette.main_surface());
    }

    (dlg, type_choice, addr_f)
}

fn show_email_sub_dialog(
    parent: &Dialog,
    _existing: Option<&EmailItem>,
    palette: Option<theme::Palette>,
) -> Option<EmailItem> {
    let (dlg, type_choice, addr_f) = build_email_sub_dialog(parent, palette);

    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        let addr = addr_f.get_value();
        // Whatever comes back already held together: OK's own handler, in
        // the window that has just closed, refuses to close at all while a
        // needed box is empty. The check that used to be here ran after the
        // window was gone and could only throw away everything typed.
        Some(EmailItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            address: addr,
        })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

/// Build the Add Phone Number dialog without showing it.
///
/// Everything `show_phone_sub_dialog` used to do up to its own
/// `.show_modal()` call, split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
///
/// Returns the type choice and the number field alongside the dialog, the
/// same way `show_phone_sub_dialog` still needs them after a real
/// `.show_modal()`.
pub fn build_phone_sub_dialog(
    parent: &Dialog,
    palette: Option<theme::Palette>,
) -> (Dialog, Choice, TextCtrl) {
    let dlg = Dialog::builder(parent, "Add Phone Number")
        .with_size(400, 200)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Accelerators are first letters, no conflicts: T(Type), N(Number)
    let type_lbl = StaticText::builder(&dlg).with_label("&Type:").build();
    let type_choices: Vec<String> = PHONE_LABELS.iter().map(|s| s.to_string()).collect();
    let type_choice = Choice::builder(&dlg).with_choices(type_choices).build();
    set_accessible_name(&type_choice, "Phone type");
    type_choice.set_selection(0);
    fields.add(
        &type_lbl,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let num_f = add_field(&dlg, &fields, "&Number:");
    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick; see
            // `wx_item_form.rs`'s module doc comment.
            event.event.skip(false);
            if num_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(
                    &d,
                    "Not added",
                    "A phone number is needed before this can be added.",
                );
                num_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last. The type Choice is left to Windows, matching every other
    // Choice this round paints around. `None` means high contrast is on, or
    // the system is set up in a way this application should not paint over,
    // so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&num_f, palette.main_surface());
    }

    (dlg, type_choice, num_f)
}

fn show_phone_sub_dialog(
    parent: &Dialog,
    _existing: Option<&PhoneItem>,
    palette: Option<theme::Palette>,
) -> Option<PhoneItem> {
    let (dlg, type_choice, num_f) = build_phone_sub_dialog(parent, palette);

    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        let num = num_f.get_value();
        // Whatever comes back already held together: OK's own handler, in
        // the window that has just closed, refuses to close at all while a
        // needed box is empty. The check that used to be here ran after the
        // window was gone and could only throw away everything typed.
        Some(PhoneItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            number: num,
        })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

/// What `show_address_sub_dialog` still needs after construction: the
/// dialog to run `.show_modal()` on, and every field and Choice to read
/// back once OK is pressed.
pub struct AddressSubDialogWidgets {
    pub dialog: Dialog,
    pub country_choice: Choice,
    pub type_choice: Choice,
    pub street_f: TextCtrl,
    pub city_f: TextCtrl,
    pub region_f: TextCtrl,
    pub code_f: TextCtrl,
}

/// Build the Add Address dialog without showing it.
///
/// Everything `show_address_sub_dialog` used to do up to its own
/// `.show_modal()` call, split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
pub fn build_address_sub_dialog(
    parent: &Dialog,
    palette: Option<theme::Palette>,
) -> AddressSubDialogWidgets {
    let dlg = Dialog::builder(parent, "Add Address")
        .with_size(440, 380)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // ── Country dropdown FIRST: drives field labels ─────────────────
    // Accelerators: C(Country), T(Type), S(Street), I(City),
    //   region and code labels set dynamically by get_address_field_labels()
    let country_lbl = StaticText::builder(&dlg).with_label("&Country:").build();
    let country_choices: Vec<String> = COUNTRIES.iter().map(|s| s.to_string()).collect();
    let country_choice = Choice::builder(&dlg).with_choices(country_choices).build();
    set_accessible_name(&country_choice, "Country");
    // Default to system locale country
    let default_country = get_default_country();
    select_choice_by_string(&country_choice, default_country);
    fields.add(
        &country_lbl,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&country_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // ── Address type ─────────────────────────────────────────────────
    let type_lbl = StaticText::builder(&dlg).with_label("&Type:").build();
    let type_choices: Vec<String> = ADDRESS_LABELS.iter().map(|s| s.to_string()).collect();
    let type_choice = Choice::builder(&dlg).with_choices(type_choices).build();
    set_accessible_name(&type_choice, "Address type");
    type_choice.set_selection(0);
    fields.add(
        &type_lbl,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // ── Address fields ───────────────────────────────────────────────
    let street_f = add_field(&dlg, &fields, "&Street:");
    let city_f = add_field(&dlg, &fields, "C&ity:");

    // Region and code labels are dynamic: set based on selected country
    let (initial_region_label, initial_code_label) = get_address_field_labels(default_country);

    let region_lbl = StaticText::builder(&dlg)
        .with_label(initial_region_label)
        .build();
    let region_f = TextCtrl::builder(&dlg).build();
    set_accessible_name(&region_f, "State or region");
    fields.add(
        &region_lbl,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&region_f, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let code_lbl = StaticText::builder(&dlg)
        .with_label(initial_code_label)
        .build();
    let code_f = TextCtrl::builder(&dlg).build();
    set_accessible_name(&code_f, "Postal code");
    fields.add(
        &code_lbl,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&code_f, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // ── Country change handler: update region/code labels ───────────
    country_choice.on_selection_changed({
        let rl = region_lbl;
        let cl = code_lbl;
        move |event| {
            if let Some(country) = event.get_string() {
                let (region_text, code_text) = get_address_field_labels(&country);
                rl.set_label(region_text);
                cl.set_label(code_text);
            }
        }
    });

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick; see
            // `wx_item_form.rs`'s module doc comment.
            event.event.skip(false);
            // Street or city, which is the same rule the caller used to
            // apply after this window had already closed, throwing away the
            // state, postcode and country somebody had filled in with it.
            if street_f.get_value().trim().is_empty() && city_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(
                    &d,
                    "Not added",
                    "A street or a town is needed before this can be added.",
                );
                street_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last. Both Choice controls are left to Windows, matching every
    // other Choice this round paints around. `None` means high contrast is
    // on, or the system is set up in a way this application should not
    // paint over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        for field in [&street_f, &city_f, &region_f, &code_f] {
            theme::paint(field, palette.main_surface());
        }
    }

    AddressSubDialogWidgets {
        dialog: dlg,
        country_choice,
        type_choice,
        street_f,
        city_f,
        region_f,
        code_f,
    }
}

fn show_address_sub_dialog(
    parent: &Dialog,
    _existing: Option<&AddressItem>,
    palette: Option<theme::Palette>,
) -> Option<AddressItem> {
    let AddressSubDialogWidgets {
        dialog: dlg,
        country_choice,
        type_choice,
        street_f,
        city_f,
        region_f,
        code_f,
    } = build_address_sub_dialog(parent, palette);

    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        let street = street_f.get_value();
        let city = city_f.get_value();
        // Whatever comes back already held together: OK's own handler, in
        // the window that has just closed, refuses to close at all while a
        // needed box is empty. The check that used to be here ran after the
        // window was gone and could only throw away everything typed.
        Some(AddressItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            street,
            city,
            state: region_f.get_value(),
            zip: code_f.get_value(),
            country: get_choice_string(&country_choice)
                .unwrap_or_else(|| get_default_country().to_string()),
        })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

/// Build the Add Custom Field dialog without showing it.
///
/// Everything `show_custom_field_sub_dialog` used to do up to its own
/// `.show_modal()` call, split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
///
/// Returns the label and value fields alongside the dialog, the same way
/// `show_custom_field_sub_dialog` still needs them after a real
/// `.show_modal()`.
pub fn build_custom_field_sub_dialog(
    parent: &Dialog,
    palette: Option<theme::Palette>,
) -> (Dialog, TextCtrl, TextCtrl) {
    let dlg = Dialog::builder(parent, "Add Custom Field")
        .with_size(400, 200)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Accelerators are first letters, no conflicts: L(Label), V(Value)
    let label_f = add_field(&dlg, &fields, "&Label:");
    let value_f = add_field(&dlg, &fields, "&Value:");
    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick; see
            // `wx_item_form.rs`'s module doc comment.
            event.event.skip(false);
            if label_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(
                    &d,
                    "Not added",
                    "A name for the field is needed before this can be added.",
                );
                label_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last. `None` means high contrast is on, or the system is set
    // up in a way this application should not paint over, so nothing is set
    // here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&label_f, palette.main_surface());
        theme::paint(&value_f, palette.main_surface());
    }

    (dlg, label_f, value_f)
}

fn show_custom_field_sub_dialog(
    parent: &Dialog,
    _existing: Option<&CustomFieldItem>,
    palette: Option<theme::Palette>,
) -> Option<CustomFieldItem> {
    let (dlg, label_f, value_f) = build_custom_field_sub_dialog(parent, palette);

    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        let label = label_f.get_value();
        let value = value_f.get_value();
        // Whatever comes back already held together: OK's own handler, in
        // the window that has just closed, refuses to close at all while a
        // needed box is empty. The check that used to be here ran after the
        // window was gone and could only throw away everything typed.
        Some(CustomFieldItem { label, value })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

// ══════════════════════════════════════════════════════════════════════════════
// Filter Manager
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct FilterRule {
    pub id: String,
    pub name: String,
    pub field: String,
    pub match_type: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub action_type: String,
    pub action_value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum FilterManagerAction {
    None,
    Updated(Vec<FilterRule>),
}

pub fn show_filter_manager_dialog(
    parent: &Frame,
    rules: &[FilterRule],
    a11y: &Arc<Accessibility>,
) -> FilterManagerAction {
    // Read once and reused for the manager shell and every Add/Edit dialog
    // it opens, rather than a second, independent disk read per dialog (see
    // `theme::current_from_stored_config`'s own doc comment for why that
    // matters).
    let palette = theme::current_from_stored_config();
    let (dialog, sizer, list, status) =
        make_shell(parent, "Filter Manager", "Filters", 650, 450, palette);

    list.insert_column(0, "Name", ListColumnFormat::Left, 130);
    list.insert_column(1, "Condition", ListColumnFormat::Left, 220);
    list.insert_column(2, "Action", ListColumnFormat::Left, 150);
    list.insert_column(3, "Status", ListColumnFormat::Centre, 70);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = rules.to_vec();
    let changed = run_manager_loop(
        ManagerChrome {
            dialog: &dialog,
            main_sizer: &sizer,
            list: &list,
            status_text: &status,
            a11y: a11y.clone(),
        },
        manager_words::FILTER,
        &mut working,
        populate_filters,
        |d, existing| show_filter_edit(d, existing, palette),
        |r| r.name.clone(),
        nothing_stops_this_closing,
    );

    if changed {
        FilterManagerAction::Updated(working)
    } else {
        FilterManagerAction::None
    }
}

/// What a rule can do, as it is stored and as it is read out.
///
/// The stored names used to be shown as they are written down, so the Action
/// list read out "mark_as_read" and "move_to_folder": machine names, spoken
/// to somebody choosing between them.
///
/// Moving to a folder is built now, and reaches the server: the sync files
/// the message where the rule says and brings this computer into line
/// afterwards. A folder the account does not have is passed over with a word
/// in the log rather than failing the whole sync, so a rule naming a folder
/// somebody has since renamed does not stop their mail arriving.
const RULE_ACTIONS: &[(&str, &str)] = &[
    ("mark_as_read", "Mark as read"),
    ("mark_as_unread", "Mark as unread"),
    ("star", "Flag it"),
    ("delete", "Delete it"),
    ("move_to_folder", "Move to a folder"),
    ("add_tag", "Add a label"),
];

/// The words for a stored action, or the stored name when it is not one of
/// these, so a rule written by a later version is shown rather than blanked.
fn shown_action(stored: &str) -> &str {
    RULE_ACTIONS
        .iter()
        .find(|(name, _)| *name == stored)
        .map_or(stored, |(_, shown)| *shown)
}

/// The stored name for what a person picked.
fn stored_action(shown: &str) -> String {
    RULE_ACTIONS
        .iter()
        .find(|(_, offered)| *offered == shown)
        .map_or_else(|| shown.to_string(), |(name, _)| (*name).to_string())
}

/// Whether the Pattern box has anything to ask for, given the words showing in
/// the Match Type list.
///
/// Asked of the words rather than of a stored name because the list is the
/// only thing that knows what has been picked, and answered by
/// [`a_way_of_matching_compares_against_nothing`] rather than by a comparison
/// written here. Four ways of matching read no pattern, and a second copy of
/// which four is how the two come to disagree; the rule editor plan 02-05 adds
/// would have been the third place to hold that list.
///
/// Words nothing is called keep the box. Either nothing has been chosen yet,
/// or the rule was written by a later version, and hiding a control over a
/// question this build could not answer is the worse of the two mistakes.
fn the_pattern_box_asks_for_something(match_words: &str) -> bool {
    the_way_of_matching_those_words_name(match_words)
        .is_none_or(|stored| !a_way_of_matching_compares_against_nothing(stored))
}

/// What a rule stores as its pattern, given how it matches and what is in the
/// box.
///
/// Nothing, when the way of matching reads no pattern. The box is disabled in
/// that case and cannot be typed into, but it can still be holding what
/// somebody typed before they changed the Match Type, and storing that leaves
/// a rule carrying a pattern nothing compares against. Plan 02-06's sentence
/// builder would then read it out as though it meant something.
///
/// Asked of the words rather than of a stored name, because the dialog reads
/// words off the list and the conversion back has already gone wrong once in
/// this file's history by being done in two places.
fn the_pattern_to_store(match_words: &str, typed: &str) -> String {
    if the_pattern_box_asks_for_something(match_words) {
        typed.to_string()
    } else {
        String::new()
    }
}

/// Every field a rule may name, in the words somebody hears, ready for a
/// `Choice`.
///
/// One builder read by both rule dialogs rather than the same four lines
/// written twice. That is the answer `RULE_ACTIONS` already gives for actions,
/// and the reason is the one `WHAT_EACH_FIELD_IS_CALLED`'s own doc gives: a
/// second copy of a list is how two lists come to disagree. A second reader of
/// one source is a different thing from a second source.
fn the_words_for_every_field() -> Vec<String> {
    A_FIELD_A_RULE_MAY_NAME
        .iter()
        .filter_map(|field| the_words_for_a_field(field))
        .map(str::to_string)
        .collect()
}

/// Every way a rule may match, in the words somebody hears, ready for a
/// `Choice`.
fn the_words_for_every_way_of_matching() -> Vec<String> {
    A_WAY_A_RULE_MAY_MATCH
        .iter()
        .filter_map(|way| the_words_for_a_way_of_matching(way))
        .map(str::to_string)
        .collect()
}

/// The field and the way of matching a brand new condition opens on.
///
/// Written down rather than left to whichever entry the constants happen to
/// list first. A `Choice` with nothing selected reads out as an unfilled combo
/// box, and pressing OK on one would store the empty string as the field, so a
/// new condition has to open on something. Which something is a decision about
/// what somebody is most likely about to write, and reordering
/// [`A_FIELD_A_RULE_MAY_NAME`] for an unrelated reason should not silently
/// change it.
///
/// A subject that contains a word: that is what the search box asks, so it is
/// the condition a rule editor is most often opened to write.
const WHAT_A_NEW_CONDITION_ASKS_FIRST: (&str, &str) = ("subject", "contains");

/// What a condition still needs before it can be saved, if anything.
///
/// Only the pattern. The two lists cannot be left unanswered, because the
/// dialog opens on [`WHAT_A_NEW_CONDITION_ASKS_FIRST`].
///
/// Asked of the words showing in the Match Type list, the same way
/// [`the_pattern_to_store`] is, and answered through
/// [`the_pattern_box_asks_for_something`] rather than by a second comparison
/// written here.
///
/// A way of matching that compares against nothing saves with an empty pattern
/// quite happily. Refusing that would be asking somebody to fill in the box
/// the dialog has just switched off, which is a trap rather than a refusal.
fn what_a_condition_still_needs(match_words: &str, typed: &str) -> Option<&'static str> {
    if the_pattern_box_asks_for_something(match_words) && typed.trim().is_empty() {
        Some("Something to compare against is needed before this can be saved.")
    } else {
        None
    }
}

/// Say what a saved search cannot find with the field now showing, or clear
/// the line.
///
/// Shown and said, through the one call that does both, for the reason
/// [`crate::presentation::status_line`] gives: a line of text under a window
/// raises no notification and is not somewhere anybody navigating by ear goes,
/// so a sentence only written there is a sentence nobody gets. This one is the
/// whole point of offering the field at all, so it is the last one to leave
/// unsaid.
///
/// At the ordinary level rather than above it. It is not a refusal, it is what
/// the choice just made means.
fn say_what_this_field_cannot_find(line: &StaticText, a11y: &Accessibility, field_words: &str) {
    let cannot = the_field_those_words_name(field_words)
        .and_then(crate::application::saved_searches::what_a_saved_search_cannot_find_with);
    match cannot {
        Some(said) => said_and_shown(line, a11y, said, Priority::Normal),
        // Cleared rather than said. There is nothing to disclose about this
        // field, and announcing an empty string would be a sound with no
        // sentence in it every time somebody arrowed through the list.
        None => line.set_label(""),
    }
}

/// What `show_rule_edit` still needs after construction: the dialog to run
/// `.show_modal()` on, every control to read back once OK is pressed, and the
/// line that says what the chosen field can find.
pub struct RuleEditWidgets {
    pub dialog: Dialog,
    pub field_choice: Choice,
    pub match_choice: Choice,
    pub pattern_f: TextCtrl,
    pub cs_check: CheckBox,
    pub what_it_can_find: StaticText,
}

/// Build the Add/Edit Condition dialog without showing it.
///
/// One condition of a saved search: which part of a message to look at, how to
/// compare it, and what to compare it against. D-2-01 makes a smart folder a
/// saved search with a fuller editor rather than a second object, so this
/// writes a [`Question`] and nothing else. A `Question` becomes a `FilterRule`
/// through `Question::as_a_rule`, which is the one matcher, so there is no
/// second way here to describe a condition.
///
/// Split from [`show_rule_edit`] the way every dialog in this file is split: a
/// test can build the real dialog and read back the real value a live control
/// holds, and never call `.show_modal()` at all.
///
/// Both lists come from the engine's own constants through
/// [`the_words_for_every_field`] and [`the_words_for_every_way_of_matching`],
/// which the filter editor reads too. All eleven fields are offered, including
/// the one a saved search never sees, because D-2-01 says the editor writes
/// any of the eleven. What that field does is said out loud instead of the
/// field being quietly left out.
pub fn build_rule_edit_dialog(
    parent: &Dialog,
    existing: Option<&Question>,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> RuleEditWidgets {
    let title = if existing.is_some() {
        "Edit Condition"
    } else {
        "Add Condition"
    };
    let dlg = Dialog::builder(parent, title).with_size(520, 320).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Accelerators: F(Field), T(Type), P(Pattern), C(Case), all first letters
    // and the same four the filter editor uses for the same four controls.
    let field_label = StaticText::builder(&dlg)
        .with_label("Match &Field:")
        .build();
    let field_choice = Choice::builder(&dlg)
        .with_choices(the_words_for_every_field())
        .build();
    set_accessible_name(&field_choice, "Match field");
    fields.add(
        &field_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&field_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let match_label = StaticText::builder(&dlg).with_label("Match &Type:").build();
    let match_choice = Choice::builder(&dlg)
        .with_choices(the_words_for_every_way_of_matching())
        .build();
    set_accessible_name(&match_choice, "Match type");
    fields.add(
        &match_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&match_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let pattern_f = add_field(&dlg, &fields, "&Pattern:");

    let cs_label = StaticText::builder(&dlg).with_label("").build();
    let cs_check = CheckBox::builder(&dlg)
        .with_label("&Case Sensitive")
        .build();
    fields.add(&cs_label, 0, SizerFlag::All, 4);
    fields.add(&cs_check, 0, SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // Beside the control rather than in a tooltip. A tooltip is reached by
    // hovering, which is not how anybody this is written for is working, and
    // it is not read out when the value under it changes.
    let what_it_can_find = StaticText::builder(&dlg).with_label("").build();
    sizer.add(
        &what_it_can_find,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom,
        8,
    );

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add_spacer(0);
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    match existing {
        Some(question) => {
            // The words, because the words are what the lists hold. Selecting
            // by the stored name selects nothing, and pressing OK on a
            // condition that opened that way rewrites its field to the empty
            // string. That happened for five of the eleven fields in the
            // filter editor before plan 02-04.
            if let Some(said) = the_words_for_a_field(&question.field) {
                select_choice_by_string(&field_choice, said);
            }
            if let Some(said) = the_words_for_a_way_of_matching(&question.match_type) {
                select_choice_by_string(&match_choice, said);
            }
            pattern_f.set_value(&question.pattern);
            cs_check.set_value(question.case_sensitive);
        }
        None => {
            let (field, way) = WHAT_A_NEW_CONDITION_ASKS_FIRST;
            if let Some(said) = the_words_for_a_field(field) {
                select_choice_by_string(&field_choice, said);
            }
            if let Some(said) = the_words_for_a_way_of_matching(way) {
                select_choice_by_string(&match_choice, said);
            }
        }
    }

    say_what_this_field_cannot_find(
        &what_it_can_find,
        a11y,
        &get_choice_string(&field_choice).unwrap_or_default(),
    );
    field_choice.on_selection_changed({
        let line = what_it_can_find;
        let a11y = Arc::clone(a11y);
        move |event| {
            say_what_this_field_cannot_find(&line, &a11y, &event.get_string().unwrap_or_default());
        }
    });

    // The Pattern box only asks when there is something to compare against.
    // Disabled rather than taken out of the sizer, so the tab order does not
    // move under somebody working by ear.
    pattern_f.enable(the_pattern_box_asks_for_something(
        &get_choice_string(&match_choice).unwrap_or_default(),
    ));
    match_choice.on_selection_changed({
        let box_to_ask_with = pattern_f;
        move |event| {
            box_to_ask_with.enable(the_pattern_box_asks_for_something(
                &event.get_string().unwrap_or_default(),
            ));
        }
    });

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick, the same
            // way the filter editor's missing-name refusal does.
            event.event.skip(false);
            let match_words = get_choice_string(&match_choice).unwrap_or_default();
            if let Some(needed) = what_a_condition_still_needs(&match_words, &pattern_f.get_value())
            {
                // "Not saved" rather than "Not added": this dialog is opened
                // on a stored condition as well as on a new one, and a
                // caption a screen reader reads out before the sentence must
                // not contradict it.
                a_sub_dialog_needs(&d, "Not saved", needed);
                pattern_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&pattern_f, palette.main_surface());
    }

    RuleEditWidgets {
        dialog: dlg,
        field_choice,
        match_choice,
        pattern_f,
        cs_check,
        what_it_can_find,
    }
}

/// Open the Add/Edit Condition dialog and give back the condition, if one was
/// saved.
///
/// Opened from [`show_rule_manager_dialog`], for Add and for Edit alike, which
/// is what makes it the second dialog of two rather than a window of its own.
pub fn show_rule_edit(
    parent: &Dialog,
    existing: Option<&Question>,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> Option<Question> {
    let RuleEditWidgets {
        dialog: dlg,
        field_choice,
        match_choice,
        pattern_f,
        cs_check,
        what_it_can_find: _,
    } = build_rule_edit_dialog(parent, existing, a11y, palette);

    // Read first, then destroy: the controls belong to the dialog, and
    // wxWidgets does not free one when the Rust value goes.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        // The lists offer words and a question stores names, so both come back
        // through the same pair of conversions the dialog built them with.
        let match_words = get_choice_string(&match_choice).unwrap_or_default();
        Some(Question {
            field: get_choice_string(&field_choice)
                .and_then(|said| the_field_those_words_name(&said))
                .unwrap_or_default()
                .to_string(),
            match_type: the_way_of_matching_those_words_name(&match_words)
                .unwrap_or_default()
                .to_string(),
            pattern: the_pattern_to_store(&match_words, &pattern_f.get_value()),
            case_sensitive: cs_check.get_value(),
        })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

/// One condition as the three things its row says: what it looks at, how it
/// compares, and what it compares against.
///
/// In words at every position, from the same two builders both rule dialogs
/// offer their lists from, so a row and the list somebody chose it from cannot
/// come to say different things. A stored name this build has no words for is
/// shown as it is stored rather than blanked: a condition written by a later
/// version is then still a row somebody can see, and remove.
///
/// Case sensitivity goes on the end of the third part rather than into a
/// fourth column, and only when it is on. Two conditions that differ only in
/// it would otherwise be two rows nobody could tell apart, which is the fault
/// this codebase keeps finding in lists; a column carrying "no" on almost
/// every row is the other way to get it wrong.
fn what_a_condition_row_says(question: &Question) -> [String; 3] {
    let in_words = |stored: &str, said: Option<&'static str>| said.unwrap_or(stored).to_string();
    let against = match (question.pattern.as_str(), question.case_sensitive) {
        ("", _) => String::new(),
        (pattern, false) => pattern.to_string(),
        (pattern, true) => format!("{pattern} (case sensitive)"),
    };
    [
        in_words(&question.field, the_words_for_a_field(&question.field)),
        in_words(
            &question.match_type,
            the_words_for_a_way_of_matching(&question.match_type),
        ),
        against,
    ]
}

/// One condition as a sentence, for the line that says what just changed.
///
/// The same three parts the row shows, joined, so the sentence and the row
/// name one thing the same way. Empty parts are dropped: a way of matching
/// that compares against nothing has no pattern, and "Read is yes ''" is a
/// pair of quotation marks read out for no reason.
fn a_condition_in_words(question: &Question) -> String {
    what_a_condition_row_says(question)
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// What a saved search's condition list still needs before its window can
/// close, if anything.
///
/// At least one condition. A search that asks nothing takes the whole mailbox
/// when its questions are joined with Any and nothing at all when they are
/// joined with All, and neither is a search anybody wrote.
///
/// The store refuses the same thing (`MessageCache::replace_saved_search`),
/// and two refusals is deliberate rather than a duplicate: a window is where
/// somebody can be told what is wrong while they can still fix it, and a store
/// is where nothing gets past whatever forgot to ask.
///
/// Public because the Close button is not the only way out of a window. The
/// caller reads this too, for a list somebody emptied and then left by the
/// close box or Escape, so there is one wording of the refusal rather than a
/// second one written where the write happens.
pub fn what_a_condition_list_still_needs(questions: &[Question]) -> Option<&'static str> {
    questions.is_empty().then_some(
        "A saved search has to ask at least one thing about a message. Add a condition \
         before closing this window.",
    )
}

/// The rows of a saved search's condition list, repainted from the working
/// copy.
///
/// Begins by emptying the control, the way `populate_filters` does: this runs
/// after every add, edit and delete, and appending to what is already there
/// would leave the removed row on screen.
///
/// Public for the same reason [`make_shell`] and [`delete_selected`] are: the
/// checks that need a real list live in `tests/`, because a process may build
/// one wxWidgets application and the library test binary has spent that budget
/// elsewhere.
pub fn populate_questions(list: &ListCtrl, questions: &[Question]) {
    list.delete_all_items();
    for (i, question) in questions.iter().enumerate() {
        let idx = i as i64;
        let [looks_at, compares, against] = what_a_condition_row_says(question);
        list.insert_item(idx, &looks_at, None);
        list.set_item_text_by_column(idx, 1, &compares);
        list.set_item_text_by_column(idx, 2, &against);
    }
}

/// Open the conditions of one saved search, and give back the new list if any
/// of it changed.
///
/// The second door D-2-01 describes. The search box keeps writing its three
/// questions; this writes any of the eleven fields the filter engine answers,
/// with any of the eleven ways it can match. Both land in the same stored
/// search and both run through `Question::as_a_rule`, so there is one matcher
/// and one storage underneath the two doors.
///
/// `None` means nothing was changed, which is not the same as an empty list:
/// the caller writes only when something came back, so opening this window and
/// closing it again touches nothing at all.
///
/// On [`run_manager_loop`], which is the only shape here where the number of
/// rows and the position in the set reach both accessibility channels from
/// Windows' own provider for a native list. A hand-rolled stack of rows would
/// have to say the count with `set_accessible_name`, which writes to MSAA
/// only, and its tab order would change as conditions came and went.
///
/// Two dialogs deep at most: this window, and the condition editor it opens.
/// The same depth the filter manager already reaches.
pub fn show_rule_manager_dialog(
    parent: &Frame,
    search_named: &str,
    questions: &[Question],
    a11y: &Arc<Accessibility>,
) -> Option<Vec<Question>> {
    // Read once and reused for this shell and every condition dialog it opens,
    // rather than a second, independent disk read per dialog.
    let palette = theme::current_from_stored_config();
    let (dialog, sizer, list, status) = make_shell(
        parent,
        &format!("Conditions for {search_named}"),
        "Conditions",
        620,
        420,
        palette,
    );

    list.insert_column(0, "Looks at", ListColumnFormat::Left, 170);
    list.insert_column(1, "How", ListColumnFormat::Left, 190);
    list.insert_column(2, "What", ListColumnFormat::Left, 220);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = questions.to_vec();
    let changed = run_manager_loop(
        ManagerChrome {
            dialog: &dialog,
            main_sizer: &sizer,
            list: &list,
            status_text: &status,
            a11y: a11y.clone(),
        },
        manager_words::CONDITION,
        &mut working,
        populate_questions,
        |d, existing| show_rule_edit(d, existing, a11y, palette),
        a_condition_in_words,
        what_a_condition_list_still_needs,
    );

    changed.then_some(working)
}

fn populate_filters(list: &ListCtrl, rules: &[FilterRule]) {
    list.delete_all_items();
    for (i, r) in rules.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &r.name, None);
        list.set_item_text_by_column(
            idx,
            1,
            &format!("{} {} '{}'", r.field, r.match_type, r.pattern),
        );
        let action = if r.action_value.is_empty() {
            r.action_type.clone()
        } else {
            format!("{} ({})", r.action_type, r.action_value)
        };
        list.set_item_text_by_column(idx, 2, &action);
        list.set_item_text_by_column(idx, 3, if r.enabled { "Active" } else { "Disabled" });
    }
}

/// What `show_filter_edit` still needs after construction: the dialog to
/// run `.show_modal()` on, and every field and Choice to read back once OK
/// is pressed.
pub struct FilterEditWidgets {
    pub dialog: Dialog,
    pub name_f: TextCtrl,
    pub field_choice: Choice,
    pub match_choice: Choice,
    pub pattern_f: TextCtrl,
    pub cs_check: CheckBox,
    pub action_choice: Choice,
    pub action_value_f: TextCtrl,
    pub en_check: CheckBox,
}

/// Build the Add/Edit Filter Rule dialog without showing it.
///
/// Everything `show_filter_edit` used to do up to its own `.show_modal()`
/// call, split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
pub fn build_filter_edit_dialog(
    parent: &Dialog,
    existing: Option<&FilterRule>,
    palette: Option<theme::Palette>,
) -> FilterEditWidgets {
    let title = if existing.is_some() {
        "Edit Filter Rule"
    } else {
        "Add Filter Rule"
    };
    let dlg = Dialog::builder(parent, title).with_size(480, 440).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Accelerators: N(Name), F(Field), T(Type), P(Pattern), C(Case),
    //   A(Action), V(Value), E(Enabled), all first letters
    let name_f = add_field(&dlg, &fields, "Rule &Name:");

    let field_label = StaticText::builder(&dlg)
        .with_label("Match &Field:")
        .build();
    // Built from the engine's own list, the same way the Action list below is
    // built from `RULE_ACTIONS`. It used to hold six names of its own, five of
    // them column names, while the engine answered questions about eleven
    // fields: a rule about the message text, about the date, or about any of
    // the three flags could not be written here at all, and the six that could
    // were offered as `body_plain` and the rest.
    //
    // Through the shared builder, which the condition editor reads too.
    let field_choice = Choice::builder(&dlg)
        .with_choices(the_words_for_every_field())
        .build();
    set_accessible_name(&field_choice, "Match field");
    fields.add(
        &field_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&field_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let match_label = StaticText::builder(&dlg).with_label("Match &Type:").build();
    let match_choice = Choice::builder(&dlg)
        .with_choices(the_words_for_every_way_of_matching())
        .build();
    set_accessible_name(&match_choice, "Match type");
    fields.add(
        &match_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&match_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let pattern_f = add_field(&dlg, &fields, "&Pattern:");

    let cs_label = StaticText::builder(&dlg).with_label("").build();
    let cs_check = CheckBox::builder(&dlg)
        .with_label("&Case Sensitive")
        .build();
    fields.add(&cs_label, 0, SizerFlag::All, 4);
    fields.add(&cs_check, 0, SizerFlag::All, 4);

    let action_label = StaticText::builder(&dlg).with_label("&Action:").build();
    let action_choices: Vec<String> = RULE_ACTIONS
        .iter()
        .map(|(_, shown)| (*shown).to_string())
        .collect();
    let action_choice = Choice::builder(&dlg).with_choices(action_choices).build();
    set_accessible_name(&action_choice, "Action");
    fields.add(
        &action_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&action_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let action_value_f = add_field(&dlg, &fields, "Action &Value:");

    let en_label = StaticText::builder(&dlg).with_label("").build();
    let en_check = CheckBox::builder(&dlg).with_label("&Enabled").build();
    en_check.set_value(true);
    fields.add(&en_label, 0, SizerFlag::All, 4);
    fields.add(&en_check, 0, SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add_spacer(0);
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    if let Some(r) = existing {
        name_f.set_value(&r.name);
        // The words, because the words are what the list holds now. Selecting
        // by the stored name silently selected nothing for five of the eleven
        // fields, and pressing OK on a rule that opened that way rewrote its
        // field to the empty string.
        if let Some(said) = the_words_for_a_field(&r.field) {
            select_choice_by_string(&field_choice, said);
        }
        if let Some(said) = the_words_for_a_way_of_matching(&r.match_type) {
            select_choice_by_string(&match_choice, said);
        }
        pattern_f.set_value(&r.pattern);
        cs_check.set_value(r.case_sensitive);
        select_choice_by_string(&action_choice, shown_action(&r.action_type));
        action_value_f.set_value(&r.action_value);
        en_check.set_value(r.enabled);
    }

    // The Pattern box only asks when there is something to compare against.
    //
    // Disabled rather than taken out of the sizer. Removing it would rebuild
    // the layout under somebody's hands, and the tab order has to stay put
    // while a screen reader is on a control two rows above.
    let asks_now =
        the_pattern_box_asks_for_something(&get_choice_string(&match_choice).unwrap_or_default());
    pattern_f.enable(asks_now);
    match_choice.on_selection_changed({
        let box_to_ask_with = pattern_f;
        move |event| {
            box_to_ask_with.enable(the_pattern_box_asks_for_something(
                &event.get_string().unwrap_or_default(),
            ));
        }
    });

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick; see
            // `wx_item_form.rs`'s module doc comment. Without a name the
            // manager's own sentence reads "Added the rule: " and stops.
            event.event.skip(false);
            if name_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(
                    &d,
                    "Not saved",
                    "A name is needed before this can be saved.",
                );
                name_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last. The three Choice controls and the two CheckBox controls
    // are left to Windows, matching every other Choice and CheckBox this
    // round paints around. `None` means high contrast is on, or the system
    // is set up in a way this application should not paint over, so nothing
    // is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        for field in [&name_f, &pattern_f, &action_value_f] {
            theme::paint(field, palette.main_surface());
        }
    }

    FilterEditWidgets {
        dialog: dlg,
        name_f,
        field_choice,
        match_choice,
        pattern_f,
        cs_check,
        action_choice,
        action_value_f,
        en_check,
    }
}

fn show_filter_edit(
    parent: &Dialog,
    existing: Option<&FilterRule>,
    palette: Option<theme::Palette>,
) -> Option<FilterRule> {
    let FilterEditWidgets {
        dialog: dlg,
        name_f,
        field_choice,
        match_choice,
        pattern_f,
        cs_check,
        action_choice,
        action_value_f,
        en_check,
    } = build_filter_edit_dialog(parent, existing, palette);

    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        // The lists offer words and a rule stores names, so both come back
        // through the same pair of conversions the dialog built them with.
        let match_words = get_choice_string(&match_choice).unwrap_or_default();
        Some(FilterRule {
            id: existing
                .map(|r| r.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            field: get_choice_string(&field_choice)
                .and_then(|said| the_field_those_words_name(&said))
                .unwrap_or_default()
                .to_string(),
            match_type: the_way_of_matching_those_words_name(&match_words)
                .unwrap_or_default()
                .to_string(),
            pattern: the_pattern_to_store(&match_words, &pattern_f.get_value()),
            case_sensitive: cs_check.get_value(),
            action_type: stored_action(&get_choice_string(&action_choice).unwrap_or_default()),
            action_value: action_value_f.get_value(),
            enabled: en_check.get_value(),
        })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

// ══════════════════════════════════════════════════════════════════════════════
// Tag Manager
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct TagEntry {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub enum TagManagerAction {
    None,
    Updated(Vec<TagEntry>),
}

const TAG_COLORS: &[(&str, &str)] = &[
    ("Red", "#E53935"),
    ("Orange", "#FB8C00"),
    ("Yellow", "#FDD835"),
    ("Green", "#43A047"),
    ("Blue", "#1E88E5"),
    ("Purple", "#8E24AA"),
    ("Pink", "#D81B60"),
    ("Gray", "#757575"),
];

pub fn show_tag_manager_dialog(
    parent: &Frame,
    tags: &[TagEntry],
    a11y: &Arc<Accessibility>,
) -> TagManagerAction {
    // Read once and reused for the manager shell and every Add/Edit dialog
    // it opens, rather than a second, independent disk read per dialog (see
    // `theme::current_from_stored_config`'s own doc comment for why that
    // matters).
    let palette = theme::current_from_stored_config();
    let (dialog, sizer, list, status) =
        make_shell(parent, "Tag Manager", "Tags", 450, 400, palette);

    list.insert_column(0, "Tag", ListColumnFormat::Left, 200);
    list.insert_column(1, "Color", ListColumnFormat::Left, 100);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = tags.to_vec();
    let changed = run_manager_loop(
        ManagerChrome {
            dialog: &dialog,
            main_sizer: &sizer,
            list: &list,
            status_text: &status,
            a11y: a11y.clone(),
        },
        manager_words::TAG,
        &mut working,
        populate_tags,
        |d, existing| show_tag_edit(d, existing, palette),
        |t| t.name.clone(),
        nothing_stops_this_closing,
    );

    if changed {
        TagManagerAction::Updated(working)
    } else {
        TagManagerAction::None
    }
}

fn populate_tags(list: &ListCtrl, tags: &[TagEntry]) {
    list.delete_all_items();
    for (i, t) in tags.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &t.name, None);
        let color_name = TAG_COLORS
            .iter()
            .find(|(_, hex)| *hex == t.color)
            .map(|(name, _)| *name)
            .unwrap_or(&t.color);
        list.set_item_text_by_column(idx, 1, color_name);
    }
}

/// Build the Add/Edit Tag dialog without showing it.
///
/// Everything `show_tag_edit` used to do up to its own `.show_modal()` call,
/// split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
///
/// Returns the name field and the colour choice alongside the dialog, the
/// same way `show_tag_edit` still needs them after a real `.show_modal()`.
pub fn build_tag_edit_dialog(
    parent: &Dialog,
    existing: Option<&TagEntry>,
    palette: Option<theme::Palette>,
) -> (Dialog, TextCtrl, Choice) {
    let title = if existing.is_some() {
        "Edit Tag"
    } else {
        "Add Tag"
    };
    let dlg = Dialog::builder(parent, title).with_size(350, 250).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Accelerators are first letters, no conflicts: N(Name), C(Color)
    let name_f = add_field(&dlg, &fields, "Tag &Name:");

    let color_label = StaticText::builder(&dlg).with_label("&Color:").build();
    let color_choices: Vec<String> = TAG_COLORS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    let color_choice = Choice::builder(&dlg).with_choices(color_choices).build();
    set_accessible_name(&color_choice, "Colour");
    color_choice.set_selection(0);
    fields.add(
        &color_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&color_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add_spacer(0);
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    if let Some(t) = existing {
        name_f.set_value(&t.name);
        if let Some(pos) = TAG_COLORS.iter().position(|(_, hex)| *hex == t.color) {
            color_choice.set_selection(pos as u32);
        }
    }

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick; see
            // `wx_item_form.rs`'s module doc comment. Without a name the
            // manager's own sentence reads "Added the tag: " and stops.
            event.event.skip(false);
            if name_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(
                    &d,
                    "Not saved",
                    "A name is needed before this can be saved.",
                );
                name_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last. The colour Choice is left to Windows, matching every
    // other Choice this round paints around. `None` means high contrast is
    // on, or the system is set up in a way this application should not
    // paint over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&name_f, palette.main_surface());
    }

    (dlg, name_f, color_choice)
}

fn show_tag_edit(
    parent: &Dialog,
    existing: Option<&TagEntry>,
    palette: Option<theme::Palette>,
) -> Option<TagEntry> {
    let (dlg, name_f, color_choice) = build_tag_edit_dialog(parent, existing, palette);

    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        let color_idx = color_choice.get_selection().unwrap_or(0) as usize;
        let color = TAG_COLORS
            .get(color_idx)
            .map(|(_, hex)| hex.to_string())
            .unwrap_or_else(|| "#1E88E5".to_string());
        Some(TagEntry {
            id: existing
                .map(|t| t.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            color,
        })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

// ══════════════════════════════════════════════════════════════════════════════
// Signature Manager
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SignatureEntry {
    pub id: String,
    pub name: String,
    pub content_plain: String,
    pub content_html: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub enum SignatureManagerAction {
    None,
    Updated(Vec<SignatureEntry>),
}

pub fn show_signature_manager_dialog(
    parent: &Frame,
    signatures: &[SignatureEntry],
    a11y: &Arc<Accessibility>,
) -> SignatureManagerAction {
    // Read once and reused for the manager shell and every Add/Edit dialog
    // it opens, rather than a second, independent disk read per dialog (see
    // `theme::current_from_stored_config`'s own doc comment for why that
    // matters).
    let palette = theme::current_from_stored_config();
    let (dialog, sizer, list, status) =
        make_shell(parent, "Signature Manager", "Signatures", 550, 450, palette);

    list.insert_column(0, "Name", ListColumnFormat::Left, 200);
    list.insert_column(1, "Default", ListColumnFormat::Centre, 80);
    list.insert_column(2, "Preview", ListColumnFormat::Left, 220);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = signatures.to_vec();
    let changed = run_manager_loop(
        ManagerChrome {
            dialog: &dialog,
            main_sizer: &sizer,
            list: &list,
            status_text: &status,
            a11y: a11y.clone(),
        },
        manager_words::SIGNATURE,
        &mut working,
        populate_sigs,
        |d, existing| show_sig_edit(d, existing, palette),
        |s| s.name.clone(),
        nothing_stops_this_closing,
    );

    if changed {
        // Ensure at most one default (last-added wins)
        let mut saw_default = false;
        for s in working.iter_mut().rev() {
            if s.is_default {
                if saw_default {
                    s.is_default = false;
                }
                saw_default = true;
            }
        }
        SignatureManagerAction::Updated(working)
    } else {
        SignatureManagerAction::None
    }
}

fn populate_sigs(list: &ListCtrl, sigs: &[SignatureEntry]) {
    list.delete_all_items();
    for (i, s) in sigs.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &s.name, None);
        list.set_item_text_by_column(idx, 1, if s.is_default { "★" } else { "" });
        let preview: String = s.content_plain.chars().take(50).collect();
        list.set_item_text_by_column(idx, 2, &preview);
    }
}

/// What `show_sig_edit` still needs after construction: the dialog to run
/// `.show_modal()` on, and every field it reads back once OK is pressed.
pub struct SigEditWidgets {
    pub dialog: Dialog,
    pub name_f: TextCtrl,
    pub def_check: CheckBox,
    pub content_f: TextCtrl,
}

/// Build the Add/Edit Signature dialog without showing it.
///
/// Everything `show_sig_edit` used to do up to its own `.show_modal()` call,
/// split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
pub fn build_sig_edit_dialog(
    parent: &Dialog,
    existing: Option<&SignatureEntry>,
    palette: Option<theme::Palette>,
) -> SigEditWidgets {
    let title = if existing.is_some() {
        "Edit Signature"
    } else {
        "Add Signature"
    };
    let dlg = Dialog::builder(parent, title).with_size(500, 420).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Accelerators are all first letters: N(Name), D(Default), S(Signature/plain), H(HTML)
    let name_f = add_field(&dlg, &fields, "&Name:");

    let def_label = StaticText::builder(&dlg).with_label("").build();
    let def_check = CheckBox::builder(&dlg)
        .with_label("&Default signature")
        .build();
    fields.add(&def_label, 0, SizerFlag::All, 4);
    fields.add(&def_check, 0, SizerFlag::All, 4);

    sizer.add_sizer(
        &fields,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        8,
    );

    let plain_label = StaticText::builder(&dlg)
        .with_label("&Signature, in Markdown:")
        .build();
    sizer.add(&plain_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    let content_f = TextCtrl::builder(&dlg)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();
    set_accessible_name_and_description(
        &content_f,
        "Signature",
        "Markdown becomes real formatting in the message. What you type here is \
         what a plain text reader sees",
    );
    sizer.add(&content_f, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // There was a second box here, headed "HTML version (optional)", for
    // writing the formatted signature by hand. It was stored, carried through
    // three layers and written to the database, and the send path took
    // `content_plain` and dropped the rest, so it never reached a message and
    // said it had saved. Markdown in the box above is what it was for, and that
    // works, so the box is gone rather than wired to a second way of saying the
    // same thing. What anybody typed into it is still on the record and is not
    // thrown away by editing a signature.

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add_spacer(0);
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    if let Some(s) = existing {
        name_f.set_value(&s.name);
        content_f.set_value(&s.content_plain);
        def_check.set_value(s.is_default);
    }

    ok.on_click({
        let d = dlg;
        move |event| {
            // Consuming the click is what makes the refusal stick; see
            // `wx_item_form.rs`'s module doc comment. Without a name the
            // manager's own sentence reads "Added the signature: " and stops.
            event.event.skip(false);
            if name_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(
                    &d,
                    "Not saved",
                    "A name is needed before this can be saved.",
                );
                name_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last. The default-signature CheckBox is left to Windows, the
    // same as every checkbox elsewhere in this round. `None` means high
    // contrast is on, or the system is set up in a way this application
    // should not paint over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        for field in [&name_f, &content_f] {
            theme::paint(field, palette.main_surface());
        }
    }

    SigEditWidgets {
        dialog: dlg,
        name_f,
        def_check,
        content_f,
    }
}

fn show_sig_edit(
    parent: &Dialog,
    existing: Option<&SignatureEntry>,
    palette: Option<theme::Palette>,
) -> Option<SignatureEntry> {
    let SigEditWidgets {
        dialog: dlg,
        name_f,
        def_check,
        content_f,
    } = build_sig_edit_dialog(parent, existing, palette);

    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        Some(SignatureEntry {
            id: existing
                .map(|s| s.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            content_plain: content_f.get_value(),
            // Kept as it was found rather than cleared. Nothing reads it and
            // nothing offers to edit it any more, and quietly deleting what
            // somebody typed is not this change's business.
            content_html: existing.and_then(|s| s.content_html.clone()),
            is_default: def_check.get_value(),
        })
    } else {
        None
    };
    dlg.destroy();
    chosen
}

/// How often the waiting window looks to see whether the answer has arrived.
///
/// The same interval the main window polls its own updates on. Often enough
/// that nobody notices the gap between the answer arriving and the window
/// saying so, rare enough to cost nothing while it waits.
const HOW_OFTEN_TO_LOOK_FOR_THE_ANSWER: i32 = 50;

/// Build the "please wait" dialog without showing it.
///
/// Everything `wait_for_an_answer` used to do up to its own timer, channel
/// and `.show_modal()` call, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// Returns the stop button alongside the dialog, the same way
/// `wait_for_an_answer` still needs it after a real `.show_modal()`: to wire
/// its click and give it focus.
pub fn build_wait_for_an_answer_dialog<W: WxWidget>(
    parent: &W,
    title: &str,
    what_is_happening: &str,
    stop: &str,
    palette: Option<theme::Palette>,
) -> (Dialog, Button) {
    let dialog = Dialog::builder(parent, title).with_size(520, 200).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Named as well as shown. A static text with no name of its own is read
    // from its label on most builds and from nothing on some, and this is the
    // only sentence in the window.
    let saying = StaticText::builder(&dialog)
        .with_label(what_is_happening)
        .build();
    set_accessible_name(&saying, what_is_happening);
    sizer.add(&saying, 1, SizerFlag::Expand | SizerFlag::All, 12);

    // The one control, and it carries the cancel id, so Escape and the window's
    // own close button both mean the same thing as pressing it.
    let stop_button = Button::builder(&dialog)
        .with_label(stop)
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&stop_button, &name_from_label(stop));
    sizer.add(&stop_button, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dialog.set_sizer(sizer, true);

    // Painted last. No `TextCtrl`, `ListCtrl` or `TreeCtrl` anywhere in this
    // dialog, so the dialog itself is the only site. `None` means high
    // contrast is on, or the system is set up in a way this application
    // should not paint over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }

    (dialog, stop_button)
}

/// Wait for an answer being worked out somewhere else, with the window alive.
///
/// The alternative, waiting on this thread, is a window that cannot repaint,
/// cannot answer a key and cannot speak for as long as the wait lasts. A window
/// that cannot repaint also cannot speak, so anybody working by ear gets
/// silence and no way to tell whether it is working, has finished, or has died.
///
/// This shows a small window that says what is happening and offers a way to
/// stop. It is modal, so it runs an event loop of its own: everything else
/// carries on, announcements are still spoken, and the wait is somebody's to
/// end at any point. `None` is somebody stopping it or closing it, which must
/// leave everything as it was.
///
/// Whatever produces the answer sends it down `coming`. It may keep running
/// after this returns: the answer then has nowhere to go and is dropped, which
/// is what makes stopping safe rather than only apparent.
///
/// **Not confirmed with a screen reader.** The window is named and its one
/// control is focused, which is what reaches NVDA, and whether it is usable is
/// a thing only a screen reader run answers.
/// `parent` is generic rather than fixed to the application's own frame, the
/// same way `wx_item_form::ask_for` is, so a question asked from inside another
/// dialog can wait under that dialog rather than under a window behind it.
pub fn wait_for_an_answer<T: 'static, W: WxWidget>(
    parent: &W,
    title: &str,
    what_is_happening: &str,
    stop: &str,
    coming: async_channel::Receiver<T>,
    palette: Option<theme::Palette>,
) -> Option<T> {
    let (dialog, stop_button) =
        build_wait_for_an_answer_dialog(parent, title, what_is_happening, stop, palette);

    let arrived: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
    // Held until this function returns: dropping a timer stops it, so one that
    // fell out of scope here would never tick and the wait would never end.
    let watching = Timer::new(&dialog);
    watching.on_tick({
        let arrived = arrived.clone();
        move |_| {
            if let Ok(answer) = coming.try_recv() {
                *arrived.borrow_mut() = Some(answer);
                dialog.end_modal(ID_OK);
            }
        }
    });
    if !watching.start(HOW_OFTEN_TO_LOOK_FOR_THE_ANSWER, false) {
        tracing::error!("The waiting window's timer refused to start; nothing would end the wait");
        dialog.destroy();
        return None;
    }
    stop_button.on_click(move |_| dialog.end_modal(ID_CANCEL));
    // On the only control there is, so the window opens with something focused
    // and the way out is where the focus already is.
    stop_button.set_focus();

    let closed_with = dialog.show_modal();
    drop(watching);
    dialog.destroy();
    // Only when the wait ended because the answer arrived. Somebody who pressed
    // Stop is told nothing was done even if the answer landed in the same
    // instant, because what they asked for was to stop.
    if closed_with != ID_OK {
        return None;
    }
    arrived.borrow_mut().take()
}

/// Build the "pick one" dialog without showing it.
///
/// Everything `choose_from_list` used to do up to its own `.show_modal()`
/// call, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// Returns the list alongside the dialog, the same way `choose_from_list`
/// still needs it after a real `.show_modal()`: to read what was chosen.
pub fn build_choose_from_list_dialog(
    parent: &Frame,
    title: &str,
    label: &str,
    confirm: &str,
    items: &[String],
    palette: Option<theme::Palette>,
) -> (Dialog, ListBox) {
    let dlg = Dialog::builder(parent, title)
        .with_size(520, 380)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let heading = StaticText::builder(&dlg).with_label(label).build();
    sizer.add(&heading, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let list = ListBox::builder(&dlg).build();
    // Named from the label, with the ampersand stripped: a mnemonic read out
    // as "and" in the accessible name is a syllable that means nothing.
    set_accessible_name(&list, &label.replace(['&', ':'], ""));
    for item in items {
        list.append(item);
    }
    if !items.is_empty() {
        // Something is always selected, so Enter always has an answer and the
        // first row is read on arrival rather than after an arrow press.
        list.set_selection(0, true);
    }
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg)
        .with_label(confirm)
        .with_id(ID_OK)
        .build();
    set_accessible_name(&ok, &name_from_label(confirm));
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&cancel, "Cancel");
    buttons.add_spacer(0);
    buttons.add(&ok, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    ok.on_click({
        let d = dlg;
        move |_| d.end_modal(ID_OK)
    });
    cancel.on_click({
        let d = dlg;
        move |_| d.end_modal(ID_CANCEL)
    });

    // Painted last. The list is left to Windows: it offers one item per
    // choice, the same as a Choice or a radio group elsewhere in this round,
    // rather than showing rows of content the way a contact or an account
    // does. `None` means high contrast is on, or the system is set up in a
    // way this application should not paint over, so nothing is set here and
    // Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
    }

    (dlg, list)
}

/// Pick one item from a list, or nothing.
///
/// A single-selection list box with OK and Cancel. Deliberately plain: it is
/// reached by keyboard, read by a screen reader, and closed with Escape, and
/// none of that is improved by anything more elaborate.
///
/// `confirm` is what the button that accepts says, because the word has to
/// match what happens: "Open" is right for a draft and wrong for adding a
/// calendar, and a button whose word is wrong is a button somebody hesitates
/// over every time.
///
/// Returns the index chosen, or `None` if it was cancelled.
pub fn choose_from_list(
    parent: &Frame,
    title: &str,
    label: &str,
    confirm: &str,
    items: &[String],
    palette: Option<theme::Palette>,
) -> Option<usize> {
    let (dlg, list) = build_choose_from_list_dialog(parent, title, label, confirm, items, palette);
    // Read first, then destroy: the fields belong to the dialog.
    // wxWidgets does not free a dialog when the Rust value goes, and
    // nothing in this file did, so every one of these little windows
    // stayed for the life of the session. `wx_compose` hit the same
    // thing and says so where it fixed it.
    let answered = dlg.show_modal();
    let chosen = if answered == ID_OK {
        list.get_selection().map(|chosen| chosen as usize)
    } else {
        None
    };
    dlg.destroy();
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This file with its own tests cut off.
    ///
    /// Cut, because the samples below quote the very calls the checks look
    /// for, and a check that reads its own words passes with the code deleted.
    fn the_manager_windows() -> String {
        let whole = std::fs::read_to_string("src/presentation/wx_managers.rs")
            .expect("this file to be readable")
            .replace("\r\n", "\n");
        match whole.split_once("\n#[cfg(test)]") {
            Some((code, _)) => code.to_string(),
            None => whole,
        }
    }

    /// The one call that shows and says, with its own tests cut off.
    fn the_one_call_that_says() -> String {
        let whole = std::fs::read_to_string("src/presentation/status_line.rs")
            .expect("the shared status line to be readable")
            .replace("\r\n", "\n");
        match whole.split_once("\n#[cfg(test)]") {
            Some((code, _)) => code.to_string(),
            None => whole,
        }
    }

    /// What these windows answer without saying it out loud.
    ///
    /// Added, updated, deleted with a name, and select something first: the
    /// tag, signature, filter and contact windows all answer on a line of
    /// text above their buttons. Nothing raises a notification for that line,
    /// so an answer only written there is an answer nobody working by ear
    /// gets.
    ///
    /// Both names are counted. The shared loop calls its line one thing and
    /// the contact loop calls it another, and a check that counted one of them
    /// would pass with half the file silent.
    fn what_these_windows_never_say(windows: &str, the_one_call: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        let Some((_, helper)) = the_one_call.split_once("pub(crate) fn said_and_shown(") else {
            return vec![
                "nothing these windows can call both shows a sentence and says it, so \
                 every answer they give is silent"
                    .to_string(),
            ];
        };
        let body = &helper[..helper.find("\n}").unwrap_or(helper.len())];
        if !body.contains("a11y.announce(") {
            wrong.push("the one place that shows a sentence never says it out loud".to_string());
        }

        // Not `rl.set_label(` and `cl.set_label(`, which rewrite the visible
        // labels of two address fields when somebody picks a different
        // country. They relabel a box rather than answer a command, and they
        // carry a fault of their own that is not this one: the accessible
        // names of those two boxes are fixed when the dialog is built and are
        // never rewritten, so choosing Japan shows "Prefecture" while a screen
        // reader still says "State or region". That wants its own change.
        for line in ["status_text.set_label(", "status.set_label("] {
            let shown = windows.matches(line).count();
            if shown != 0 {
                wrong.push(format!(
                    "{shown} answers are put on the line of text by themselves, rather than \
                     through the one call that says them as well"
                ));
            }
        }

        let said = windows.matches("said_and_shown(").count();
        if said < 10 {
            wrong.push(format!(
                "only {said} answers these windows give are said out loud, and there are \
                 more answers than that"
            ));
        }

        // A promise in a comment with nothing checking it is what this round
        // is cleaning up: the description of this chrome said its line of text
        // announced changes for as long as it never did.
        if windows.contains("announce") && !windows.contains("said_and_shown(") {
            wrong.push(
                "the description of the shared chrome claims its line of text announces \
                 things, and nothing in this file says anything"
                    .to_string(),
            );
        }
        wrong
    }

    #[test]
    fn test_every_answer_the_manager_windows_give_is_said_out_loud() {
        // Ten answers across the tag, signature, filter and contact windows,
        // every one of them shown on a line of text and said nowhere.
        //
        // What this cannot see: whether the announcement reaches a screen
        // reader from inside a modal dialog, or whether the sentence handed in
        // is a true one. Only a screen reader run answers the first.
        let windows = the_manager_windows();
        assert!(
            windows.len() > 5000,
            "only {} characters were read, so the reading is broken",
            windows.len()
        );
        assert!(
            !windows.contains("fn the_manager_windows("),
            "the tests were not cut off, so the check is reading its own words"
        );
        let wrong = what_these_windows_never_say(&windows, &the_one_call_that_says());
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_these_windows_no_longer_word_their_own_added_updated_or_deleted() {
        // Two windows here said "Added" and "Updated" with nothing else, and
        // three delete sites here worded a delete character for character
        // like the mail path's own sentence for a message leaving a server.
        // All of that now goes through manager_words, which is what this
        // checks for rather than for the word "Deleted" itself: that owner's
        // own sentence starts with the same word.
        let windows = the_manager_windows();
        assert!(
            !windows.contains("\"Added\""),
            "a bare \"Added\" with no kind or name survives"
        );
        assert!(
            !windows.contains("\"Updated\""),
            "a bare \"Updated\" with no kind or name survives"
        );
        assert!(
            !windows.contains("format!(\"Deleted: "),
            "a delete worded here rather than through manager_words::deleted survives"
        );
    }

    #[test]
    fn test_each_manager_window_hands_the_shared_loop_its_own_kind() {
        // run_manager_loop and the contact window both ask manager_words for
        // their words, given the kind of row this window holds. Nothing
        // about the loop's own definition can see whether a caller handed it
        // the wrong word, because all four calls share one loop body; this
        // reads each caller instead.
        let windows = the_manager_windows();
        for (function, kind) in [
            ("fn show_filter_manager_dialog", "manager_words::FILTER"),
            ("fn show_tag_manager_dialog", "manager_words::TAG"),
            (
                "fn show_signature_manager_dialog",
                "manager_words::SIGNATURE",
            ),
            ("fn show_contact_manager_dialog", "manager_words::CONTACT"),
        ] {
            let start = windows
                .find(function)
                .unwrap_or_else(|| panic!("{function} is not in this file"));
            // Bounded by the next top-level function this file declares,
            // rather than a fixed width: the contact window's own function is
            // long enough that a short, fixed window missed the calls
            // entirely and reported the right wiring as missing.
            let end = windows[start + function.len()..]
                .find("\npub fn ")
                .map(|at| start + function.len() + at)
                .unwrap_or(windows.len());
            let body = &windows[start..end];
            assert!(
                body.contains(kind),
                "{function} does not hand its own kind, {kind}, to the words it says"
            );
        }
    }

    #[test]
    fn test_the_manager_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes,
        // and from outside that is indistinguishable from one that finds
        // everything.
        let call = "pub(crate) fn said_and_shown(\n\
            \x20   line: &StaticText,\n\
            ) {\n\
            \x20   line.set_label(said);\n\
            \x20   let _ = a11y.announce(said, priority);\n\
            }\n";
        let sound = "said_and_shown(&status, a11y, x, Priority::High);\n".repeat(10);
        assert!(
            what_these_windows_never_say(&sound, call).is_empty(),
            "windows that say everything were reported as silent"
        );

        // The two names the same line goes by. A check that counted only the
        // first would pass with the whole contact window silent, and the other
        // way round.
        for left_silent in ["status_text.set_label(x);\n", "status.set_label(x);\n"] {
            let one_left = format!("{sound}{left_silent}");
            let wrong = what_these_windows_never_say(&one_left, call);
            assert!(
                wrong.iter().any(|said| said.contains("by themselves")),
                "a line written by itself as {left_silent:?} was not reported: {wrong:?}"
            );
        }

        let too_few = "said_and_shown(&status, a11y, x, Priority::High);\n".repeat(9);
        let wrong = what_these_windows_never_say(&too_few, call);
        assert!(
            wrong.iter().any(|said| said.contains("more answers")),
            "windows that lost answers were not reported: {wrong:?}"
        );

        let promise_only = "the status line that announces changes\n";
        let wrong = what_these_windows_never_say(promise_only, call);
        assert!(
            wrong.iter().any(|said| said.contains("claims its line")),
            "a promise with nothing behind it was not reported: {wrong:?}"
        );
        // And that check is awake on the real file rather than only on the
        // sample above. It fires when the description promises something the
        // file does nothing about, so it is asleep the moment the description
        // stops promising.
        assert!(
            the_manager_windows().contains("announce"),
            "the description of the shared chrome no longer promises anything, so the \
             promise check is asleep on the real file"
        );

        let never_says = call.replace("let _ = a11y.announce(said, priority);", "let _ = said;");
        assert!(
            what_these_windows_never_say(&sound, &never_says)
                .iter()
                .any(|said| said.contains("never says it out loud")),
            "a call that only shows was not reported"
        );

        assert!(
            what_these_windows_never_say(&sound, "fn nothing() {}")[0]
                .contains("every answer they give is silent"),
            "windows with nothing to call were not reported"
        );
    }

    /// The body of one function this file declares, bounded by the next
    /// top-level `fn` or `pub fn`, the same way
    /// `test_each_manager_window_hands_the_shared_loop_its_own_kind` above
    /// bounds a manager window's own function: a fixed-width window missed a
    /// long function's own wiring and reported the right thing as missing.
    fn body_of<'a>(windows: &'a str, signature: &str) -> &'a str {
        let start = windows
            .find(signature)
            .unwrap_or_else(|| panic!("{signature} is not in this file"));
        let after = &windows[start + signature.len()..];
        let end = ["\npub fn ", "\nfn "]
            .iter()
            .filter_map(|marker| after.find(marker))
            .min()
            .unwrap_or(after.len());
        &windows[start..start + signature.len() + end]
    }

    /// Whether `needle` sits on a line of `haystack` that a `//` comment has
    /// not swallowed.
    ///
    /// `str::contains` cannot tell a live call from a commented-out one,
    /// because a line commented out with `// delete_selected(...)` still
    /// holds the call's exact text as a literal substring.
    /// `tests/theme_reach.rs` draws the same line, for the same reason, over
    /// a file this test cannot reach from inside `src/**` (see that file's
    /// own comment on why a live wxWidgets check cannot live here instead).
    fn appears_live(haystack: &str, needle: &str) -> bool {
        haystack.lines().any(|line| {
            line.find(needle)
                .is_some_and(|at| !line[..at].contains("//"))
        })
    }

    /// What a Delete button gets wrong when it still ends this dialog's own
    /// modal loop to do work that never needed to leave it.
    ///
    /// `end_modal` immediately followed by another `show_modal()` on the same
    /// dialog, with nothing yielded to the Windows message pump in between,
    /// is how NVDA lost both a button's own announcement and the dialog
    /// reappearing: a live run against Account Manager's Sign In Again heard
    /// neither, only NVDA's own generic "Wixen Mail, unavailable". Delete
    /// never opens a nested dialog, so it never needed to end this one
    /// either; Add and Edit do, and keep `end_modal`.
    fn what_a_delete_button_gets_wrong(function_body: &str, extracted_fn: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        let Some((_, after)) = function_body.split_once("del_btn.on_click({") else {
            return vec!["no Delete button is wired in this function".to_string()];
        };
        let click_body = &after[..after.find("\n    });").unwrap_or(after.len())];
        if !appears_live(click_body, extracted_fn) {
            wrong.push(format!(
                "Delete's own click never calls {extracted_fn}, so its work happens nowhere \
                 a test can reach it"
            ));
        }
        if appears_live(click_body, "end_modal(") {
            wrong.push(
                "Delete still ends this dialog's own modal loop, the round-trip NVDA loses \
                 the next announcement to"
                    .to_string(),
            );
        }
        wrong
    }

    #[test]
    fn test_delete_answers_from_its_own_click_and_never_ends_the_modal_loop() {
        // Structural, not behavioural: this reads the source rather than
        // clicking a real button, because nothing in this crate can fire a
        // real wx click event and observe live NVDA output inside a fast
        // unit test. `tests/manager_delete_stays_open.rs` proves the logic
        // in `delete_selected` and `delete_selected_contact` runs correctly
        // against real widgets; this proves each is actually wired to the
        // button that used to `end_modal` instead of calling it.
        let windows = the_manager_windows();
        for (function, extracted_fn) in [
            ("fn run_manager_loop", "delete_selected("),
            (
                "fn build_contact_manager_dialog",
                "delete_selected_contact(",
            ),
        ] {
            let body = body_of(&windows, function);
            let wrong = what_a_delete_button_gets_wrong(body, extracted_fn);
            assert!(wrong.is_empty(), "{function}: {}", wrong.join("\n  "));
        }

        // Step 4 of the fix: once end_modal is never called for Delete,
        // show_modal() can never return with its own ID, so the arm that
        // used to handle it is dead code, not defensive code worth keeping.
        for function in ["fn run_manager_loop", "fn show_contact_manager_dialog"] {
            let body = body_of(&windows, function);
            assert!(
                !body.contains("r if r == ID_MGR_DELETE"),
                "{function} still matches on ID_MGR_DELETE in its own modal loop, which \
                 show_modal() can never return now that Delete never calls end_modal"
            );
        }
    }

    #[test]
    fn test_the_delete_button_check_can_tell_the_two_apart() {
        // Proving the measurement, the same way
        // test_the_manager_check_can_tell_the_two_apart above does for the
        // shared status line: a source read that finds nothing passes, and
        // from outside that is indistinguishable from one that finds
        // everything.
        let sound = "del_btn.on_click({\n\
            \x20   let state = state.clone();\n\
            \x20   move |_| {\n\
            \x20       delete_selected(&state, &list, &status_text, &a11y, &kind, populate, name_fn);\n\
            \x20   }\n\
            \x20});\n";
        assert!(
            what_a_delete_button_gets_wrong(sound, "delete_selected(").is_empty(),
            "a Delete that only calls the extracted function was reported as wrong"
        );

        let still_ends_modal = "del_btn.on_click({\n\
            \x20   let d = *dialog;\n\
            \x20   move |_| {\n\
            \x20       delete_selected(&state, &list, &status_text, &a11y, &kind, populate, name_fn);\n\
            \x20       d.end_modal(ID_MGR_DELETE);\n\
            \x20   }\n\
            \x20});\n";
        let wrong = what_a_delete_button_gets_wrong(still_ends_modal, "delete_selected(");
        assert!(
            wrong.iter().any(|w| w.contains("round-trip")),
            "a Delete that still ends this dialog's modal loop was not reported: {wrong:?}"
        );

        // Commenting the real call out, not deleting it, is what this
        // project's own "prove the measurement" convention asks for: a check
        // built on scanning source text can be fooled by a call that is
        // still there in letters but never runs.
        let commented_out = "del_btn.on_click({\n\
            \x20   move |_| {\n\
            \x20       // delete_selected(&state, &list, &status_text, &a11y, &kind, populate, name_fn);\n\
            \x20   }\n\
            \x20});\n";
        let wrong = what_a_delete_button_gets_wrong(commented_out, "delete_selected(");
        assert!(
            wrong.iter().any(|w| w.contains("never calls")),
            "a call commented out rather than deleted was not reported: {wrong:?}"
        );

        let no_button = "close_btn.on_click({\n    move |_| {}\n});\n";
        assert!(
            what_a_delete_button_gets_wrong(no_button, "delete_selected(")[0]
                .contains("no Delete button"),
            "a function with no Delete button at all was not reported"
        );

        // And that check is awake on the real file, not only on the samples
        // above: it fires on sabotaged text, so it had better still see the
        // real, unsabotaged wiring as sound.
        let windows = the_manager_windows();
        assert!(
            appears_live(body_of(&windows, "fn run_manager_loop"), "delete_selected("),
            "run_manager_loop no longer calls delete_selected live, so the check above is \
             asleep on the real file"
        );
    }

    /// Somebody with two addresses and two numbers, which is the ordinary
    /// shape of a contact and the shape the search used to half ignore.
    fn a_contact_with_two_of_everything() -> ContactEntry {
        ContactEntry {
            id: "c1".to_string(),
            name: "Grace Hopper".to_string(),
            given_name: "Grace".to_string(),
            family_name: "Hopper".to_string(),
            nickname: String::new(),
            company: "Navy".to_string(),
            department: String::new(),
            job_title: String::new(),
            emails: vec![
                EmailItem {
                    label: "Personal".to_string(),
                    address: "grace@example.com".to_string(),
                },
                EmailItem {
                    label: "Work".to_string(),
                    address: "g.hopper@navy.example".to_string(),
                },
            ],
            phones: vec![
                PhoneItem {
                    label: "Home".to_string(),
                    number: "555 0100".to_string(),
                },
                PhoneItem {
                    label: "Mobile".to_string(),
                    number: "555 0101".to_string(),
                },
            ],
            addresses: Vec::new(),
            birthday: String::new(),
            website: String::new(),
            relationship: String::new(),
            notes: String::new(),
            custom_fields: Vec::new(),
            avatar_url: String::new(),
            favorite: false,
        }
    }

    #[test]
    fn test_a_contact_is_found_by_an_address_that_is_not_her_first() {
        // The address you have for somebody is the address you would type,
        // and it is as likely to be her work one as her personal one. Only
        // the first was searched, so typing the one you have found nobody and
        // the list said there was no such person.
        assert!(worth_showing(
            &a_contact_with_two_of_everything(),
            "g.hopper@navy.example"
        ));
    }

    #[test]
    fn test_a_contact_is_found_by_a_number_that_is_not_her_first() {
        assert!(worth_showing(&a_contact_with_two_of_everything(), "0101"));
    }

    #[test]
    fn test_a_contact_is_found_by_an_address_typed_in_capitals() {
        assert!(worth_showing(
            &a_contact_with_two_of_everything(),
            "Grace@Example.com"
        ));
    }

    #[test]
    fn test_somebody_the_search_does_not_name_is_left_out() {
        // The other direction. A filter that shows everybody is not a filter.
        assert!(!worth_showing(
            &a_contact_with_two_of_everything(),
            "lovelace"
        ));
    }

    #[test]
    fn test_an_empty_search_box_shows_everybody() {
        assert!(worth_showing(&a_contact_with_two_of_everything(), ""));
        assert!(worth_showing(&a_contact_with_two_of_everything(), "   "));
    }

    #[test]
    fn test_address_labels_us() {
        let (region, code) = get_address_field_labels("United States");
        assert!(region.contains("ate"));
        assert!(code.contains("ZIP"));
    }

    #[test]
    fn test_address_labels_uk() {
        let (region, code) = get_address_field_labels("United Kingdom");
        assert!(region.contains("unty"));
        assert!(code.contains("Postcode"));
    }

    #[test]
    fn test_address_labels_japan() {
        let (region, code) = get_address_field_labels("Japan");
        assert!(region.contains("fecture"));
        assert!(code.contains("Postal"));
    }

    #[test]
    fn test_address_labels_germany() {
        let (region, code) = get_address_field_labels("Germany");
        assert!(region.contains("Land"));
        assert!(code.contains("PLZ"));
    }

    #[test]
    fn test_address_labels_ireland() {
        let (region, code) = get_address_field_labels("Ireland");
        assert!(region.contains("unty"));
        assert!(code.contains("Eircode"));
    }

    #[test]
    fn test_address_labels_unknown_country_uses_default() {
        let (region, code) = get_address_field_labels("Narnia");
        assert!(region.contains("Province"));
        assert!(code.contains("Postal"));
    }

    #[test]
    fn test_address_labels_all_countries_return_non_empty() {
        let countries = [
            "United States",
            "United Kingdom",
            "Canada",
            "Australia",
            "Japan",
            "Germany",
            "Austria",
            "Switzerland",
            "France",
            "Brazil",
            "India",
            "South Korea",
            "China",
            "Italy",
            "Spain",
            "Mexico",
            "Ireland",
            "Netherlands",
        ];
        for country in &countries {
            let (region, code) = get_address_field_labels(country);
            assert!(!region.is_empty(), "Empty region for {}", country);
            assert!(!code.is_empty(), "Empty code for {}", country);
        }
    }
}

#[cfg(test)]
mod what_a_rule_stores_for_a_pattern_nothing_compares {
    use super::*;

    /// The words for a way of matching, so the tests read as the dialog does.
    fn said(stored: &str) -> &'static str {
        the_words_for_a_way_of_matching(stored).expect("every way of matching has words")
    }

    #[test]
    fn test_a_way_of_matching_that_reads_no_pattern_stores_none() {
        // The box is disabled for these four, and a disabled box still holds
        // whatever was typed before the Match Type was changed. Storing that
        // leaves a rule carrying a pattern nothing ever compares against, and
        // anything that describes the rule in words reads it out as though it
        // meant something.
        for way in ["is_empty", "is_not_empty", "is_true", "is_false"] {
            assert_eq!(
                the_pattern_to_store(said(way), "invoice"),
                "",
                "a rule matching by {way} kept a pattern it never reads"
            );
        }
    }

    #[test]
    fn test_a_way_of_matching_that_reads_the_pattern_stores_what_was_typed() {
        for way in ["contains", "not_contains", "equals", "regex"] {
            assert_eq!(
                the_pattern_to_store(said(way), "invoice"),
                "invoice",
                "a rule matching by {way} lost the pattern it compares against"
            );
        }
    }

    #[test]
    fn test_words_nothing_is_called_keep_what_was_typed() {
        // Nothing chosen yet, or a rule written by a later version. Emptying
        // the pattern on a question this build could not answer would throw
        // away a rule's contents on the way through a dialog that only opened
        // to change its name.
        assert_eq!(the_pattern_to_store("", "invoice"), "invoice");
        assert_eq!(
            the_pattern_to_store("sounds like", "invoice"),
            "invoice",
            "a way of matching this build has never heard of had its pattern emptied"
        );
    }
}

#[cfg(test)]
mod what_the_condition_editor_refuses_and_what_it_opens_on {
    use super::*;

    /// The words for a way of matching, so the tests read as the dialog does.
    fn said(stored: &str) -> &'static str {
        the_words_for_a_way_of_matching(stored).expect("every way of matching has words")
    }

    #[test]
    fn test_a_condition_is_refused_only_where_the_pattern_would_be_compared() {
        // Every one of the eleven, both ways round, because this switches on a
        // string and the families with no test are the ones mutation testing
        // has found here before.
        for way in A_WAY_A_RULE_MAY_MATCH {
            let refused = what_a_condition_still_needs(said(way), "").is_some();
            if a_way_of_matching_compares_against_nothing(way) {
                assert!(
                    !refused,
                    "a condition matching by {way} was refused for an empty pattern, and the \
                     box it is being asked to fill in is switched off"
                );
            } else {
                assert!(
                    refused,
                    "a condition matching by {way} saved with nothing to compare against, so \
                     it would match every message or none"
                );
            }
        }
    }

    #[test]
    fn test_a_condition_with_something_typed_in_is_never_refused() {
        for way in A_WAY_A_RULE_MAY_MATCH {
            assert_eq!(
                what_a_condition_still_needs(said(way), "invoice"),
                None,
                "a condition matching by {way} was refused with a pattern typed into it"
            );
        }
        // Nothing chosen yet, or a way of matching written by a later version.
        assert_eq!(what_a_condition_still_needs("", "invoice"), None);
    }

    #[test]
    fn test_a_new_condition_opens_on_a_field_and_a_way_of_matching_that_really_exist() {
        // A default naming something the lists do not offer selects nothing,
        // which is the accident this constant exists to stop: the dialog opens
        // with an unfilled combo box and OK stores the empty string.
        let (field, way) = WHAT_A_NEW_CONDITION_ASKS_FIRST;
        assert!(
            the_words_for_a_field(field).is_some(),
            "a new condition opens on {field:?}, and no field is called that"
        );
        assert!(
            the_words_for_a_way_of_matching(way).is_some(),
            "a new condition opens on {way:?}, and no way of matching is called that"
        );
    }
}

#[cfg(test)]
mod where_the_row_cursor_lands_after_a_delete {
    use super::*;

    #[test]
    fn test_the_row_that_moved_up_takes_the_place_of_the_one_removed() {
        // Delete rule twenty of fifty and the list is repainted with nothing
        // selected. Pressing Delete again then says "select a filter to
        // delete", which sounds like a refusal and is really the repaint, and
        // getting back to where you were means tabbing to the list and
        // arrowing down nineteen times.
        assert_eq!(the_row_to_select_after_removing(20, 49), Some(20));
    }

    #[test]
    fn test_removing_the_last_row_lands_on_the_new_last_one() {
        // There is no row twenty once twenty rows are left, so landing on the
        // index just removed would be landing on nothing.
        assert_eq!(the_row_to_select_after_removing(19, 19), Some(18));
    }

    #[test]
    fn test_removing_the_only_row_leaves_nothing_to_land_on() {
        assert_eq!(the_row_to_select_after_removing(0, 0), None);
    }
}

#[cfg(test)]
mod what_a_saved_searchs_condition_list_says_and_refuses {
    use super::*;

    fn asking(field: &str, match_type: &str, pattern: &str) -> Question {
        Question {
            field: field.to_string(),
            match_type: match_type.to_string(),
            pattern: pattern.to_string(),
            case_sensitive: false,
        }
    }

    #[test]
    fn test_a_condition_row_names_its_field_and_its_comparison_in_words() {
        // Against the words the two lists in the editor offer rather than
        // against a string written out here, so the row and the list somebody
        // chose from are pinned to one source. Those lists are already pinned
        // to the engine's constants in both directions, which makes this
        // transitive.
        let [looks_at, compares, against] =
            what_a_condition_row_says(&asking("body_plain", "contains", "invoice"));

        assert_eq!(
            looks_at,
            the_words_for_a_field("body_plain").expect("the message text has words")
        );
        assert_eq!(
            compares,
            the_words_for_a_way_of_matching("contains").expect("contains has words")
        );
        assert_eq!(against, "invoice");
    }

    #[test]
    fn test_every_field_and_every_comparison_reaches_a_row_in_its_own_words() {
        // Every arm both ways, because this reads a stored string and
        // `CLAUDE.md` records what happens to the families nobody thought to
        // test: four fields and six ways of matching had no test at all here
        // once, and mutation testing is what found them.
        for field in A_FIELD_A_RULE_MAY_NAME {
            let [looks_at, _, _] = what_a_condition_row_says(&asking(field, "contains", "x"));
            assert_eq!(
                looks_at,
                the_words_for_a_field(field).expect("every field a rule may name has words"),
                "the row for {field:?} read out its stored name"
            );
        }
        for way in A_WAY_A_RULE_MAY_MATCH {
            let [_, compares, _] = what_a_condition_row_says(&asking("subject", way, "x"));
            assert_eq!(
                compares,
                the_words_for_a_way_of_matching(way).expect("every way of matching has words"),
                "the row for {way:?} read out its stored name"
            );
        }
    }

    #[test]
    fn test_a_condition_this_build_has_no_words_for_is_still_a_row() {
        // A search written by a later version. Blanking what this build cannot
        // name would leave a row somebody cannot tell from the one above it,
        // and the one thing they can still do with it is take it out.
        let [looks_at, compares, against] =
            what_a_condition_row_says(&asking("sender_reputation", "sounds_like", "invoice"));

        assert_eq!(looks_at, "sender_reputation");
        assert_eq!(compares, "sounds_like");
        assert_eq!(against, "invoice");
    }

    #[test]
    fn test_case_sensitivity_is_on_the_row_when_it_is_on() {
        // Two conditions differing only in this would otherwise be two rows
        // nobody could tell apart.
        let mut fussy = asking("subject", "contains", "Invoice");
        fussy.case_sensitive = true;
        let [_, _, against] = what_a_condition_row_says(&fussy);

        assert!(
            against.to_lowercase().contains("case"),
            "a case sensitive condition read out exactly like one that is not: {against:?}"
        );
    }

    #[test]
    fn test_case_sensitivity_is_left_off_the_row_when_it_is_off() {
        // The ordinary case. A column or a clause carrying "no" on almost
        // every row is the other way to make a list unreadable.
        let [_, _, against] = what_a_condition_row_says(&asking("subject", "contains", "Invoice"));

        assert_eq!(against, "Invoice");
    }

    #[test]
    fn test_a_condition_with_nothing_to_compare_against_reads_without_a_gap() {
        // Four of the eleven ways of matching read no pattern at all, so the
        // third part is empty and joining it in leaves a sentence that trails
        // off. The row itself keeps the empty cell; the sentence does not.
        let said = a_condition_in_words(&asking("read", "is_true", ""));

        assert_eq!(said, said.trim(), "{said:?}");
        assert!(!said.contains("  "), "{said:?}");
        assert!(said.contains("Read"), "{said:?}");
    }

    #[test]
    fn test_a_saved_search_with_no_conditions_left_cannot_be_closed() {
        // The whole reason this window counts out loud. A search that asks
        // nothing takes the whole mailbox when its questions are joined with
        // Any and nothing at all when they are joined with All, and neither is
        // a search anybody wrote.
        let needed = what_a_condition_list_still_needs(&[]);

        assert!(
            needed.is_some(),
            "a saved search asking nothing at all was allowed out of the window"
        );
        assert!(
            needed.is_some_and(|said| said.to_lowercase().contains("condition")),
            "the refusal does not say what is missing: {needed:?}"
        );
    }

    #[test]
    fn test_a_saved_search_with_a_condition_closes() {
        assert_eq!(
            what_a_condition_list_still_needs(&[asking("subject", "contains", "invoice")]),
            None
        );
    }

    #[test]
    fn test_nothing_holds_the_other_manager_windows_open() {
        // A filter list or a tag list somebody emptied on purpose is a list
        // somebody emptied on purpose. Only a condition list has a floor.
        let no_filters: [FilterRule; 0] = [];
        assert_eq!(nothing_stops_this_closing(&no_filters), None);
    }
}
