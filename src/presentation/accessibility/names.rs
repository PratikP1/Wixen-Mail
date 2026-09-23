//! Accessible names for controls that carry no visible label.
//!
//! A list, tree, or text field with no label next to it reaches the UI
//! Automation tree with a null Name. A screen reader then announces it as
//! "list" or "tree" with nothing to say which one, which is the difference
//! between a usable window and a guessing game.
//!
//! `wxWindow::SetName` does **not** do this. That name is an internal
//! wxWidgets identifier used for resource lookup and never reaches assistive
//! technology. Only a `wxAccessible` attached to the window does, which is
//! what this module provides.
//!
//! **This is Windows only.** `wxAccessible` is implemented against Microsoft
//! Active Accessibility and has no GTK or macOS counterpart in wxWidgets, so on
//! those platforms these calls are accepted and have no effect. A port would
//! need `setAccessibilityLabel:` on the NSView for macOS and an ATK name for
//! GTK, written as separate bridges rather than as a change here.
//!
//! # Which reader hears what, measured
//!
//! What is set here is an MSAA name, and for a control Windows draws itself,
//! that is not what UI Automation reports. Windows supplies its own UIA
//! provider for a native edit, button or combo box, and it answers before the
//! bridge that would otherwise expose this object. Read through the two APIs
//! side by side against the running composer, the same control answers twice:
//!
//! ```text
//! Button   UIA='B'                MSAA='Bold, Ctrl+B'
//! Button   UIA='Attach File...'   MSAA='Attach a file, Alt+A'
//! Edit     UIA='To:'              MSAA='To'
//! ```
//!
//! So these names are not lost, and a reader that uses IAccessible for ordinary
//! Win32 controls, which is what NVDA does in a window like this one, hears
//! exactly what is written here, descriptions included. A reader that uses UI
//! Automation, which is what Narrator does, hears the control's own window
//! text: the visible label for a button, the static label beside a field.
//!
//! Three things follow, and they are the reason this is written down.
//!
//! Set a name here **and** make the visible label say the same thing, wherever
//! the two can agree. Where they cannot, the name here is the fuller one and
//! the label is the shorter one, never a different fact.
//!
//! A description set here reaches MSAA only. Anything somebody must hear to
//! work a control cannot live only in a description; put it in the label, or
//! announce it.
//!
//! And the automated scan reads the UI Automation tree, so it is measuring the
//! labels rather than any of this. A clean scan says the labels are present. It
//! says nothing at all about these.

use wxdragon::accessible::{AccStatus, Accessible, AccessibleImpl};
use wxdragon::ffi;
use wxdragon::prelude::WxWidget;

/// Whether a name set here reaches the accessibility tree on this build.
///
/// **A statement about wxWidgets, not about this code.** The module header
/// above records the measurement: `wxAccessible` is implemented against
/// Microsoft Active Accessibility and has no GTK or macOS counterpart, so
/// `set_accessible` is accepted everywhere and does something in one place.
/// That is why this is two arms here rather than two modules the way
/// [`super::screen_reader`] has them: there is no per-platform code in this file
/// to put in a module, only a fact about what the toolkit underneath does.
///
/// A port is a third arm, written beside the bridge that earns it:
/// `setAccessibilityLabel:` on the NSView for macOS, an ATK name for GTK, as
/// the header says. Read by [`super::platform_bridge`], which turns it into the
/// sentence somebody meets.
#[cfg(target_os = "windows")]
pub const A_NAME_REACHES_THE_ACCESSIBILITY_TREE: bool = true;

/// Whether a name set here reaches the accessibility tree on this build.
///
/// See the Windows arm above for why this is a fact about wxWidgets. Here the
/// calls in this module are accepted and have no effect, so every control with
/// no visible label beside it reaches the accessibility tree with no name.
#[cfg(not(target_os = "windows"))]
pub const A_NAME_REACHES_THE_ACCESSIBILITY_TREE: bool = false;

/// Supplies one fixed name, and optionally a description, for a control,
/// leaving every other accessibility property to the platform's default
/// handling.
struct FixedName {
    name: String,
    description: Option<String>,
}

