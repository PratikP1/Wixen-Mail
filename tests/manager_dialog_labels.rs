//! What the manager dialogs name, and what they offer.
//!
//! Named for what it covers rather than for the dialog that prompted it. The
//! filter rule editor is the first one in here and it will not be the last.
//!
//! One `#[test]` function, for the reason `tests/checkbox_labels.rs` and
//! `tests/theme_reach.rs` both give: wxWidgets allows one application per
//! process and `cargo test` runs each file under `tests/` as its own process.
//! That was measured again for this file rather than taken on trust, and the
//! second half of it is sharper than either of them says. A library test can
//! build a real dialog: one added to `src/presentation/wx_managers.rs` built
//! the filter editor and the whole 5,876-test library run stayed green. A
//! second one in the same process does not. wxWidgets prints
//! `assert "!argc && !argv" failed in Initialize(): initializing twice?` and
//! the run hangs until it is killed. So the constraint is not "no live windows
//! in `cargo test`", it is one `wxdragon::main` per process, and every check
//! that needs a window has to share this one.
//!
//! # What this can see, and what only a screen reader can
//!
//! Windows has two accessibility channels. UI Automation is what Narrator
//! reads, and for a native control the system supplies its own provider that
//! takes the name from the window's own text. MSAA is what NVDA reads, and it
//! is the only place `set_accessible_name` writes.
//!
//! A `CheckBox` therefore has to carry its label on the control, which is
//! readable here through `get_label`. A `Choice` has no text of its own in
//! these dialogs, so it needs `set_accessible_name`. `wxdragon`'s `Accessible`
//! has no name getter, so the name is read the way NVDA reads it, over MSAA
//! with `AccessibleObjectFromWindow`, and compared with the words the code
//! sets. Until 13.4-12 this file asked only whether an accessible object was
//! attached, and wxWidgets 3.3.4 creates one of its own for more controls when
//! anything asks, so that question could answer yes with no name set. What a
//! reading cannot say is how the name sounds in a sentence a person hears;
//! that is a real NVDA pass, recorded in `.planning/WINDOWS.md`.
//!
//! Worth stating plainly because the absence of a failure here is easy to
//! misread: a control with a visible label beside it gets that label as its
//! MSAA name from Windows even when nothing set one. A clean run does not mean
//! every name came from this code.

use std::sync::{Arc, Mutex};
use wixen_mail::application::filters::{
    A_FIELD_A_RULE_MAY_NAME, A_WAY_A_RULE_MAY_MATCH, a_way_of_matching_compares_against_nothing,
    the_words_for_a_field, the_words_for_a_way_of_matching,
};
use wixen_mail::application::saved_searches::{
    Question, THE_FIELD_A_SAVED_SEARCH_NEVER_SEES, what_a_saved_search_cannot_find_with,
};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_managers::{
    FilterRule, build_filter_edit_dialog, build_rule_edit_dialog, make_shell, populate_questions,
};
use wxdragon::prelude::*;

/// One check that failed: what it was about, and what was wrong with it.
type Wrong = Vec<(String, String)>;

/// The MSAA name a reader is given for a window, read through
/// `AccessibleObjectFromWindow` the way NVDA asks for a native control's.
/// Read in this process, which is enough for a name written through
/// wxWidgets' own accessible object: the object answers whoever asks.
#[cfg(target_os = "windows")]
mod msaa {
    use std::ffi::c_void;

