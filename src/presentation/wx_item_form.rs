//! One dialog that can ask for any of the things a person makes.
//!
//! It reads [`crate::application::item_fields`] and builds the controls that
//! description asks for. Every form laid out by the same code means every form
//! has the same tab order, the same shape of label, and the same treatment of
//! what is required, which for somebody moving through it by ear is the
//! difference between four forms and one form with different fields in it.
//!
//! What it replaces is a single text box. An event was a title and an hour
//! from now; a task had no due date and no priority; a note went into no
//! folder. Every column existed in the database and nothing filled them.
//!
//! # Naming and describing
//!
//! wxWidgets associates a static label with the control after it for sighted
//! use only. The accessible name has to be set on the control itself, so each
//! one is named from its own label with the mnemonic taken out, and the help
//! sentence becomes its description, which a screen reader reads on focus.

use crate::application::item_fields::{Entry, Field, Filled, fields_for};
use crate::application::new_item::ItemKind;
use crate::presentation::accessibility::names::{
    name_from_label, set_accessible_name, set_accessible_name_and_description,
};
use wxdragon::prelude::*;

/// A container the new thing could go in: a calendar, a task list, a folder.
#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
}

/// The controls built for one field, so the value can be read back.
enum Control {
    Line(TextCtrl),
    Paragraph(TextCtrl),
    Date(DatePickerCtrl),
    Time(TimePickerCtrl),
    Pick(Choice),
    Containers(Choice),
    Whole(SpinCtrl),
    Tick(CheckBox),
}

/// Ask for everything needed to make one of these.
///
/// `None` when the person cancelled. The containers are the ones this module
/// has; an empty list is fine and means the choice is left out rather than
/// offered with nothing in it.
pub fn ask_for(
    parent: &Frame,
    kind: ItemKind,
    containers: &[Container],
) -> Option<(Filled, Option<String>)> {
    let fields = fields_for(kind);
    if fields.is_empty() {
        return None;
    }

    let heading = format!("New {}", kind.label());
    let dialog = Dialog::builder(parent, &heading)
        .with_size(520, 40 + (fields.len() as i32 * 52))
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let mut built: Vec<(&'static Field, Control)> = Vec::new();

    for field in fields {
        // A container choice with nothing to choose from is left out rather
        // than shown empty. An empty combo box is a control somebody lands on,
        // hears nothing from, and cannot leave a value in.
        if field.entry == Entry::PickContainer && containers.is_empty() {
            continue;
        }

        let label = StaticText::builder(&dialog).with_label(field.label).build();
        sizer.add(&label, 0, SizerFlag::Left | SizerFlag::Top, 8);

        let spoken = name_from_label(field.label);
        let control = build_control(&dialog, field, containers);
        name_it(&control, &spoken, field.help);
        add_to_sizer(&sizer, &control);

        built.push((field, control));
    }

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let save = Button::builder(&dialog)
        .with_label("&Save")
        .with_id(ID_OK)
        .build();
    set_accessible_name(&save, "Save");
    let cancel = Button::builder(&dialog)
        .with_label("&Cancel")
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&cancel, "Cancel");
    buttons.add(&save, 0, SizerFlag::All, 6);
    buttons.add(&cancel, 0, SizerFlag::All, 6);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(sizer, true);

    // Focus starts on the first field rather than on Save, so the first thing
    // heard is what to fill in.
    if let Some((_, first)) = built.first() {
        focus(first);
    }

    let answer = dialog.show_modal();
    let filled = if answer == ID_OK {
        Some(read_back(&built, containers))
    } else {
        None
    };
    dialog.destroy();
    filled
}

/// Build the control one field asks for.
fn build_control(dialog: &Dialog, field: &Field, containers: &[Container]) -> Control {
    match &field.entry {
        Entry::Line => Control::Line(TextCtrl::builder(dialog).build()),
        Entry::Paragraph => Control::Paragraph(
            TextCtrl::builder(dialog)
                .with_style(TextCtrlStyle::MultiLine)
                .with_size(Size::new(480, 90))
                .build(),
        ),
        Entry::Date => Control::Date(DatePickerCtrl::builder(dialog).build()),
        Entry::Time => Control::Time(TimePickerCtrl::builder(dialog).build()),
        Entry::Pick(options) => {
            let choice = Choice::builder(dialog).build();
            for option in *options {
                choice.append(option);
            }
            // The first is the default, and a choice control with nothing
            // selected reads as blank.
            choice.set_selection(0);
            Control::Pick(choice)
        }
        Entry::PickContainer => {
            let choice = Choice::builder(dialog).build();
            for container in containers {
                choice.append(&container.name);
            }
            choice.set_selection(0);
            Control::Containers(choice)
        }
        Entry::Whole { least, most } => {
            let spin = SpinCtrl::builder(dialog).build();
            spin.set_range(*least, *most);
            spin.set_value(*least);
            Control::Whole(spin)
        }
        Entry::Tick => Control::Tick(CheckBox::builder(dialog).with_label("").build()),
    }
}

