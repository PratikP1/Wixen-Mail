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

use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::{name_from_label, set_accessible_name};
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
    a11y: &'a Accessibility,
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
/// Returns `true` if any changes were made.
fn run_manager_loop<T: Clone>(
    chrome: ManagerChrome<'_>,
    kind: &str,
    working: &mut Vec<T>,
    populate: impl Fn(&ListCtrl, &[T]),
    add_fn: impl Fn(&Dialog) -> Option<T>,
    edit_fn: impl Fn(&Dialog, &T) -> Option<T>,
    name_fn: impl Fn(&T) -> String,
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
        let d = *dialog;
        move |_| {
            d.end_modal(ID_MGR_DELETE);
        }
    });
    close_btn.on_click({
        let d = *dialog;
        move |_| {
            d.end_modal(ID_OK);
        }
    });

    populate(list, working);
    let mut changed = false;

    loop {
        match dialog.show_modal() {
            r if r == ID_MGR_ADD => {
                if let Some(item) = add_fn(dialog) {
                    let name = name_fn(&item);
                    working.push(item);
                    changed = true;
                    populate(list, working);
                    said_and_shown(
                        status_text,
                        a11y,
                        &manager_words::added(kind, &name),
                        Priority::Normal,
                    );
                }
            }
            r if r == ID_MGR_EDIT => {
                if let Some(idx) = get_selected(list) {
                    if let Some(edited) = edit_fn(dialog, &working[idx]) {
                        let name = name_fn(&edited);
                        working[idx] = edited;
                        changed = true;
                        populate(list, working);
                        said_and_shown(
                            status_text,
                            a11y,
                            &manager_words::updated(kind, &name),
                            Priority::Normal,
                        );
                    }
                } else {
                    said_and_shown(
                        status_text,
                        a11y,
                        &manager_words::nothing_selected(kind, "edit"),
                        Priority::High,
                    );
                }
            }
            r if r == ID_MGR_DELETE => {
                if let Some(idx) = get_selected(list) {
                    let name = name_fn(&working[idx]);
                    working.remove(idx);
                    changed = true;
                    populate(list, working);
                    said_and_shown(
                        status_text,
                        a11y,
                        &manager_words::deleted(kind, &name),
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
            _ => break,
        }
    }
    changed
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
pub fn make_shell(
    parent: &Frame,
    title: &str,
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
    set_accessible_name(&list, "Items");
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
/// shared state the live search and the Add/Edit/Delete handlers already
/// close over.
pub struct ContactManagerDialogHandles {
    pub dialog: Dialog,
    pub search: TextCtrl,
    pub list: ListCtrl,
    pub status: StaticText,
    working: Rc<RefCell<Vec<ContactEntry>>>,
    index_map: Rc<RefCell<Vec<usize>>>,
}

/// Build the Contact Manager's own list window without showing it.
///
/// Everything `show_contact_manager_dialog` used to do up to its own modal
/// loop, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
pub fn build_contact_manager_dialog(
    parent: &Frame,
    contacts: &[ContactEntry],
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
    del_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_MGR_DELETE);
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
    } = build_contact_manager_dialog(parent, contacts, palette);

    // ── Modal loop ──────────────────────────────────────────────────
    let mut changed = false;
    loop {
        match dialog.show_modal() {
            r if r == ID_MGR_ADD => {
                if let Some(item) = show_contact_edit(&dialog, None, palette) {
                    let name = item.name.clone();
                    working.borrow_mut().push(item);
                    changed = true;
                    let query = search_f.get_value();
                    let w = working.borrow();
                    populate_contacts_filtered(&list, &w, &query, &mut index_map.borrow_mut());
                    said_and_shown(
                        &status,
                        a11y,
                        &manager_words::added(manager_words::CONTACT, &name),
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
                    if let Some(edited) = show_contact_edit(&dialog, Some(&existing), palette) {
                        let name = edited.name.clone();
                        working.borrow_mut()[working_idx] = edited;
                        changed = true;
                        let query = search_f.get_value();
                        let w = working.borrow();
                        populate_contacts_filtered(&list, &w, &query, &mut index_map.borrow_mut());
                        said_and_shown(
                            &status,
                            a11y,
                            &manager_words::updated(manager_words::CONTACT, &name),
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
            r if r == ID_MGR_DELETE => {
                if let Some(sel) = get_selected(&list) {
                    let (working_idx, name) = {
                        let map = index_map.borrow();
                        let w = working.borrow();
                        match map.get(sel) {
                            Some(&idx) => (idx, w[idx].name.clone()),
                            None => continue,
                        }
                    };
                    working.borrow_mut().remove(working_idx);
                    changed = true;
                    let query = search_f.get_value();
                    let w = working.borrow();
                    populate_contacts_filtered(&list, &w, &query, &mut index_map.borrow_mut());
                    said_and_shown(
                        &status,
                        a11y,
                        &manager_words::deleted(manager_words::CONTACT, &name),
                        Priority::Normal,
                    );
                } else {
                    said_and_shown(
                        &status,
                        a11y,
                        &manager_words::nothing_selected(manager_words::CONTACT, "delete"),
                        Priority::High,
                    );
                }
            }
            r if r == ID_MGR_SYNC => {
                return ContactManagerAction::SyncRequested;
            }
            _ => break,
        }
    }

    let result = working.borrow().clone();
    if changed {
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
pub fn show_new_contact_dialog(parent: &Frame) -> Option<ContactEntry> {
    show_contact_edit(parent, None, theme::current_from_stored_config())
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
        .with_label("&Notes:")
        .build();
    notes_sizer.add(&notes_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let notes_f = TextCtrl::builder(&notes_panel)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();
    set_accessible_name(&notes_f, "Notes");
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
    ok.on_click({
        let d = dlg;
        move |_| {
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
        let d = dlg;
        move |_| {
            d.end_modal(ID_DEL_EMAIL);
        }
    });
    add_phone_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD_PHONE);
        }
    });
    del_phone_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_DEL_PHONE);
        }
    });
    add_addr_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD_ADDR);
        }
    });
    del_addr_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_DEL_ADDR);
        }
    });
    add_custom_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD_CUSTOM);
        }
    });
    del_custom_btn.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_DEL_CUSTOM);
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
    } = build_contact_edit_dialog(parent, existing, palette);

    // ── Modal loop (handle sub-list actions before OK/Cancel) ────────────
    loop {
        match dlg.show_modal() {
            r if r == ID_ADD_EMAIL => {
                if let Some(item) = show_email_sub_dialog(&dlg, None, palette) {
                    emails_data.borrow_mut().push(item);
                    refresh_email_list(&email_list, &emails_data.borrow());
                }
            }
            r if r == ID_DEL_EMAIL => {
                if let Some(idx) = get_selected(&email_list) {
                    emails_data.borrow_mut().remove(idx);
                    refresh_email_list(&email_list, &emails_data.borrow());
                }
            }
            r if r == ID_ADD_PHONE => {
                if let Some(item) = show_phone_sub_dialog(&dlg, None, palette) {
                    phones_data.borrow_mut().push(item);
                    refresh_phone_list(&phone_list, &phones_data.borrow());
                }
            }
            r if r == ID_DEL_PHONE => {
                if let Some(idx) = get_selected(&phone_list) {
                    phones_data.borrow_mut().remove(idx);
                    refresh_phone_list(&phone_list, &phones_data.borrow());
                }
            }
            r if r == ID_ADD_ADDR => {
                if let Some(item) = show_address_sub_dialog(&dlg, None, palette) {
                    addrs_data.borrow_mut().push(item);
                    refresh_addr_list(&addr_list, &addrs_data.borrow());
                }
            }
            r if r == ID_DEL_ADDR => {
                if let Some(idx) = get_selected(&addr_list) {
                    addrs_data.borrow_mut().remove(idx);
                    refresh_addr_list(&addr_list, &addrs_data.borrow());
                }
            }
            r if r == ID_ADD_CUSTOM => {
                if let Some(item) = show_custom_field_sub_dialog(&dlg, None, palette) {
                    custom_data.borrow_mut().push(item);
                    refresh_custom_list(&custom_list, &custom_data.borrow());
                }
            }
            r if r == ID_DEL_CUSTOM => {
                if let Some(idx) = get_selected(&custom_list) {
                    custom_data.borrow_mut().remove(idx);
                    refresh_custom_list(&custom_list, &custom_data.borrow());
                }
            }
            r if r == ID_OK => {
                let contact_name = name_f.get_value();
                if contact_name.trim().is_empty() {
                    // Name is required: re-show dialog
                    continue;
                }
                return Some(ContactEntry {
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
                });
            }
            _ => return None, // Cancel or close
        }
    }
}

