//! Whether `delete_selected` and `delete_selected_contact` really do the
//! deleting, against real widgets, without a human closing a live modal.
//!
//! `wx_managers.rs`'s own tests can only read its source as text: nothing in
//! that file's own `#[cfg(test)]` module may build a live wxWidgets control
//! (see `tests/theme_reach.rs`'s file comment for why: every unit test in
//! `src/**` shares one test binary, wxWidgets supports exactly one
//! application per process, and that binary already has no live-widget test
//! in it to collide with). This file is a separate integration test binary,
//! so its own single `wxdragon::main` call below does not collide with
//! `tests/theme_reach.rs`'s either; each `tests/*.rs` file is its own
//! process.
//!
//! What this proves: given a real `ListCtrl` with a row selected and a real
//! `StaticText` line, Delete's own extracted logic removes the right row,
//! marks the state changed, repaints the list, and says the right sentence
//! (or, with nothing selected, says so and changes nothing). What it cannot
//! prove: that a real Delete button's real click reaches this function
//! (`wx_managers.rs`'s own wiring tests read the source for that) or that a
//! screen reader hears the sentence this writes to the line of text
//! (nothing here has ever been run against NVDA; see this crate's own
//! `docs/ALPHA_TESTING.md` and the calling task's own notes).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_managers::{
    self, ContactEntry, ManagerState, TagEntry, delete_selected, delete_selected_contact,
};
use wxdragon::prelude::*;

