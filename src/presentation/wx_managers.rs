//! wxdragon Manager Dialogs
//!
//! Contact, Filter, Tag, and Signature managers sharing a generic modal loop.

use std::cell::RefCell;
use std::rc::Rc;
use wxdragon::prelude::*;

// ── Shared Button IDs ──────────────────────────────────────────────────────

const ID_MGR_ADD: Id = ID_HIGHEST + 300;
const ID_MGR_EDIT: Id = ID_HIGHEST + 301;
const ID_MGR_DELETE: Id = ID_HIGHEST + 302;

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

/// Run the standard Add/Edit/Delete modal loop shared by all manager dialogs.
///
/// Returns `true` if any changes were made.
fn run_manager_loop<T: Clone>(
    dialog: &Dialog,
    main_sizer: &BoxSizer,
    list: &ListCtrl,
    status_text: &StaticText,
    working: &mut Vec<T>,
    populate: impl Fn(&ListCtrl, &[T]),
    add_fn: impl Fn(&Dialog) -> Option<T>,
    edit_fn: impl Fn(&Dialog, &T) -> Option<T>,
    name_fn: impl Fn(&T) -> String,
) -> bool {
    // Create and attach buttons
    let add_btn = Button::builder(dialog).with_label("&Add...").with_id(ID_MGR_ADD).build();
    let edit_btn = Button::builder(dialog).with_label("&Edit...").with_id(ID_MGR_EDIT).build();
    let del_btn = Button::builder(dialog).with_label("&Delete").with_id(ID_MGR_DELETE).build();
    let close_btn = Button::builder(dialog).with_label("&Close").with_id(ID_OK).build();

    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add(&add_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&edit_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&del_btn, 0, SizerFlag::All, 4);
    btn_sizer.add_spacer(16);
    btn_sizer.add(&close_btn, 0, SizerFlag::All, 4);

    main_sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    main_sizer.add(status_text, 0, SizerFlag::Expand | SizerFlag::All, 4);
    dialog.set_sizer(*main_sizer, true);

    add_btn.on_click({ let d = *dialog; move |_| { d.end_modal(ID_MGR_ADD); } });
    edit_btn.on_click({ let d = *dialog; move |_| { d.end_modal(ID_MGR_EDIT); } });
    del_btn.on_click({ let d = *dialog; move |_| { d.end_modal(ID_MGR_DELETE); } });
    close_btn.on_click({ let d = *dialog; move |_| { d.end_modal(ID_OK); } });

    populate(list, working);
    let mut changed = false;

    loop {
        match dialog.show_modal() {
            r if r == ID_MGR_ADD => {
                if let Some(item) = add_fn(dialog) {
                    working.push(item);
                    changed = true;
                    populate(list, working);
                    status_text.set_label("Added");
                }
            }
            r if r == ID_MGR_EDIT => {
                if let Some(idx) = get_selected(list) {
                    if let Some(updated) = edit_fn(dialog, &working[idx]) {
                        working[idx] = updated;
                        changed = true;
                        populate(list, working);
                        status_text.set_label("Updated");
                    }
                } else {
                    status_text.set_label("Select an item to edit");
                }
            }
            r if r == ID_MGR_DELETE => {
                if let Some(idx) = get_selected(list) {
                    let name = name_fn(&working[idx]);
                    working.remove(idx);
                    changed = true;
                    populate(list, working);
                    status_text.set_label(&format!("Deleted: {}", name));
                } else {
                    status_text.set_label("Select an item to delete");
                }
            }
            _ => break,
        }
    }
    changed
}

/// Create the standard manager dialog shell: dialog + sizer + list + status.
fn make_shell(parent: &Frame, title: &str, w: i32, h: i32) -> (Dialog, BoxSizer, ListCtrl, StaticText) {
    let dialog = Dialog::builder(parent, title)
        .with_size(w, h)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    let status = StaticText::builder(&dialog).with_label(" ").build();
    (dialog, sizer, list, status)
}

// ══════════════════════════════════════════════════════════════════════════════
// Contact Manager — Comprehensive Google Contacts-style fields
// ══════════════════════════════════════════════════════════════════════════════

/// Phone number with type label
#[derive(Debug, Clone)]
pub struct PhoneItem {
    pub label: String,
    pub number: String,
}