// ── List refresh helpers ─────────────────────────────────────────────────────

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
        move |_| {
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

    if dlg.show_modal() == ID_OK {
        let addr = addr_f.get_value();
        if addr.trim().is_empty() {
            return None;
        }
        Some(EmailItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            address: addr,
        })
    } else {
        None
    }
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
        move |_| {
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

    if dlg.show_modal() == ID_OK {
        let num = num_f.get_value();
        if num.trim().is_empty() {
            return None;
        }
        Some(PhoneItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            number: num,
        })
    } else {
        None
    }
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
        move |_| {
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

    if dlg.show_modal() == ID_OK {
        let street = street_f.get_value();
        let city = city_f.get_value();
        // Allow at least street or city
        if street.trim().is_empty() && city.trim().is_empty() {
            return None;
        }
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
    }
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
        move |_| {
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

    if dlg.show_modal() == ID_OK {
        let label = label_f.get_value();
        let value = value_f.get_value();
        if label.trim().is_empty() {
            return None;
        }
        Some(CustomFieldItem { label, value })
    } else {
        None
    }
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
    let (dialog, sizer, list, status) = make_shell(
        parent,
        "Filter Manager",
        650,
        450,
        theme::current_from_stored_config(),
    );

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
            a11y,
        },
        manager_words::FILTER,
        &mut working,
        populate_filters,
        |d| show_filter_edit(d, None),
        |d, r| show_filter_edit(d, Some(r)),
        |r| r.name.clone(),
    );

    if changed {
        FilterManagerAction::Updated(working)
    } else {
        FilterManagerAction::None
    }
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