    const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
    const VT_I4: u16 = 3;
    // IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent
    // (7), get_accChildCount (8), get_accChild (9), get_accName (10).
    const VTBL_RELEASE: usize = 2;
    const VTBL_GET_ACC_NAME: usize = 10;

    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    /// {618736E0-3C3D-11CF-810C-00AA00389B71}
    const IID_IACCESSIBLE: Guid = Guid {
        data1: 0x618736E0,
        data2: 0x3C3D,
        data3: 0x11CF,
        data4: [0x81, 0x0C, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
    };

    /// A VARIANT as the 64-bit ABI lays it out: 24 bytes, the type at offset
    /// 0 and the payload at offset 8.
    #[repr(C)]
    struct Variant {
        vt: u16,
        reserved: [u16; 3],
        val: i64,
        extra: u64,
    }

    type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
    type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> i32;

    #[link(name = "oleacc")]
    unsafe extern "system" {
        fn AccessibleObjectFromWindow(
            hwnd: isize,
            id_object: u32,
            riid: *const Guid,
            out: *mut *mut c_void,
        ) -> i32;
    }

    #[link(name = "oleaut32")]
    unsafe extern "system" {
        fn SysStringLen(s: *mut u16) -> u32;
        fn SysFreeString(s: *mut u16);
    }

    unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
        // SAFETY: a COM object is a pointer to its vtable.
        unsafe { *(*(object as *const *const *const c_void)).add(index) }
    }

    /// The name of `hwnd`'s client object at `CHILDID_SELF`, or why there
    /// is none.
    fn name_of(hwnd: isize) -> Result<String, String> {
        let mut object: *mut c_void = std::ptr::null_mut();
        // SAFETY: the out pointer is a local; on success it holds an
        // IAccessible, released once below.
        let hr = unsafe {
            AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object)
        };
        if hr < 0 || object.is_null() {
            return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
        }
        let child_self = Variant {
            vt: VT_I4,
            reserved: [0; 3],
            val: 0,
            extra: 0,
        };
        // SAFETY: a live IAccessible; the slots are get_accName and Release,
        // and the BSTR it hands out carries its length and is freed once.
        unsafe {
            let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
            let mut s: *mut u16 = std::ptr::null_mut();
            let hr = get_name(object, child_self, &mut s);
            let name = (!s.is_null()).then(|| {
                let text = String::from_utf16_lossy(std::slice::from_raw_parts(
                    s,
                    SysStringLen(s) as usize,
                ));
                SysFreeString(s);
                text
            });
            let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
            release(object);
            match hr >= 0 {
                true => Ok(name.unwrap_or_default()),
                false => Err(format!("get_accName failed 0x{hr:x}")),
            }
        }
    }

    /// Why `hwnd` is not named `words` on MSAA, or nothing when it is.
    pub fn not_named(hwnd: isize, words: &str) -> Option<String> {
        match name_of(hwnd) {
            Ok(name) if name == words => None,
            Ok(name) => Some(format!(
                "reads {name:?} on MSAA, the channel NVDA reads, where the code sets {words:?}"
            )),
            Err(why) => Some(format!("has no name to read on MSAA: {why}")),
        }
    }
}

/// A stored rule, so a dialog can be opened on one.
fn stored(field: &str, match_type: &str, pattern: &str) -> FilterRule {
    FilterRule {
        id: "stored".to_string(),
        name: "A stored rule".to_string(),
        field: field.to_string(),
        match_type: match_type.to_string(),
        pattern: pattern.to_string(),
        case_sensitive: false,
        action_type: "mark_as_read".to_string(),
        action_value: String::new(),
        enabled: true,
        plays_a_sound: false,
    }
}

/// One condition of a saved search.
fn asking(field: &str, match_type: &str, pattern: &str) -> Question {
    Question {
        field: field.to_string(),
        match_type: match_type.to_string(),
        pattern: pattern.to_string(),
        case_sensitive: false,
    }
}

/// Every string a `Choice` is offering, in order.
fn offered(choice: &Choice) -> Vec<String> {
    (0..choice.get_count())
        .filter_map(|i| choice.get_string(i))
        .collect()
}