impl AccessibleImpl for FixedName {
    /// Name the control itself, and nothing inside it.
    ///
    /// Child zero is the control. Anything else is a row, a tree node, or a
    /// notebook tab asking for its own name, and answering those with the
    /// control's name gave every one of them the same label. That is why the
    /// settings tabs stayed silent even after child enumeration was restored:
    /// enumeration worked, and then every tab reported itself as "Settings
    /// categories".
    fn get_name(&self, child_id: i32) -> (AccStatus, Option<String>) {
        if child_id != 0 {
            return (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None);
        }
        (ffi::wxd_AccStatus_WXD_ACC_OK, Some(self.name.clone()))
    }

    /// Defer to the control's own child enumeration.
    ///
    /// This override is not optional. The trait's default answers `OK` with a
    /// count of zero, which is a positive claim that the control has no
    /// children rather than a request to fall back. Attaching one of these to a
    /// notebook silenced its tabs, and to a list or tree it would have hidden
    /// every item, because the accessible object was answering "nothing in
    /// here" on the control's behalf.
    ///
    /// Every other method left at its default returns `NOT_IMPLEMENTED`, which
    /// is what makes wxWidgets use its own implementation. Only the name and
    /// the description are meant to be replaced here.
    fn get_child_count(&self) -> (AccStatus, i32) {
        (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0)
    }

    /// The sentence a screen reader reads after the name, on focus.
    ///
    /// This is the property that carries "what does choosing this cost me".
    /// Putting that in the name instead makes it part of every announcement,
    /// including the one when arrowing past, which is how a four-sentence
    /// radio button label happens. A description is spoken once, when the
    /// control takes focus.
    ///
    /// Same rule as the name about child zero: a row or a tab asking for its
    /// own description must not be handed the control's.
    fn get_description(&self, child_id: i32) -> (AccStatus, Option<String>) {
        match (&self.description, child_id) {
            (Some(description), 0) => (ffi::wxd_AccStatus_WXD_ACC_OK, Some(description.clone())),
            _ => (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None),
        }
    }
}

// Until 2026-09-18 this file also held `CheckedRows`, an object that answered
// for each row of the folder chooser's `CheckListBox` as a check box with a
// checked state, written through wxdragon's `acc_state` constants and held by
// twenty tests to answering those flags. Read over `AccessibleObjectFromWindow`
// by `tests/a_kept_folder_reads_as_a_checked_check_box.rs` (11-03, reading A)
// the rows answered role check button and states 0xc000840 on the kept row and
// 0x40 on the others: READONLY, BUSY and two ALERT bits, never CHECKED, which
// is what the tester heard (#70). wxdragon 0.9.17 numbers those constants as
// MSAA does (CHECKED 0x10, FOCUSABLE 0x100000, SELECTABLE 0x200000), its shim
// hands them to `wxAccessible::GetState` unconverted, and wxWidgets numbers its
// own enumeration differently (BUSY 0x10, PROTECTED 0x100000, READONLY
// 0x200000) before converting that to the platform's. The dialog is a native
// tree with `TVS_CHECKBOXES` now, whose state NVDA reads from the control
// itself, and the object, its tests and its four guard records went with it.
// Nothing else in this tree writes a state through those constants; whoever
// next does should read this paragraph first.

/// Give `window` an accessible name that screen readers will announce.
///
/// Use the same wording as the control's visible heading or tree root, so what
/// is spoken matches what is on screen.
pub fn set_accessible_name(window: &dyn WxWidget, name: &str) {
    window.set_accessible(Accessible::new(
        window,
        FixedName {
            name: name.to_string(),
            description: None,
        },
    ));
}

/// Give `window` a name and the sentence that explains it.
///
/// For a control whose label is not enough on its own: a choice with a
/// consequence, a field with a rule about what it accepts. Screen readers read
/// the description after the name when the control takes focus, so what is on
/// screen next to the control is heard by somebody who cannot see it.
///
/// Without this, an explanation sitting beside a control is a label floating in
/// the window that a screen reader user reaches only by leaving the control and
/// reading around, if they think to. The first-run screen shipped that way:
/// three radio buttons that read correctly and three explanations of what each
/// one costs that were never spoken.
///
/// One call, not two. Attaching an accessible object replaces the last one, so
/// setting a name and then a description would leave only the description.
pub fn set_accessible_name_and_description(window: &dyn WxWidget, name: &str, description: &str) {
    window.set_accessible(Accessible::new(
        window,
        FixedName {
            name: name.to_string(),
            description: Some(description.to_string()),
        },
    ));
}