fn show_filter_edit(parent: &Dialog, existing: Option<&FilterRule>) -> Option<FilterRule> {
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
    let field_choices: Vec<String> = ["subject", "from", "to", "cc", "body_plain", "date"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let field_choice = Choice::builder(&dlg).with_choices(field_choices).build();
    set_accessible_name(&field_choice, "Match field");
    fields.add(
        &field_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&field_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let match_label = StaticText::builder(&dlg).with_label("Match &Type:").build();
    let match_choices: Vec<String> = [
        "contains",
        "not_contains",
        "equals",
        "starts_with",
        "ends_with",
        "regex",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let match_choice = Choice::builder(&dlg).with_choices(match_choices).build();
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
    let action_choices: Vec<String> = [
        "mark_as_read",
        "mark_as_unread",
        "star",
        "delete",
        "move_to_folder",
        "add_tag",
    ]
    .iter()
    .map(|s| s.to_string())
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
        select_choice_by_string(&field_choice, &r.field);
        select_choice_by_string(&match_choice, &r.match_type);
        pattern_f.set_value(&r.pattern);
        cs_check.set_value(r.case_sensitive);
        select_choice_by_string(&action_choice, &r.action_type);
        action_value_f.set_value(&r.action_value);
        en_check.set_value(r.enabled);
    }

    ok.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if dlg.show_modal() == ID_OK {
        Some(FilterRule {
            id: existing
                .map(|r| r.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            field: get_choice_string(&field_choice).unwrap_or_default(),
            match_type: get_choice_string(&match_choice).unwrap_or_default(),
            pattern: pattern_f.get_value(),
            case_sensitive: cs_check.get_value(),
            action_type: get_choice_string(&action_choice).unwrap_or_default(),
            action_value: action_value_f.get_value(),
            enabled: en_check.get_value(),
        })
    } else {
        None
    }
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
    let (dialog, sizer, list, status) = make_shell(
        parent,
        "Tag Manager",
        450,
        400,
        theme::current_from_stored_config(),
    );

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
            a11y,
        },
        manager_words::TAG,
        &mut working,
        populate_tags,
        |d| show_tag_edit(d, None),
        |d, t| show_tag_edit(d, Some(t)),
        |t| t.name.clone(),
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

fn show_tag_edit(parent: &Dialog, existing: Option<&TagEntry>) -> Option<TagEntry> {
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
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if dlg.show_modal() == ID_OK {
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
    }
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
    let (dialog, sizer, list, status) = make_shell(
        parent,
        "Signature Manager",
        550,
        450,
        theme::current_from_stored_config(),
    );

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
            a11y,
        },
        manager_words::SIGNATURE,
        &mut working,
        populate_sigs,
        |d| show_sig_edit(d, None),
        |d, s| show_sig_edit(d, Some(s)),
        |s| s.name.clone(),
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

fn show_sig_edit(parent: &Dialog, existing: Option<&SignatureEntry>) -> Option<SignatureEntry> {
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
        .with_label("&Signature (plain text):")
        .build();
    sizer.add(&plain_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    let content_f = TextCtrl::builder(&dlg)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();
    set_accessible_name(&content_f, "Signature, plain text");
    sizer.add(&content_f, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let html_label = StaticText::builder(&dlg)
        .with_label("&HTML version (optional):")
        .build();
    sizer.add(&html_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    let html_f = TextCtrl::builder(&dlg)
        .with_style(TextCtrlStyle::MultiLine)
        .build();
    set_accessible_name(&html_f, "Signature, HTML version");
    sizer.add(&html_f, 1, SizerFlag::Expand | SizerFlag::All, 8);

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
        if let Some(ref html) = s.content_html {
            html_f.set_value(html);
        }
        def_check.set_value(s.is_default);
    }

    ok.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if dlg.show_modal() == ID_OK {
        let html_val = html_f.get_value();
        Some(SignatureEntry {
            id: existing
                .map(|s| s.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            content_plain: content_f.get_value(),
            content_html: if html_val.trim().is_empty() {
                None
            } else {
                Some(html_val)
            },
            is_default: def_check.get_value(),
        })
    } else {
        None
    }
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
pub fn build_wait_for_an_answer_dialog(
    parent: &Frame,
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
pub fn wait_for_an_answer<T: 'static>(
    parent: &Frame,
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
    if dlg.show_modal() == ID_OK {
        list.get_selection().map(|chosen| chosen as usize)
    } else {
        None
    }
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