/// Whether a list of offered strings is exactly the words for a list of stored
/// names, in both directions and by count.
///
/// Both directions, because either one alone passes over half the fault. A
/// dialog offering nine of eleven and a dialog offering the eleven plus two of
/// its own invention are different bugs and neither is visible to a check that
/// only asks whether every offered string is known.
fn what_is_wrong_with(
    what: &str,
    offering: &[String],
    names: &[&str],
    words: impl Fn(&str) -> Option<&'static str>,
) -> Wrong {
    let mut wrong = Vec::new();
    for name in names {
        let Some(said) = words(name) else {
            wrong.push((
                what.to_string(),
                format!("{name} has no words at all, so nothing could be offered for it"),
            ));
            continue;
        };
        if !offering.iter().any(|o| o == said) {
            wrong.push((
                what.to_string(),
                format!(
                    "the engine answers rules about {name} and the list does not offer \
                     {said:?}, so nobody can write one"
                ),
            ));
        }
    }
    for said in offering {
        let known = names.iter().any(|name| words(name) == Some(said.as_str()));
        if !known {
            wrong.push((
                what.to_string(),
                format!(
                    "{said:?} is offered and is not the words for anything the engine \
                     answers, so a rule written with it matches nothing and says nothing"
                ),
            ));
        }
    }
    if offering.len() != names.len() {
        wrong.push((
            what.to_string(),
            format!(
                "{} offered against {} the engine answers",
                offering.len(),
                names.len()
            ),
        ));
    }
    wrong
}