/// One outcome recorded rather than asserted immediately, the same shape
/// `tests/theme_reach.rs` uses for the same reason: a run inside
/// `wxdragon::main` cannot let a panic unwind through the C++ call stack
/// above it, so every check records a `(name, ok, detail)` triple and the
/// `#[test]` function asserts on the collected list afterward.
type Outcome = (&'static str, bool, String);

fn record(into: &mut Vec<Outcome>, name: &'static str, ok: bool, detail: impl Into<String>) {
    into.push((name, ok, detail.into()));
}

/// A tag with a name worth asserting on, and a colour that plays no part in
/// this test.
fn a_tag(name: &str) -> TagEntry {
    TagEntry {
        id: format!("tag-{name}"),
        name: name.to_string(),
        color: "#1E88E5".to_string(),
    }
}

/// Repaint a list of tags the same way `wx_managers::populate_tags` does:
/// one row per tag, named. Written here rather than reused, because the
/// real `populate_tags` is private to `wx_managers.rs` and this test only
/// needs enough of a repaint to prove `delete_selected` calls whatever it
/// was given.
fn populate_tag_rows(list: &ListCtrl, tags: &[TagEntry]) {
    list.delete_all_items();
    for (i, t) in tags.iter().enumerate() {
        list.insert_item(i as i64, &t.name, None);
    }
}

/// A contact with only the fields this test reads filled in meaningfully;
/// every other field is the empty value `ContactEntry` allows.
fn a_contact(id: &str, name: &str) -> ContactEntry {
    ContactEntry {
        id: id.to_string(),
        name: name.to_string(),
        given_name: String::new(),
        family_name: String::new(),
        nickname: String::new(),
        company: String::new(),
        department: String::new(),
        job_title: String::new(),
        emails: Vec::new(),
        phones: Vec::new(),
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

/// Select row `row` of `list` the way a person arrowing to it would leave
/// it: the one call `wxdragon::widgets::list_ctrl::ListCtrl::set_item_state`
/// exists for.
fn select_row(list: &ListCtrl, row: i64) {
    let selected = list.set_item_state(row, ListItemState::Selected, ListItemState::Selected);
    assert!(
        selected,
        "selecting row {row} of a freshly populated list failed"
    );
}

/// `delete_selected`, the Filter/Tag/Signature managers' shared logic, with
/// something selected: the row is gone from `state`, `state.changed` is
/// set, the list is repainted to match, and the line of text says which row
/// went.
fn check_delete_selected_removes_the_selected_row(
    frame: &Frame,
    a11y: &Accessibility,
    into: &mut Vec<Outcome>,
) {
    let (_dialog, _sizer, list, status) =
        wx_managers::make_shell(frame, "Tag Manager", 450, 400, None);
    list.insert_column(0, "Tag", ListColumnFormat::Left, 200);

    let tags = vec![a_tag("Blue"), a_tag("Green")];
    populate_tag_rows(&list, &tags);
    select_row(&list, 0);

    let state = Rc::new(RefCell::new(ManagerState {
        working: tags,
        changed: false,
    }));

    delete_selected(
        &state,
        &list,
        &status,
        a11y,
        "tag",
        populate_tag_rows,
        |t: &TagEntry| t.name.clone(),
    );

    let s = state.borrow();
    record(
        into,
        "delete_selected removes the selected row",
        s.working.len() == 1 && s.working[0].name == "Green",
        format!("working rows after delete: {:?}", s.working),
    );
    record(
        into,
        "delete_selected marks the state changed",
        s.changed,
        format!("changed = {}", s.changed),
    );
    drop(s);
    record(
        into,
        "delete_selected repaints the list to match",
        list.get_item_count() == 1,
        format!("list row count after delete: {}", list.get_item_count()),
    );
    record(
        into,
        "delete_selected says which row it deleted",
        status.get_label() == "Deleted the tag: Blue",
        format!("line of text after delete: {:?}", status.get_label()),
    );
}

/// `delete_selected` with nothing selected: nothing is removed, nothing is
/// marked changed, and the line of text asks for a selection instead of
/// reporting a deletion that never happened.
fn check_delete_selected_with_nothing_selected_changes_nothing(
    frame: &Frame,
    a11y: &Accessibility,
    into: &mut Vec<Outcome>,
) {
    let (_dialog, _sizer, list, status) =
        wx_managers::make_shell(frame, "Tag Manager", 450, 400, None);
    list.insert_column(0, "Tag", ListColumnFormat::Left, 200);

    let tags = vec![a_tag("Blue")];
    populate_tag_rows(&list, &tags);
    // Deliberately left unselected.

    let state = Rc::new(RefCell::new(ManagerState {
        working: tags,
        changed: false,
    }));

    delete_selected(
        &state,
        &list,
        &status,
        a11y,
        "tag",
        populate_tag_rows,
        |t: &TagEntry| t.name.clone(),
    );

    let s = state.borrow();
    record(
        into,
        "delete_selected with nothing selected removes nothing",
        s.working.len() == 1,
        format!("working rows: {:?}", s.working),
    );
    record(
        into,
        "delete_selected with nothing selected leaves changed false",
        !s.changed,
        format!("changed = {}", s.changed),
    );
    drop(s);
    record(
        into,
        "delete_selected with nothing selected asks for a selection",
        status.get_label() == "Select a tag to delete",
        format!("line of text: {:?}", status.get_label()),
    );
}

/// `delete_selected_contact`, the Contact Manager's own version: the same
/// four guarantees, against a real Contact Manager list window built
/// through the public `build_contact_manager_dialog`, with its own fresh
/// `working`/`index_map`/`changed` cells the way its own Delete button gets
/// them (this function does not read them back off the handles, which keep
/// them private on purpose; it takes them as its own arguments instead).
fn check_delete_selected_contact_removes_the_selected_row(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Vec<Outcome>,
) {
    let handles = wx_managers::build_contact_manager_dialog(frame, &[], a11y, None);
    let list = handles.list;
    let search = handles.search;
    let status = handles.status;

    // Seeded directly rather than through the private `populate_contacts_filtered`:
    // `delete_selected_contact` reads a contact's name from `working`, not
    // from the list's own displayed text, so a placeholder row is enough to
    // give `get_selected` something real to report.
    list.insert_item(0, "row 0", None);
    list.insert_item(1, "row 1", None);
    select_row(&list, 0);

    let working = Rc::new(RefCell::new(vec![
        a_contact("c1", "Grace Hopper"),
        a_contact("c2", "Ada Lovelace"),
    ]));
    let index_map = Rc::new(RefCell::new(vec![0usize, 1usize]));
    let changed = Rc::new(RefCell::new(false));

    delete_selected_contact(
        &list, &search, &status, a11y, &working, &index_map, &changed,
    );

    let w = working.borrow();
    record(
        into,
        "delete_selected_contact removes the selected contact",
        w.len() == 1 && w[0].name == "Ada Lovelace",
        format!(
            "working contacts after delete: {:?}",
            w.iter().map(|c| &c.name).collect::<Vec<_>>()
        ),
    );
    drop(w);
    record(
        into,
        "delete_selected_contact marks the state changed",
        *changed.borrow(),
        format!("changed = {}", *changed.borrow()),
    );
    record(
        into,
        "delete_selected_contact says which contact it deleted",
        status.get_label() == "Deleted the contact: Grace Hopper",
        format!("line of text after delete: {:?}", status.get_label()),
    );
}

/// `delete_selected_contact` with nothing selected: the same "nothing
/// changes, and the line of text asks for a selection" guarantee as the
/// generic `delete_selected`.
fn check_delete_selected_contact_with_nothing_selected_changes_nothing(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Vec<Outcome>,
) {
    let handles = wx_managers::build_contact_manager_dialog(frame, &[], a11y, None);
    let list = handles.list;
    let search = handles.search;
    let status = handles.status;
    list.insert_item(0, "row 0", None);
    // Deliberately left unselected.

    let working = Rc::new(RefCell::new(vec![a_contact("c1", "Grace Hopper")]));
    let index_map = Rc::new(RefCell::new(vec![0usize]));
    let changed = Rc::new(RefCell::new(false));

    delete_selected_contact(
        &list, &search, &status, a11y, &working, &index_map, &changed,
    );

    record(
        into,
        "delete_selected_contact with nothing selected removes nothing",
        working.borrow().len() == 1,
        format!("working contacts: {}", working.borrow().len()),
    );
    record(
        into,
        "delete_selected_contact with nothing selected leaves changed false",
        !*changed.borrow(),
        format!("changed = {}", *changed.borrow()),
    );
    record(
        into,
        "delete_selected_contact with nothing selected asks for a selection",
        status.get_label() == "Select a contact to delete",
        format!("line of text: {:?}", status.get_label()),
    );
}

#[test]
fn test_delete_removes_the_right_row_against_real_widgets() {
    let results: Arc<Mutex<Vec<Outcome>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let results = results.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            let mut outcomes = Vec::new();
            check_delete_selected_removes_the_selected_row(&frame, &a11y, &mut outcomes);
            check_delete_selected_with_nothing_selected_changes_nothing(
                &frame,
                &a11y,
                &mut outcomes,
            );
            check_delete_selected_contact_removes_the_selected_row(&frame, &a11y, &mut outcomes);
            check_delete_selected_contact_with_nothing_selected_changes_nothing(
                &frame,
                &a11y,
                &mut outcomes,
            );

            *results.lock().unwrap() = outcomes;

            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };

    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
    let outcomes = results.lock().unwrap();
    assert!(
        !outcomes.is_empty(),
        "on_init never ran, so nothing was checked"
    );
    let failed: Vec<String> = outcomes
        .iter()
        .filter(|(_, ok, _)| !ok)
        .map(|(name, _, detail)| format!("{name} -- {detail}"))
        .collect();
    assert!(
        failed.is_empty(),
        "{} check(s) failed:\n{}",
        failed.len(),
        failed.join("\n")
    );
}