/// Turn a visible label into the name a screen reader should announce.
///
/// Drops the mnemonic ampersand: some screen readers read it aloud, and none
/// of them should.
///
/// A trailing colon, the visual convention marking a field's label, becomes a
/// trailing comma rather than being dropped outright. Dropping it left
/// nothing between the name and whatever a screen reader reads next, the
/// control's role and then its value, and a real NVDA run found exactly
/// that: no pause, consistently, across most of the fields in the
/// application. A comma is never read aloud as a word the way "colon"
/// sometimes is, and it is still a pause to the speech synthesiser
/// underneath.
///
/// An ampersand means two different things in a wxWidgets label and they have
/// to be told apart. A lone one marks the letter after it as the mnemonic and
/// is not part of what the label says. Two in a row are how a label writes one
/// real ampersand, as in the composer's "&Go Back && Edit". Deleting every one
/// of them takes the word out of the label and leaves a double space where a
/// listener hears an odd pause. The real ampersand is kept as a character
/// rather than turned into the word "and", because a screen reader already
/// reads it and rewriting it would put a word in the name that is not on the
/// control.
///
/// One trailing colon becomes one trailing comma, not a run of them. A single
/// colon after a field label is the visual convention; a second is something
/// the label says, and is left as a colon rather than turned into something
/// the label never had.
pub fn name_from_label(label: &str) -> String {
    let mut spoken = String::with_capacity(label.len());
    let mut rest = label.chars().peekable();
    while let Some(character) = rest.next() {
        if character != '&' {
            spoken.push(character);
            continue;
        }
        if rest.peek() == Some(&'&') {
            rest.next();
            spoken.push('&');
        }
    }
    let trimmed = spoken.trim();
    match trimmed.strip_suffix(':') {
        Some(before) if !before.trim().is_empty() => format!("{},", before.trim()),
        Some(_) => String::new(),
        None => trimmed.to_string(),
    }
}

/// Which property of a spin control's typing field some words are written to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldProperty {
    Name,
    Description,
}

/// What the field a person types in carries, property by property: the
/// arrows' name, and their description where they have one.
///
/// `pub` because nothing inside the crate calls it until the helper that
/// writes these arrives, and a narrower visibility does not compile under the
/// lint then.
pub fn what_the_typing_field_carries<'a>(
    name: &'a str,
    _description: Option<&'a str>,
) -> Vec<(FieldProperty, &'a str)> {
    vec![(FieldProperty::Name, name)]
}

/// Hold one cell of a two-column grid open with nothing in it.
///
/// A checkbox carries its own label, so in a grid of label and field pairs
/// the label column beside it is empty. That cell used to be filled with a
/// `StaticText` built on `""`, which is a real window: it reaches both
/// accessibility channels as a nameless control, sitting straight before the
/// checkbox in the order Tab moves in, and it is the window Windows picks
/// when it names a control that set no name from the nearest static text.
/// Two testers met an unnamed checkbox built that way on the first day of
/// testing (#42, #40). A sizer spacer takes up the cell and is not a window
/// at all, so nothing nameless is in the tree.
///
/// One pixel rather than none, because `wxdragon` adds nothing at all for a
/// spacer of size zero and the cells after it would shift left a column. In
/// a grid with a growable field column a pixel is not visible.
pub fn leave_the_cell_empty(grid: &wxdragon::sizers::FlexGridSizer) {
    grid.add_spacer(1);
}

#[cfg(test)]
mod tests {
    use super::{
        AccessibleImpl, FieldProperty, FixedName, name_from_label, what_the_typing_field_carries,
    };
    use wxdragon::ffi;

    #[test]
    fn test_a_spin_controls_description_travels_to_the_field_a_person_types_in() {
        // Tab lands in the field, never on the arrows, so a sentence the
        // arrows alone carry is one nobody tabbing through a form hears. The
        // event form's Alert minutes before has one, "Nought for no alert".
        assert_eq!(
            what_the_typing_field_carries("Alert minutes before", Some("Nought for no alert")),
            vec![
                (FieldProperty::Name, "Alert minutes before"),
                (FieldProperty::Description, "Nought for no alert"),
            ]
        );
    }