/// Email address with type label
#[derive(Debug, Clone)]
pub struct EmailItem {
    pub label: String,
    pub address: String,
}

/// Structured physical address with type label
#[derive(Debug, Clone)]
pub struct AddressItem {
    pub label: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

/// User-defined custom field
#[derive(Debug, Clone)]
pub struct CustomFieldItem {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ContactEntry {
    pub id: String,
    // ── Name & Identity ─────────────────────────────────────────────────
    pub name: String,
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
        self.emails.first().map(|e| e.address.as_str()).unwrap_or("")
    }
    /// Primary phone (first in list, or empty)
    pub fn primary_phone(&self) -> &str {
        self.phones.first().map(|p| p.number.as_str()).unwrap_or("")
    }
}

#[derive(Debug, Clone)]
pub enum ContactManagerAction {
    None,
    Updated(Vec<ContactEntry>),
}

// ── Label constants for dropdowns ────────────────────────────────────────────

const EMAIL_LABELS: &[&str] = &["Personal", "Work", "Other"];
const PHONE_LABELS: &[&str] = &["Mobile", "Home", "Work", "Work Fax", "Home Fax", "Pager", "Other"];
const ADDRESS_LABELS: &[&str] = &["Home", "Work", "Other"];

// ── Country Data ────────────────────────────────────────────────────────────

/// Comprehensive country list for address entry (alphabetical)
const COUNTRIES: &[&str] = &[
    "Afghanistan", "Albania", "Algeria", "Andorra", "Angola",
    "Argentina", "Armenia", "Australia", "Austria", "Azerbaijan",
    "Bahamas", "Bahrain", "Bangladesh", "Barbados", "Belarus",
    "Belgium", "Belize", "Bolivia", "Bosnia and Herzegovina", "Botswana",
    "Brazil", "Brunei", "Bulgaria", "Cambodia", "Cameroon",
    "Canada", "Chile", "China", "Colombia", "Costa Rica",
    "Croatia", "Cuba", "Cyprus", "Czech Republic", "Denmark",
    "Dominican Republic", "Ecuador", "Egypt", "El Salvador", "Estonia",
    "Ethiopia", "Finland", "France", "Georgia", "Germany",
    "Ghana", "Greece", "Guatemala", "Honduras", "Hong Kong",
    "Hungary", "Iceland", "India", "Indonesia", "Iran",
    "Iraq", "Ireland", "Israel", "Italy", "Jamaica",
    "Japan", "Jordan", "Kazakhstan", "Kenya", "Kuwait",
    "Latvia", "Lebanon", "Libya", "Lithuania", "Luxembourg",
    "Malaysia", "Mexico", "Moldova", "Monaco", "Mongolia",
    "Morocco", "Mozambique", "Myanmar", "Nepal", "Netherlands",
    "New Zealand", "Nicaragua", "Nigeria", "North Korea", "Norway",
    "Oman", "Pakistan", "Panama", "Paraguay", "Peru",
    "Philippines", "Poland", "Portugal", "Qatar", "Romania",
    "Russia", "Saudi Arabia", "Senegal", "Serbia", "Singapore",
    "Slovakia", "Slovenia", "South Africa", "South Korea", "Spain",
    "Sri Lanka", "Sudan", "Sweden", "Switzerland", "Syria",
    "Taiwan", "Tanzania", "Thailand", "Tunisia", "Turkey",
    "Uganda", "Ukraine", "United Arab Emirates", "United Kingdom", "United States",
    "Uruguay", "Uzbekistan", "Venezuela", "Vietnam", "Yemen",
    "Zambia", "Zimbabwe",
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
fn get_address_field_labels(country: &str) -> (&'static str, &'static str) {
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

// ── Contact Manager — Custom Loop with Live Search ──────────────────────────

pub fn show_contact_manager_dialog(parent: &Frame, contacts: &[ContactEntry]) -> ContactManagerAction {
    let dialog = Dialog::builder(parent, "Contact Manager")
        .with_size(700, 500)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // ── Search row — first in tab order for accessibility ────────────
    let search_row = BoxSizer::builder(Orientation::Horizontal).build();
    let search_lbl = StaticText::builder(&dialog).with_label("&Search:").build();
    let search_f = TextCtrl::builder(&dialog).build();
    search_row.add(&search_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    search_row.add(&search_f, 1, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add_sizer(&search_row, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 4);

    // ── Contact list ────────────────────────────────────────────────
    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    list.insert_column(0, "Name", ListColumnFormat::Left, 160);
    list.insert_column(1, "Email", ListColumnFormat::Left, 200);
    list.insert_column(2, "Phone", ListColumnFormat::Left, 130);
    list.insert_column(3, "Company", ListColumnFormat::Left, 140);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // ── Buttons ─────────────────────────────────────────────────────
    let add_btn = Button::builder(&dialog).with_label("&Add...").with_id(ID_MGR_ADD).build();
    let edit_btn = Button::builder(&dialog).with_label("&Edit...").with_id(ID_MGR_EDIT).build();
    let del_btn = Button::builder(&dialog).with_label("&Delete").with_id(ID_MGR_DELETE).build();
    let close_btn = Button::builder(&dialog).with_label("&Close").with_id(ID_OK).build();
    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add(&add_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&edit_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&del_btn, 0, SizerFlag::All, 4);
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

    // ── Live search — update results as user types ──────────────────
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
    add_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_MGR_ADD); } });
    edit_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_MGR_EDIT); } });
    del_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_MGR_DELETE); } });
    close_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_OK); } });

    // Set focus to search field for accessibility
    search_f.set_focus();

    // ── Modal loop ──────────────────────────────────────────────────
    let mut changed = false;
    loop {
        match dialog.show_modal() {
            r if r == ID_MGR_ADD => {
                if let Some(item) = show_contact_edit(&dialog, None) {
                    working.borrow_mut().push(item);
                    changed = true;
                    let query = search_f.get_value();
                    let w = working.borrow();
                    populate_contacts_filtered(&list, &w, &query, &mut index_map.borrow_mut());
                    status.set_label("Added");
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
                    if let Some(updated) = show_contact_edit(&dialog, Some(&existing)) {
                        working.borrow_mut()[working_idx] = updated;
                        changed = true;
                        let query = search_f.get_value();
                        let w = working.borrow();
                        populate_contacts_filtered(&list, &w, &query, &mut index_map.borrow_mut());
                        status.set_label("Updated");
                    }
                } else {
                    status.set_label("Select a contact to edit");
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
                    status.set_label(&format!("Deleted: {}", name));
                } else {
                    status.set_label("Select a contact to delete");
                }
            }
            _ => break,
        }
    }

    let result = working.borrow().clone();
    if changed { ContactManagerAction::Updated(result) } else { ContactManagerAction::None }
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
    let q = query.to_lowercase();
    for (i, c) in contacts.iter().enumerate() {
        if !q.is_empty() {
            let matches = c.name.to_lowercase().contains(&q)
                || c.primary_email().to_lowercase().contains(&q)
                || c.primary_phone().contains(&q)
                || c.company.to_lowercase().contains(&q)
                || c.nickname.to_lowercase().contains(&q);
            if !matches {
                continue;
            }
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

// ── Contact Edit — Tabbed Dialog ─────────────────────────────────────────────

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
    sizer.add(&lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    sizer.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    field
}

fn show_contact_edit(parent: &Dialog, existing: Option<&ContactEntry>) -> Option<ContactEntry> {
    let title = if existing.is_some() { "Edit Contact" } else { "Add Contact" };
    let dlg = Dialog::builder(parent, title)
        .with_size(560, 580)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let root = BoxSizer::builder(Orientation::Vertical).build();

    let notebook = Notebook::builder(&dlg).build();

    // ── Tab 1: Basic Info ────────────────────────────────────────────────
    // Accelerators: N(Name), K(Nickname), C(Company), D(Department),
    //   J(Job Title), B(Birthday), W(Website), R(Relationship), A(Avatar), F(Favorite)
    let basic_panel = Panel::builder(&notebook).build();
    let basic_sizer = BoxSizer::builder(Orientation::Vertical).build();
    let basic_fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    basic_fields.add_growable_col(1, 1);

    let name_f = add_panel_field(&basic_panel, &basic_fields, "&Name:");
    let nick_f = add_panel_field(&basic_panel, &basic_fields, "Nic&kname:");
    let company_f = add_panel_field(&basic_panel, &basic_fields, "&Company:");
    let dept_f = add_panel_field(&basic_panel, &basic_fields, "&Department:");
    let title_f = add_panel_field(&basic_panel, &basic_fields, "&Job Title:");
    let bday_f = add_panel_field(&basic_panel, &basic_fields, "&Birthday:");
    let web_f = add_panel_field(&basic_panel, &basic_fields, "&Website:");
    let rel_f = add_panel_field(&basic_panel, &basic_fields, "&Relationship:");
    let avatar_f = add_panel_field(&basic_panel, &basic_fields, "&Avatar URL:");

    let fav_spacer = StaticText::builder(&basic_panel).with_label("").build();
    let fav_check = CheckBox::builder(&basic_panel).with_label("&Favorite").build();
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
    let email_label = StaticText::builder(&contact_panel).with_label("Email Addresses:").build();
    contact_sizer.add(&email_label, 0, SizerFlag::Left | SizerFlag::Top | SizerFlag::Right, 8);
    let email_list = ListCtrl::builder(&contact_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    email_list.insert_column(0, "Type", ListColumnFormat::Left, 100);
    email_list.insert_column(1, "Address", ListColumnFormat::Left, 300);
    contact_sizer.add(&email_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let email_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_email_btn = Button::builder(&contact_panel).with_label("&Add Email...").with_id(ID_ADD_EMAIL).build();
    let del_email_btn = Button::builder(&contact_panel).with_label("&Remove Email").with_id(ID_DEL_EMAIL).build();
    email_btn_row.add(&add_email_btn, 0, SizerFlag::All, 4);
    email_btn_row.add(&del_email_btn, 0, SizerFlag::All, 4);
    contact_sizer.add_sizer(&email_btn_row, 0, SizerFlag::Left, 4);

    // Phone section
    let phone_label = StaticText::builder(&contact_panel).with_label("Phone Numbers:").build();
    contact_sizer.add(&phone_label, 0, SizerFlag::Left | SizerFlag::Top | SizerFlag::Right, 8);
    let phone_list = ListCtrl::builder(&contact_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    phone_list.insert_column(0, "Type", ListColumnFormat::Left, 100);
    phone_list.insert_column(1, "Number", ListColumnFormat::Left, 300);
    contact_sizer.add(&phone_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let phone_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_phone_btn = Button::builder(&contact_panel).with_label("Add &Phone...").with_id(ID_ADD_PHONE).build();
    let del_phone_btn = Button::builder(&contact_panel).with_label("Remo&ve Phone").with_id(ID_DEL_PHONE).build();
    phone_btn_row.add(&add_phone_btn, 0, SizerFlag::All, 4);
    phone_btn_row.add(&del_phone_btn, 0, SizerFlag::All, 4);
    contact_sizer.add_sizer(&phone_btn_row, 0, SizerFlag::Left, 4);

    contact_panel.set_sizer(contact_sizer, true);
    notebook.add_page(&contact_panel, "Email && Phone", false, None);

    // ── Tab 3: Addresses ─────────────────────────────────────────────────
    // Accelerators: A(Add Address), R(Remove Address)
    let addr_panel = Panel::builder(&notebook).build();
    let addr_sizer = BoxSizer::builder(Orientation::Vertical).build();
    let addr_label = StaticText::builder(&addr_panel).with_label("Physical Addresses:").build();
    addr_sizer.add(&addr_label, 0, SizerFlag::Left | SizerFlag::Top | SizerFlag::Right, 8);
    let addr_list = ListCtrl::builder(&addr_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    addr_list.insert_column(0, "Type", ListColumnFormat::Left, 80);
    addr_list.insert_column(1, "Street", ListColumnFormat::Left, 150);
    addr_list.insert_column(2, "City", ListColumnFormat::Left, 100);
    addr_list.insert_column(3, "State/Zip", ListColumnFormat::Left, 100);
    addr_list.insert_column(4, "Country", ListColumnFormat::Left, 80);
    addr_sizer.add(&addr_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let addr_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_addr_btn = Button::builder(&addr_panel).with_label("&Add Address...").with_id(ID_ADD_ADDR).build();
    let del_addr_btn = Button::builder(&addr_panel).with_label("&Remove Address").with_id(ID_DEL_ADDR).build();
    addr_btn_row.add(&add_addr_btn, 0, SizerFlag::All, 4);
    addr_btn_row.add(&del_addr_btn, 0, SizerFlag::All, 4);
    addr_sizer.add_sizer(&addr_btn_row, 0, SizerFlag::Left, 4);
    addr_panel.set_sizer(addr_sizer, true);
    notebook.add_page(&addr_panel, "Addresses", false, None);

    // ── Tab 4: Notes & Custom ────────────────────────────────────────────
    // Accelerators: N(Notes), A(Add Field), R(Remove Field)
    let notes_panel = Panel::builder(&notebook).build();
    let notes_sizer = BoxSizer::builder(Orientation::Vertical).build();
    let notes_label = StaticText::builder(&notes_panel).with_label("&Notes:").build();
    notes_sizer.add(&notes_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let notes_f = TextCtrl::builder(&notes_panel)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();
    notes_sizer.add(&notes_f, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let custom_label = StaticText::builder(&notes_panel).with_label("Custom Fields:").build();
    notes_sizer.add(&custom_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let custom_list = ListCtrl::builder(&notes_panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    custom_list.insert_column(0, "Label", ListColumnFormat::Left, 150);
    custom_list.insert_column(1, "Value", ListColumnFormat::Left, 300);
    notes_sizer.add(&custom_list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let custom_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let add_custom_btn = Button::builder(&notes_panel).with_label("&Add Field...").with_id(ID_ADD_CUSTOM).build();
    let del_custom_btn = Button::builder(&notes_panel).with_label("&Remove Field").with_id(ID_DEL_CUSTOM).build();
    custom_btn_row.add(&add_custom_btn, 0, SizerFlag::All, 4);
    custom_btn_row.add(&del_custom_btn, 0, SizerFlag::All, 4);
    notes_sizer.add_sizer(&custom_btn_row, 0, SizerFlag::Left, 4);
    notes_panel.set_sizer(notes_sizer, true);
    notebook.add_page(&notes_panel, "Notes && Custom", false, None);

    root.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // ── OK / Cancel ──────────────────────────────────────────────────────
    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
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
    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });
    add_email_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_ADD_EMAIL); } });
    del_email_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_DEL_EMAIL); } });
    add_phone_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_ADD_PHONE); } });
    del_phone_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_DEL_PHONE); } });
    add_addr_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_ADD_ADDR); } });
    del_addr_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_DEL_ADDR); } });
    add_custom_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_ADD_CUSTOM); } });
    del_custom_btn.on_click({ let d = dlg; move |_| { d.end_modal(ID_DEL_CUSTOM); } });

    // ── Modal loop (handle sub-list actions before OK/Cancel) ────────────
    loop {
        match dlg.show_modal() {
            r if r == ID_ADD_EMAIL => {
                if let Some(item) = show_email_sub_dialog(&dlg, None) {
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
                if let Some(item) = show_phone_sub_dialog(&dlg, None) {
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
                if let Some(item) = show_address_sub_dialog(&dlg, None) {
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
                if let Some(item) = show_custom_field_sub_dialog(&dlg, None) {
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
                    // Name is required — re-show dialog
                    continue;
                }
                return Some(ContactEntry {
                    id: existing.map(|c| c.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                    name: contact_name,
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
        list.set_item_text_by_column(idx, 3, &format!("{} {}", a.state, a.zip).trim().to_string());
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

fn show_email_sub_dialog(parent: &Dialog, _existing: Option<&EmailItem>) -> Option<EmailItem> {
    let dlg = Dialog::builder(parent, "Add Email Address").with_size(400, 200).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    // Accelerators: T(Type), A(Address) — first letters, no conflicts
    let type_lbl = StaticText::builder(&dlg).with_label("&Type:").build();
    let type_choices: Vec<String> = EMAIL_LABELS.iter().map(|s| s.to_string()).collect();
    let type_choice = Choice::builder(&dlg).with_choices(type_choices).build();
    type_choice.set_selection(0);
    fields.add(&type_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let addr_f = add_field(&dlg, &fields, "&Address:");
    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let addr = addr_f.get_value();
        if addr.trim().is_empty() { return None; }
        Some(EmailItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            address: addr,
        })
    } else {
        None
    }
}

fn show_phone_sub_dialog(parent: &Dialog, _existing: Option<&PhoneItem>) -> Option<PhoneItem> {
    let dlg = Dialog::builder(parent, "Add Phone Number").with_size(400, 200).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    // Accelerators: T(Type), N(Number) — first letters, no conflicts
    let type_lbl = StaticText::builder(&dlg).with_label("&Type:").build();
    let type_choices: Vec<String> = PHONE_LABELS.iter().map(|s| s.to_string()).collect();
    let type_choice = Choice::builder(&dlg).with_choices(type_choices).build();
    type_choice.set_selection(0);
    fields.add(&type_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let num_f = add_field(&dlg, &fields, "&Number:");
    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let num = num_f.get_value();
        if num.trim().is_empty() { return None; }
        Some(PhoneItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            number: num,
        })
    } else {
        None
    }
}

fn show_address_sub_dialog(parent: &Dialog, _existing: Option<&AddressItem>) -> Option<AddressItem> {
    let dlg = Dialog::builder(parent, "Add Address").with_size(440, 380).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    // ── Country dropdown FIRST — drives field labels ─────────────────
    // Accelerators: C(Country), T(Type), S(Street), I(City),
    //   region and code labels set dynamically by get_address_field_labels()
    let country_lbl = StaticText::builder(&dlg).with_label("&Country:").build();
    let country_choices: Vec<String> = COUNTRIES.iter().map(|s| s.to_string()).collect();
    let country_choice = Choice::builder(&dlg).with_choices(country_choices).build();
    // Default to system locale country
    let default_country = get_default_country();
    select_choice_by_string(&country_choice, default_country);
    fields.add(&country_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&country_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // ── Address type ─────────────────────────────────────────────────
    let type_lbl = StaticText::builder(&dlg).with_label("&Type:").build();
    let type_choices: Vec<String> = ADDRESS_LABELS.iter().map(|s| s.to_string()).collect();
    let type_choice = Choice::builder(&dlg).with_choices(type_choices).build();
    type_choice.set_selection(0);
    fields.add(&type_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // ── Address fields ───────────────────────────────────────────────
    let street_f = add_field(&dlg, &fields, "&Street:");
    let city_f = add_field(&dlg, &fields, "C&ity:");

    // Region and code labels are dynamic — set based on selected country
    let (initial_region_label, initial_code_label) = get_address_field_labels(default_country);

    let region_lbl = StaticText::builder(&dlg).with_label(initial_region_label).build();
    let region_f = TextCtrl::builder(&dlg).build();
    fields.add(&region_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&region_f, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let code_lbl = StaticText::builder(&dlg).with_label(initial_code_label).build();
    let code_f = TextCtrl::builder(&dlg).build();
    fields.add(&code_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&code_f, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // ── Country change handler — update region/code labels ───────────
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
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let street = street_f.get_value();
        let city = city_f.get_value();
        // Allow at least street or city
        if street.trim().is_empty() && city.trim().is_empty() { return None; }
        Some(AddressItem {
            label: get_choice_string(&type_choice).unwrap_or_else(|| "Other".to_string()),
            street,
            city,
            state: region_f.get_value(),
            zip: code_f.get_value(),
            country: get_choice_string(&country_choice).unwrap_or_else(|| default_country.to_string()),
        })
    } else {
        None
    }
}

fn show_custom_field_sub_dialog(parent: &Dialog, _existing: Option<&CustomFieldItem>) -> Option<CustomFieldItem> {
    let dlg = Dialog::builder(parent, "Add Custom Field").with_size(400, 200).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    // Accelerators: L(Label), V(Value) — first letters, no conflicts
    let label_f = add_field(&dlg, &fields, "&Label:");
    let value_f = add_field(&dlg, &fields, "&Value:");
    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    btn_row.add_spacer(0);
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let label = label_f.get_value();
        let value = value_f.get_value();
        if label.trim().is_empty() { return None; }
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

pub fn show_filter_manager_dialog(parent: &Frame, rules: &[FilterRule]) -> FilterManagerAction {
    let (dialog, sizer, list, status) = make_shell(parent, "Filter Manager", 650, 450);

    list.insert_column(0, "Name", ListColumnFormat::Left, 130);
    list.insert_column(1, "Condition", ListColumnFormat::Left, 220);
    list.insert_column(2, "Action", ListColumnFormat::Left, 150);
    list.insert_column(3, "Status", ListColumnFormat::Centre, 70);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = rules.to_vec();
    let changed = run_manager_loop(
        &dialog, &sizer, &list, &status, &mut working,
        populate_filters,
        |d| show_filter_edit(d, None),
        |d, r| show_filter_edit(d, Some(r)),
        |r| r.name.clone(),
    );

    if changed { FilterManagerAction::Updated(working) } else { FilterManagerAction::None }
}

fn populate_filters(list: &ListCtrl, rules: &[FilterRule]) {
    list.delete_all_items();
    for (i, r) in rules.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &r.name, None);
        list.set_item_text_by_column(idx, 1, &format!("{} {} '{}'", r.field, r.match_type, r.pattern));
        let action = if r.action_value.is_empty() { r.action_type.clone() } else { format!("{} ({})", r.action_type, r.action_value) };
        list.set_item_text_by_column(idx, 2, &action);
        list.set_item_text_by_column(idx, 3, if r.enabled { "Active" } else { "Disabled" });
    }
}

fn show_filter_edit(parent: &Dialog, existing: Option<&FilterRule>) -> Option<FilterRule> {
    let title = if existing.is_some() { "Edit Filter Rule" } else { "Add Filter Rule" };
    let dlg = Dialog::builder(parent, title).with_size(480, 440).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    // Accelerators: N(Name), F(Field), T(Type), P(Pattern), C(Case),
    //   A(Action), V(Value), E(Enabled) — all first letters
    let name_f = add_field(&dlg, &fields, "Rule &Name:");

    let field_label = StaticText::builder(&dlg).with_label("Match &Field:").build();
    let field_choices: Vec<String> = ["subject", "from", "to", "cc", "body_plain", "date"]
        .iter().map(|s| s.to_string()).collect();
    let field_choice = Choice::builder(&dlg).with_choices(field_choices).build();
    fields.add(&field_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&field_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let match_label = StaticText::builder(&dlg).with_label("Match &Type:").build();
    let match_choices: Vec<String> = ["contains", "not_contains", "equals", "starts_with", "ends_with", "regex"]
        .iter().map(|s| s.to_string()).collect();
    let match_choice = Choice::builder(&dlg).with_choices(match_choices).build();
    fields.add(&match_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&match_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let pattern_f = add_field(&dlg, &fields, "&Pattern:");

    let cs_label = StaticText::builder(&dlg).with_label("").build();
    let cs_check = CheckBox::builder(&dlg).with_label("&Case Sensitive").build();
    fields.add(&cs_label, 0, SizerFlag::All, 4);
    fields.add(&cs_check, 0, SizerFlag::All, 4);

    let action_label = StaticText::builder(&dlg).with_label("&Action:").build();
    let action_choices: Vec<String> = ["mark_as_read", "mark_as_unread", "star", "delete", "move_to_folder", "add_tag"]
        .iter().map(|s| s.to_string()).collect();
    let action_choice = Choice::builder(&dlg).with_choices(action_choices).build();
    fields.add(&action_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&action_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let action_value_f = add_field(&dlg, &fields, "Action &Value:");

    let en_label = StaticText::builder(&dlg).with_label("").build();
    let en_check = CheckBox::builder(&dlg).with_label("&Enabled").build();
    en_check.set_value(true);
    fields.add(&en_label, 0, SizerFlag::All, 4);
    fields.add(&en_check, 0, SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
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

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        Some(FilterRule {
            id: existing.map(|r| r.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
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
    ("Red", "#E53935"), ("Orange", "#FB8C00"), ("Yellow", "#FDD835"), ("Green", "#43A047"),
    ("Blue", "#1E88E5"), ("Purple", "#8E24AA"), ("Pink", "#D81B60"), ("Gray", "#757575"),
];

pub fn show_tag_manager_dialog(parent: &Frame, tags: &[TagEntry]) -> TagManagerAction {
    let (dialog, sizer, list, status) = make_shell(parent, "Tag Manager", 450, 400);

    list.insert_column(0, "Tag", ListColumnFormat::Left, 200);
    list.insert_column(1, "Color", ListColumnFormat::Left, 100);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = tags.to_vec();
    let changed = run_manager_loop(
        &dialog, &sizer, &list, &status, &mut working,
        populate_tags,
        |d| show_tag_edit(d, None),
        |d, t| show_tag_edit(d, Some(t)),
        |t| t.name.clone(),
    );

    if changed { TagManagerAction::Updated(working) } else { TagManagerAction::None }
}

fn populate_tags(list: &ListCtrl, tags: &[TagEntry]) {
    list.delete_all_items();
    for (i, t) in tags.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &t.name, None);
        let color_name = TAG_COLORS.iter()
            .find(|(_, hex)| *hex == t.color)
            .map(|(name, _)| *name)
            .unwrap_or(&t.color);
        list.set_item_text_by_column(idx, 1, color_name);
    }
}

fn show_tag_edit(parent: &Dialog, existing: Option<&TagEntry>) -> Option<TagEntry> {
    let title = if existing.is_some() { "Edit Tag" } else { "Add Tag" };
    let dlg = Dialog::builder(parent, title).with_size(350, 250).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    // Accelerators: N(Name), C(Color) — first letters, no conflicts
    let name_f = add_field(&dlg, &fields, "Tag &Name:");

    let color_label = StaticText::builder(&dlg).with_label("&Color:").build();
    let color_choices: Vec<String> = TAG_COLORS.iter().map(|(name, _)| name.to_string()).collect();
    let color_choice = Choice::builder(&dlg).with_choices(color_choices).build();
    color_choice.set_selection(0);
    fields.add(&color_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&color_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
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

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let color_idx = color_choice.get_selection().unwrap_or(0) as usize;
        let color = TAG_COLORS.get(color_idx).map(|(_, hex)| hex.to_string()).unwrap_or_else(|| "#1E88E5".to_string());
        Some(TagEntry {
            id: existing.map(|t| t.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
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

pub fn show_signature_manager_dialog(parent: &Frame, signatures: &[SignatureEntry]) -> SignatureManagerAction {
    let (dialog, sizer, list, status) = make_shell(parent, "Signature Manager", 550, 450);

    list.insert_column(0, "Name", ListColumnFormat::Left, 200);
    list.insert_column(1, "Default", ListColumnFormat::Centre, 80);
    list.insert_column(2, "Preview", ListColumnFormat::Left, 220);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = signatures.to_vec();
    let changed = run_manager_loop(
        &dialog, &sizer, &list, &status, &mut working,
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
                if saw_default { s.is_default = false; }
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
    let title = if existing.is_some() { "Edit Signature" } else { "Add Signature" };
    let dlg = Dialog::builder(parent, title).with_size(500, 420).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    // Accelerators: N(Name), D(Default), S(Signature/plain), H(HTML) — first letters
    let name_f = add_field(&dlg, &fields, "&Name:");

    let def_label = StaticText::builder(&dlg).with_label("").build();
    let def_check = CheckBox::builder(&dlg).with_label("&Default signature").build();
    fields.add(&def_label, 0, SizerFlag::All, 4);
    fields.add(&def_check, 0, SizerFlag::All, 4);

    sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 8);

    let plain_label = StaticText::builder(&dlg).with_label("&Signature (plain text):").build();
    sizer.add(&plain_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    let content_f = TextCtrl::builder(&dlg)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();
    sizer.add(&content_f, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let html_label = StaticText::builder(&dlg).with_label("&HTML version (optional):").build();
    sizer.add(&html_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    let html_f = TextCtrl::builder(&dlg).with_style(TextCtrlStyle::MultiLine).build();
    sizer.add(&html_f, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
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

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let html_val = html_f.get_value();
        Some(SignatureEntry {
            id: existing.map(|s| s.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            content_plain: content_f.get_value(),
            content_html: if html_val.trim().is_empty() { None } else { Some(html_val) },
            is_default: def_check.get_value(),
        })
    } else {
        None
    }
}
