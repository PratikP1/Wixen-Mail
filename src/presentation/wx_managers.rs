//! wxdragon Manager Dialogs
//!
//! Contact, Filter, Tag, and Signature managers sharing a generic modal loop.

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
    let add_btn = Button::builder(dialog).with_label("Add...").with_id(ID_MGR_ADD).build();
    let edit_btn = Button::builder(dialog).with_label("Edit...").with_id(ID_MGR_EDIT).build();
    let del_btn = Button::builder(dialog).with_label("Delete").with_id(ID_MGR_DELETE).build();
    let close_btn = Button::builder(dialog).with_label("Close").with_id(ID_OK).build();

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
// Contact Manager
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ContactEntry {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub notes: String,
    pub favorite: bool,
}

#[derive(Debug, Clone)]
pub enum ContactManagerAction {
    None,
    Updated(Vec<ContactEntry>),
}

pub fn show_contact_manager_dialog(parent: &Frame, contacts: &[ContactEntry]) -> ContactManagerAction {
    let (dialog, sizer, list, status) = make_shell(parent, "Contact Manager", 600, 450);

    let search_row = BoxSizer::builder(Orientation::Horizontal).build();
    let search_lbl = StaticText::builder(&dialog).with_label("Search:").build();
    let _search_f = TextCtrl::builder(&dialog).build();
    search_row.add(&search_lbl, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    search_row.add(&_search_f, 1, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add_sizer(&search_row, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 4);

    list.insert_column(0, "Name", ListColumnFormat::Left, 150);
    list.insert_column(1, "Email", ListColumnFormat::Left, 180);
    list.insert_column(2, "Phone", ListColumnFormat::Left, 110);
    list.insert_column(3, "Company", ListColumnFormat::Left, 120);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    let mut working = contacts.to_vec();
    let changed = run_manager_loop(
        &dialog, &sizer, &list, &status, &mut working,
        populate_contacts,
        |d| show_contact_edit(d, None),
        |d, c| show_contact_edit(d, Some(c)),
        |c| c.name.clone(),
    );

    if changed { ContactManagerAction::Updated(working) } else { ContactManagerAction::None }
}

fn populate_contacts(list: &ListCtrl, contacts: &[ContactEntry]) {
    list.delete_all_items();
    for (i, c) in contacts.iter().enumerate() {
        let idx = i as i64;
        let name = if c.favorite { format!("★ {}", c.name) } else { c.name.clone() };
        list.insert_item(idx, &name, None);
        list.set_item_text_by_column(idx, 1, &c.email);
        list.set_item_text_by_column(idx, 2, &c.phone);
        list.set_item_text_by_column(idx, 3, &c.company);
    }
}

fn show_contact_edit(parent: &Dialog, existing: Option<&ContactEntry>) -> Option<ContactEntry> {
    let title = if existing.is_some() { "Edit Contact" } else { "Add Contact" };
    let dlg = Dialog::builder(parent, title).with_size(420, 380).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(4).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    let name_f = add_field(&dlg, &fields, "Name:");
    let email_f = add_field(&dlg, &fields, "Email:");
    let phone_f = add_field(&dlg, &fields, "Phone:");
    let company_f = add_field(&dlg, &fields, "Company:");
    let notes_f = add_field(&dlg, &fields, "Notes:");

    let fav_label = StaticText::builder(&dlg).with_label("").build();
    let fav_check = CheckBox::builder(&dlg).with_label("Favorite").build();
    fields.add(&fav_label, 0, SizerFlag::All, 4);
    fields.add(&fav_check, 0, SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
    btn_row.add_spacer(0);
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 4);
    dlg.set_sizer(sizer, true);

    if let Some(c) = existing {
        name_f.set_value(&c.name);
        email_f.set_value(&c.email);
        phone_f.set_value(&c.phone);
        company_f.set_value(&c.company);
        notes_f.set_value(&c.notes);
        fav_check.set_value(c.favorite);
    }

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        Some(ContactEntry {
            id: existing.map(|c| c.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            email: email_f.get_value(),
            phone: phone_f.get_value(),
            company: company_f.get_value(),
            notes: notes_f.get_value(),
            favorite: fav_check.get_value(),
        })
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

    let name_f = add_field(&dlg, &fields, "Rule Name:");

    let field_label = StaticText::builder(&dlg).with_label("Match Field:").build();
    let field_choices: Vec<String> = ["subject", "from", "to", "cc", "body_plain", "date"]
        .iter().map(|s| s.to_string()).collect();
    let field_choice = Choice::builder(&dlg).with_choices(field_choices).build();
    fields.add(&field_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&field_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let match_label = StaticText::builder(&dlg).with_label("Match Type:").build();
    let match_choices: Vec<String> = ["contains", "not_contains", "equals", "starts_with", "ends_with", "regex"]
        .iter().map(|s| s.to_string()).collect();
    let match_choice = Choice::builder(&dlg).with_choices(match_choices).build();
    fields.add(&match_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&match_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let pattern_f = add_field(&dlg, &fields, "Pattern:");

    let cs_label = StaticText::builder(&dlg).with_label("").build();
    let cs_check = CheckBox::builder(&dlg).with_label("Case Sensitive").build();
    fields.add(&cs_label, 0, SizerFlag::All, 4);
    fields.add(&cs_check, 0, SizerFlag::All, 4);

    let action_label = StaticText::builder(&dlg).with_label("Action:").build();
    let action_choices: Vec<String> = ["mark_as_read", "mark_as_unread", "star", "delete", "move_to_folder", "add_tag"]
        .iter().map(|s| s.to_string()).collect();
    let action_choice = Choice::builder(&dlg).with_choices(action_choices).build();
    fields.add(&action_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&action_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let action_value_f = add_field(&dlg, &fields, "Action Value:");

    let en_label = StaticText::builder(&dlg).with_label("").build();
    let en_check = CheckBox::builder(&dlg).with_label("Enabled").build();
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

    let name_f = add_field(&dlg, &fields, "Tag Name:");

    let color_label = StaticText::builder(&dlg).with_label("Color:").build();
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

    let name_f = add_field(&dlg, &fields, "Name:");

    let def_label = StaticText::builder(&dlg).with_label("").build();
    let def_check = CheckBox::builder(&dlg).with_label("Default signature").build();
    fields.add(&def_label, 0, SizerFlag::All, 4);
    fields.add(&def_check, 0, SizerFlag::All, 4);

    sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 8);

    let plain_label = StaticText::builder(&dlg).with_label("Signature (plain text):").build();
    sizer.add(&plain_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    let content_f = TextCtrl::builder(&dlg)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();
    sizer.add(&content_f, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let html_label = StaticText::builder(&dlg).with_label("HTML version (optional):").build();
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