    #[test]
    fn test_the_name_is_the_only_thing_replaced() {
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };
        let (status, name) = named.get_name(0);
        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(name.as_deref(), Some("Messages"));
    }

    #[test]
    fn test_children_name_themselves() {
        // A list row, a tree node and a notebook tab each ask for their own
        // name through the parent. Answering with the control's name labels
        // every one of them identically.
        let named = FixedName {
            name: "Settings categories".to_string(),
            description: None,
        };
        let (status, name) = named.get_name(1);
        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(name.is_none());
    }

    #[test]
    fn test_a_description_is_offered_for_the_control_itself() {
        // The bug this was written for: the first-run screen's three choices
        // each had a sentence beside them saying what it costs, and a screen
        // reader never read any of them. The label was named, the explanation
        // was a separate piece of text nobody was pointed at, and somebody
        // choosing what Wixen Mail may change heard only "read my mail,
        // change nothing" with no idea what the other two did.
        let described = FixedName {
            name: "Read my mail, change nothing".to_string(),
            description: Some("Nothing you do here reaches your provider.".to_string()),
        };

        let (status, description) = described.get_description(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(
            description.as_deref(),
            Some("Nothing you do here reaches your provider.")
        );
    }

    #[test]
    fn test_a_control_with_no_description_leaves_the_question_to_the_platform() {
        // NOT_IMPLEMENTED rather than an empty string. Answering with nothing
        // is a claim that there is no description, which stops wxWidgets
        // supplying whatever the control would have said for itself.
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };

        let (status, description) = named.get_description(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(description.is_none());
    }

    #[test]
    fn test_children_describe_themselves() {
        // Same trap as the name. Handing every row of a list the control's
        // description would have each one read it out.
        let described = FixedName {
            name: "Messages".to_string(),
            description: Some("Your inbox".to_string()),
        };

        let (status, description) = described.get_description(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(description.is_none());
    }

    #[test]
    fn test_child_enumeration_is_left_to_the_control() {
        // The trait default answers OK with zero children, which is a claim
        // that the control is empty rather than a request to fall back. With
        // that default in place a named notebook lost its tabs and a named
        // list would have lost every row.
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };
        let (status, count) = named.get_child_count();
        assert_eq!(
            status,
            ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED,
            "naming a control must not claim it has no children"
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_strips_the_mnemonic_and_turns_the_colon_into_a_pause() {
        // Not dropped outright: a real NVDA run found nothing left between
        // the name and whatever is read next, the field's role and then its
        // value, ran together with no pause. A comma is never read aloud as
        // a word, and is still a pause to the synthesiser underneath it.
        assert_eq!(name_from_label("&Subject:"), "Subject,");
        assert_eq!(
            name_from_label("Start &Date (YYYY-MM-DD):"),
            "Start Date (YYYY-MM-DD),"
        );
    }

    #[test]
    fn test_leaves_a_plain_label_alone() {
        // No colon on the label, so nothing here to turn into a pause.
        assert_eq!(name_from_label("Accounts"), "Accounts");
    }

    #[test]
    fn test_handles_a_label_that_is_only_decoration() {
        assert_eq!(name_from_label(":"), "");
        assert_eq!(name_from_label("   "), "");
    }

    #[test]
    fn test_a_doubled_ampersand_is_the_one_a_label_really_means() {
        // wxWidgets writes a literal ampersand as two of them, so deleting
        // every ampersand takes the word out of the label and leaves a double
        // space behind it. This is the composer's own send confirmation, the
        // button that goes back to a message nobody has sent yet.
        assert_eq!(name_from_label("&Go Back && Edit"), "Go Back & Edit");
        assert!(
            !name_from_label("&Go Back && Edit").contains("  "),
            "a double space is an odd pause in the middle of the button"
        );
        // The same shape on the notebook pages of the contact and settings
        // windows.
        assert_eq!(name_from_label("Email && Phone"), "Email & Phone");
    }

    #[test]
    fn test_only_one_trailing_colon_is_a_visual_convention() {
        // One colon after a field label is how a form is written, and becomes
        // a comma. A second one is something the label says, and is left as
        // a colon rather than turned into something the label never had.
        assert_eq!(name_from_label("Ratio::"), "Ratio:,");
    }
}