#[test]
fn test_the_manager_dialogs_name_their_controls_and_offer_what_the_engine_answers() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |_| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let parent = Dialog::builder(&frame, "Filter Manager").build();

            // ── What the two lists offer ────────────────────────────────────
            let fresh = build_filter_edit_dialog(&parent, None, None);

            wrong.extend(what_is_wrong_with(
                "the Match Field list",
                &offered(&fresh.field_choice),
                &A_FIELD_A_RULE_MAY_NAME,
                the_words_for_a_field,
            ));
            wrong.extend(what_is_wrong_with(
                "the Match Type list",
                &offered(&fresh.match_choice),
                &A_WAY_A_RULE_MAY_MATCH,
                the_words_for_a_way_of_matching,
            ));

            // ── Every control this dialog builds carries a name ─────────────
            //
            // The three Choice controls have no visible text of their own, so
            // each is named by the code, and the name is read back over MSAA
            // and compared with the words it sets. The two check boxes have
            // to carry their label on the control instead, because that is
            // the only thing UI Automation reads for one.
            let mut choices_seen = 0;
            for (what, choice, words) in [
                ("the Match Field list", &fresh.field_choice, "Match field"),
                ("the Match Type list", &fresh.match_choice, "Match type"),
                ("the Action list", &fresh.action_choice, "Action"),
            ] {
                choices_seen += 1;
                // The names are read on Windows only, where wxWidgets has an
                // accessible object to carry them, and so is every reading
                // below.
                #[cfg(target_os = "windows")]
                if let Some(why) = msaa::not_named(choice.get_handle() as isize, words) {
                    wrong.push((what.to_string(), why));
                }
                #[cfg(not(target_os = "windows"))]
                let _ = (what, choice, words);
            }
            if choices_seen == 0 {
                wrong.push((
                    "the lists themselves".to_string(),
                    "the dialog built none, so this measured nothing".to_string(),
                ));
            }

            let mut ticks_seen = 0;
            for (what, tick) in [
                ("the Case Sensitive tick", &fresh.cs_check),
                ("the Enabled tick", &fresh.en_check),
            ] {
                ticks_seen += 1;
                let carried = tick.get_label().unwrap_or_default();
                if carried.trim().is_empty() {
                    wrong.push((
                        what.to_string(),
                        "carries no label on the control, so UI Automation has no name for \
                         it and Narrator reads an unnamed check box"
                            .to_string(),
                    ));
                }
            }
            if ticks_seen == 0 {
                wrong.push((
                    "the ticks themselves".to_string(),
                    "the dialog built none, so this measured nothing".to_string(),
                ));
            }

            fresh.dialog.destroy();

            // ── A field the dialog could not previously offer ───────────────
            let formatted = build_filter_edit_dialog(
                &parent,
                Some(&stored("body_html", "contains", "invoice")),
                None,
            );
            let chosen = formatted.field_choice.get_string_selection();
            if chosen.as_deref() != the_words_for_a_field("body_html") {
                wrong.push((
                    "a stored rule about the formatted message text".to_string(),
                    format!(
                        "opens with {chosen:?} selected rather than {:?}, so opening a rule \
                         and pressing OK rewrites it to ask about something else",
                        the_words_for_a_field("body_html")
                    ),
                ));
            }
            formatted.dialog.destroy();

            // ── The Pattern box, when there is nothing to compare against ───
            for way in A_WAY_A_RULE_MAY_MATCH {
                let opened = build_filter_edit_dialog(
                    &parent,
                    Some(&stored("subject", way, "invoice")),
                    None,
                );
                let asking = opened.pattern_f.is_enabled();
                if a_way_of_matching_compares_against_nothing(way) && asking {
                    wrong.push((
                        format!("a rule that matches by {way}"),
                        "opens with a Pattern box asking for something to compare against, \
                         and this way of matching never reads it"
                            .to_string(),
                    ));
                }
                if !a_way_of_matching_compares_against_nothing(way) && !asking {
                    wrong.push((
                        format!("a rule that matches by {way}"),
                        "opens with no Pattern box to fill in, and this way of matching \
                         compares the field against it"
                            .to_string(),
                    ));
                }
                opened.dialog.destroy();
            }

            // ── The condition editor offers what the filter editor does ─────
            //
            // Compared against the other dialog rather than against a written
            // list, because the claim is that there is one vocabulary and not
            // that either dialog matches something typed out here. The filter
            // dialog's own two lists are pinned against the engine's constants
            // above, so this makes the pair transitive.
            //
            // Both non-empty as well as equal: two dialogs offering nothing
            // are equal, and a check that only asked whether they matched
            // would pass over exactly that.
            let a11y = Arc::new(Accessibility::new().expect("an accessibility layer"));
            let fresh_rule = build_rule_edit_dialog(&parent, None, &a11y, None);
            let fresh_filter = build_filter_edit_dialog(&parent, None, None);

            for (what, in_the_condition, in_the_filter) in [
                (
                    "the Match Field list",
                    offered(&fresh_rule.field_choice),
                    offered(&fresh_filter.field_choice),
                ),
                (
                    "the Match Type list",
                    offered(&fresh_rule.match_choice),
                    offered(&fresh_filter.match_choice),
                ),
            ] {
                if in_the_condition.is_empty() {
                    wrong.push((
                        format!("{what} in the condition editor"),
                        "offers nothing at all, so comparing it against the filter editor \
                         would prove nothing"
                            .to_string(),
                    ));
                }
                if in_the_condition != in_the_filter {
                    wrong.push((
                        what.to_string(),
                        format!(
                            "the condition editor offers {in_the_condition:?} and the filter \
                             editor offers {in_the_filter:?}, so one of them holds a list of \
                             its own"
                        ),
                    ));
                }
            }
            fresh_filter.dialog.destroy();

            // ── A new condition opens on something, not on nothing ──────────
            //
            // A `Choice` with no selection reads out as an unfilled combo box,
            // and OK on one stores the empty string as the field.
            for (what, chosen) in [
                (
                    "the Match Field list",
                    fresh_rule.field_choice.get_string_selection(),
                ),
                (
                    "the Match Type list",
                    fresh_rule.match_choice.get_string_selection(),
                ),
            ] {
                if chosen.as_deref().unwrap_or_default().is_empty() {
                    wrong.push((
                        format!("{what} in a new condition"),
                        "opens with nothing selected, so it reads out as an unfilled combo \
                         box and OK would store the empty string"
                            .to_string(),
                    ));
                }
            }
            if !fresh_rule.pattern_f.get_value().is_empty() {
                wrong.push((
                    "the Pattern box in a new condition".to_string(),
                    format!(
                        "opens holding {:?} rather than empty",
                        fresh_rule.pattern_f.get_value()
                    ),
                ));
            }

            // ── Every control the condition editor builds carries a name ────
            for (what, choice, words) in [
                (
                    "the Match Field list",
                    &fresh_rule.field_choice,
                    "Match field",
                ),
                (
                    "the Match Type list",
                    &fresh_rule.match_choice,
                    "Match type",
                ),
            ] {
                #[cfg(target_os = "windows")]
                if let Some(why) = msaa::not_named(choice.get_handle() as isize, words) {
                    wrong.push((format!("{what} in the condition editor"), why));
                }
                #[cfg(not(target_os = "windows"))]
                let _ = (what, choice, words);
            }
            if fresh_rule
                .cs_check
                .get_label()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                wrong.push((
                    "the Case Sensitive tick in the condition editor".to_string(),
                    "carries no label on the control, so UI Automation has no name for it \
                     and Narrator reads an unnamed check box"
                        .to_string(),
                ));
            }
            fresh_rule.dialog.destroy();

            // ── Opening on a stored condition selects all four of it ────────
            let stored_question = Question {
                field: "body_html".to_string(),
                match_type: "ends_with".to_string(),
                pattern: "unsubscribe".to_string(),
                case_sensitive: true,
            };
            let opened = build_rule_edit_dialog(&parent, Some(&stored_question), &a11y, None);
            for (what, chosen, wanted) in [
                (
                    "the Match Field list",
                    opened.field_choice.get_string_selection(),
                    the_words_for_a_field(&stored_question.field),
                ),
                (
                    "the Match Type list",
                    opened.match_choice.get_string_selection(),
                    the_words_for_a_way_of_matching(&stored_question.match_type),
                ),
            ] {
                if chosen.as_deref() != wanted {
                    wrong.push((
                        format!("{what} opened on a stored condition"),
                        format!(
                            "selected {chosen:?} rather than {wanted:?}, so pressing OK \
                             rewrites the condition to ask something else"
                        ),
                    ));
                }
            }
            if opened.pattern_f.get_value() != stored_question.pattern {
                wrong.push((
                    "the Pattern box opened on a stored condition".to_string(),
                    format!(
                        "holds {:?} rather than {:?}",
                        opened.pattern_f.get_value(),
                        stored_question.pattern
                    ),
                ));
            }
            if !opened.cs_check.is_checked() {
                wrong.push((
                    "the Case Sensitive tick opened on a stored condition".to_string(),
                    "is clear, so a condition that matched case would stop doing so the \
                     first time somebody opened it"
                        .to_string(),
                ));
            }
            opened.dialog.destroy();

            // ── What a saved search cannot find, said where it is chosen ────
            //
            // Three fields, because there are two disclosures and they are not
            // the same one (D-2-13), and because a check that only looked for
            // a sentence on one field could be answered by a dialog that says
            // the same thing about all eleven.
            for (field, expected) in [
                (THE_FIELD_A_SAVED_SEARCH_NEVER_SEES, true),
                ("body_plain", true),
                ("subject", false),
            ] {
                let on = build_rule_edit_dialog(
                    &parent,
                    Some(&Question {
                        field: field.to_string(),
                        match_type: "contains".to_string(),
                        pattern: "invoice".to_string(),
                        case_sensitive: false,
                    }),
                    &a11y,
                    None,
                );
                let said = on.what_it_can_find.get_label();
                match (expected, said.trim().is_empty()) {
                    (true, true) => wrong.push((
                        format!("a condition about {field}"),
                        "opens saying nothing, so a condition that can only ever find \
                         nothing is offered in silence"
                            .to_string(),
                    )),
                    (false, false) => wrong.push((
                        format!("a condition about {field}"),
                        format!(
                            "opens saying {said:?}, and a saved search searches this field \
                             in full"
                        ),
                    )),
                    _ => {}
                }
                if expected && Some(said.as_str()) != what_a_saved_search_cannot_find_with(field) {
                    wrong.push((
                        format!("a condition about {field}"),
                        format!(
                            "shows {said:?}, which is not the sentence the application layer \
                             gives for it, so the window has a second wording of its own"
                        ),
                    ));
                }
                on.dialog.destroy();
            }

            // ── The manager shell's own list ────────────────────────────────
            let (shell, _sizer, list, _status) =
                make_shell(&frame, "Filter Manager", "Filters", 450, 400, None);
            #[cfg(target_os = "windows")]
            if let Some(why) = msaa::not_named(list.get_handle() as isize, "Filters") {
                wrong.push(("the manager window's list of rules".to_string(), why));
            }
            #[cfg(not(target_os = "windows"))]
            let _ = &list;
            shell.destroy();

            // ── The condition manager's own list ────────────────────────────
            //
            // A saved search's conditions, in the window plan 02-07 opens from
            // the folder tree. The shell is built here rather than through
            // `show_rule_manager_dialog`, which runs a modal loop and cannot
            // return until somebody closes it; what this can still settle is
            // that the list is named for what it holds, that it carries an
            // accessible object, and that one condition makes one row with all
            // three of its parts in words.
            let (conditions_shell, _sizer, conditions, _status) = make_shell(
                &frame,
                "Conditions for Invoices",
                "Conditions",
                620,
                420,
                None,
            );
            conditions.insert_column(0, "Looks at", ListColumnFormat::Left, 170);
            conditions.insert_column(1, "How", ListColumnFormat::Left, 190);
            conditions.insert_column(2, "What", ListColumnFormat::Left, 220);
            populate_questions(
                &conditions,
                &[
                    asking("subject", "contains", "invoice"),
                    asking("body_plain", "contains", "overdue"),
                    asking("read", "is_true", ""),
                ],
            );

            #[cfg(target_os = "windows")]
            if let Some(why) = msaa::not_named(conditions.get_handle() as isize, "Conditions") {
                wrong.push(("the condition manager's list".to_string(), why));
            }
            if conditions.get_item_count() != 3 {
                wrong.push((
                    "the condition manager's list".to_string(),
                    format!(
                        "painted {} rows for three conditions",
                        conditions.get_item_count()
                    ),
                ));
            }
            // The words, not the stored names. A row reading "body_plain
            // contains overdue" is the machine name spoken to somebody
            // choosing between rows, which is the fault plan 02-04 found in
            // the filter editor's own lists.
            // Compared whole: a field with no words at all is `None`, which no
            // cell equals, so an empty row cannot pass for an empty answer.
            let second_row = conditions.get_item_text(1, 0);
            let message_text = the_words_for_a_field("body_plain");
            if message_text != Some(second_row.as_str()) {
                wrong.push((
                    "a condition row about the message text".to_string(),
                    format!(
                        "reads {second_row:?} rather than {message_text:?}, the words the \
                         editor's own list offers"
                    ),
                ));
            }
            // Four of the eleven ways of matching read no pattern, and the
            // third column is then empty rather than carrying two quotation
            // marks with nothing between them.
            let third_row_pattern = conditions.get_item_text(2, 2);
            if !third_row_pattern.is_empty() {
                wrong.push((
                    "a condition that compares against nothing".to_string(),
                    format!("still shows {third_row_pattern:?} in the What column"),
                ));
            }
            conditions_shell.destroy();

            drop(wrong);
            wxdragon::call_after(Box::new(|| {
                if let Some(app) = wxdragon::get_app_instance() {
                    app.exit_main_loop();
                }
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let wrong = wrong.lock().unwrap();
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong in the manager dialogs:\n{}",
        wrong.len(),
        wrong
            .iter()
            .map(|(what, why)| format!("  {what}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