fn name_it(control: &Control, name: &str, help: &str) {
    let widget: &dyn WxWidget = as_widget(control);
    if help.is_empty() {
        set_accessible_name(widget, name);
    } else {
        set_accessible_name_and_description(widget, name, help);
    }
}

fn as_widget(control: &Control) -> &dyn WxWidget {
    match control {
        Control::Line(c) | Control::Paragraph(c) => c,
        Control::Date(c) => c,
        Control::Time(c) => c,
        Control::Pick(c) | Control::Containers(c) => c,
        Control::Whole(c) => c,
        Control::Tick(c) => c,
    }
}

/// Put the control in the sizer.
///
/// Matched rather than passed as `&dyn WxWidget`, because `add` is generic
/// over a sized widget. The naming above can take the trait object; this
/// cannot.
fn add_to_sizer(sizer: &BoxSizer, control: &Control) {
    let flags = SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom;
    match control {
        Control::Line(c) | Control::Paragraph(c) => sizer.add(c, 0, flags, 8),
        Control::Date(c) => sizer.add(c, 0, flags, 8),
        Control::Time(c) => sizer.add(c, 0, flags, 8),
        Control::Pick(c) | Control::Containers(c) => sizer.add(c, 0, flags, 8),
        Control::Whole(c) => sizer.add(c, 0, flags, 8),
        Control::Tick(c) => sizer.add(c, 0, flags, 8),
    };
}

fn focus(control: &Control) {
    as_widget(control).set_focus();
}

/// Read every control back into a filled form.
///
/// Also gives back the id of the chosen container, which is not a field value:
/// the control shows names and the database wants ids, and matching them up
/// by name later is how two calendars called Work end up merged.
fn read_back(
    built: &[(&'static Field, Control)],
    containers: &[Container],
) -> (Filled, Option<String>) {
    let mut filled = Filled::default();
    let mut chosen_container = None;

    for (field, control) in built {
        match control {
            Control::Line(c) | Control::Paragraph(c) => filled.put(field.name, c.get_value()),
            Control::Date(c) => {
                let date = c.get_value();
                filled.put(
                    field.name,
                    format!(
                        "{:04}-{:02}-{:02}",
                        date.year(),
                        date.month() + 1,
                        date.day()
                    ),
                );
            }
            Control::Time(c) => {
                let time = c.get_value();
                filled.put(
                    field.name,
                    format!("{:02}:{:02}", time.hour(), time.minute()),
                );
            }
            Control::Pick(c) => {
                if let Entry::Pick(options) = field.entry {
                    let at = c.get_selection().unwrap_or(0) as usize;
                    filled.put(field.name, options.get(at).copied().unwrap_or_default());
                }
            }
            Control::Containers(c) => {
                let at = c.get_selection().unwrap_or(0) as usize;
                if let Some(container) = containers.get(at) {
                    filled.put(field.name, container.name.clone());
                    chosen_container = Some(container.id.clone());
                }
            }
            Control::Whole(c) => filled.put(field.name, c.value().to_string()),
            Control::Tick(c) => {
                filled.put(field.name, if c.is_checked() { "true" } else { "false" })
            }
        }
    }

    (filled, chosen_container)
}

/// What to say when a required field was left empty.
///
/// Here rather than in the dialog so the wording can be tested. It names the
/// field, because "some required fields are empty" makes somebody go hunting
/// through a form they cannot see.
pub fn complaint_about(missing: &[&Field]) -> String {
    let names: Vec<String> = missing
        .iter()
        .map(|field| name_from_label(field.label))
        .collect();
    match names.len() {
        0 => String::new(),
        1 => format!("{} is needed before this can be saved", names[0]),
        _ => format!(
            "These are needed before this can be saved: {}",
            names.join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::item_fields::Filled;

    #[test]
    fn test_the_complaint_names_the_field() {
        let missing = Filled::default().missing(ItemKind::Task);

        let said = complaint_about(&missing);

        assert!(said.contains("Title"), "{said}");
        assert!(
            !said.contains('&'),
            "the mnemonic should not be read out: {said}"
        );
    }

    #[test]
    fn test_nothing_missing_says_nothing() {
        assert_eq!(complaint_about(&[]), "");
    }

    #[test]
    fn test_two_missing_are_both_named() {
        let missing = Filled::default().missing(ItemKind::Event);

        let said = complaint_about(&missing);

        // An event needs a title and both dates.
        assert!(said.contains("Title"), "{said}");
        assert!(said.contains("Starts"), "{said}");
        assert!(said.contains("Ends"), "{said}");
    }
}
