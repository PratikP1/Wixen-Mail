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
use crate::presentation::theme;
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
    /// Chosen from a list or typed, for a category.
    Category(ComboBox),
    Tick(CheckBox),
}

/// Ask for everything needed to make one of these.
///
/// `None` when the person cancelled. The containers are the ones this module
/// has; an empty list is fine and means the choice is left out rather than
/// offered with nothing in it.
///
/// `known_categories` are the ones already in use, offered alongside the
/// built-in ones. A fixed list of categories is one that is wrong for everybody
/// eventually.
pub fn ask_for(
    parent: &Frame,
    kind: ItemKind,
    containers: &[Container],
    known_categories: &[String],
) -> Option<(Filled, Option<String>)> {
    let widgets = build_item_form_dialog(
        parent,
        kind,
        containers,
        known_categories,
        theme::current_from_stored_config(),
    )?;

    let answer = widgets.dialog.show_modal();
    let filled = if answer == ID_OK {
        Some(read_back(&widgets.built, containers))
    } else {
        None
    };
    widgets.dialog.destroy();
    filled
}

/// The item form dialog's widgets, returned so a test can build it without a
/// human closing a live modal and so `ask_for` can read every field back
/// after a real `.show_modal()`.
pub struct ItemFormWidgets {
    pub dialog: Dialog,
    /// Every `TextCtrl` this form built for an `Entry::Line` or
    /// `Entry::Paragraph` field, paired with the field it belongs to. A test
    /// can find the one it wants by name without reaching into `Control`,
    /// which stays private to this module.
    pub text_fields: Vec<(&'static Field, TextCtrl)>,
    built: Vec<(&'static Field, Control)>,
}

/// Build the item form dialog without showing it.
///
/// Everything `ask_for` used to do up to its own `.show_modal()` call, split
/// out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// `None` when `kind` has no fields to ask for, mirroring what `ask_for`
/// itself used to do on the same case.
pub fn build_item_form_dialog(
    parent: &Frame,
    kind: ItemKind,
    containers: &[Container],
    known_categories: &[String],
    palette: Option<theme::Palette>,
) -> Option<ItemFormWidgets> {
    let fields = fields_for(kind);
    if fields.is_empty() {
        return None;
    }

    let heading = format!("New {}", kind.label());
    // No height here. It is worked out from what goes in the window, by
    // `set_sizer_and_fit` at the bottom of this function.
    //
    // It used to be a guess: forty pixels plus fifty-two per field. The guess
    // was too small, and what fell off the bottom of every one of these windows
    // was the Save button. There was no way to keep an event, a reminder, a
    // task or a note, from the keyboard or with a mouse, and no error either:
    // the window opened, took what was typed, and had nothing that could
    // agree to it. Pressing Enter did nothing, because the button Enter would
    // have pressed was not there to be the default.
    let dialog = Dialog::builder(parent, &heading)
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
        let control = build_control(&dialog, field, containers, known_categories);
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

    // Save is what Enter presses, so the form can be finished without going to
    // find a button. It is safe as the default here because the other answer
    // is Escape, which every dialog already understands.
    save.set_default();

    // Sized by what is in it. Anything else is a guess that is wrong on a
    // display it was not guessed on, and the part that falls off the bottom is
    // the part added last.
    dialog.set_sizer_and_fit(sizer, true);

    // Focus starts on the first field rather than on Save, so the first thing
    // heard is what to fill in.
    if let Some((_, first)) = built.first() {
        focus(first);
    }

    let text_fields: Vec<(&'static Field, TextCtrl)> = built
        .iter()
        .filter_map(|(field, control)| match control {
            Control::Line(c) | Control::Paragraph(c) => Some((*field, *c)),
            _ => None,
        })
        .collect();

    // Painted last. `None` means high contrast is on, or the system is set
    // up in a way this application should not paint over, so nothing is set
    // here and Windows decides. Every other kind of field this form can
    // build (`DatePickerCtrl`, `TimePickerCtrl`, `Choice`, `ComboBox`,
    // `SpinCtrl`, `CheckBox`) is left to Windows, matching every one of
    // those elsewhere in this round; only a real `TextCtrl` gets its own
    // call.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
        for (_, field) in &text_fields {
            theme::paint(field, palette.main_surface());
        }
    }

    Some(ItemFormWidgets {
        dialog,
        text_fields,
        built,
    })
}

/// Build the control one field asks for.
fn build_control(
    dialog: &Dialog,
    field: &Field,
    containers: &[Container],
    known_categories: &[String],
) -> Control {
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
        // Chosen or typed. A ComboBox rather than a Choice, because a fixed
        // list of categories is one that is wrong for everybody eventually and
        // the ones somebody adds are offered back to them afterwards.
        Entry::PickCategory => {
            let box_ = ComboBox::builder(dialog).build();
            for category in crate::application::categories::offered(known_categories) {
                box_.append(&category);
            }
            Control::Category(box_)
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
        Control::Category(c) => c,
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
        Control::Category(c) => sizer.add(c, 0, flags, 8),
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
/// A date from a picker, in the form everything downstream stores.
///
/// The month is taken as it comes. It used to have one added to it, on the
/// belief that the binding counts months from nought the way wxWidgets does,
/// and it does not: `DateTime::month` adds the one itself and is documented as
/// 1 to 12. So every date anybody typed into an event, a reminder or a task was
/// filed a month later than they set it, silently, and a reminder set for this
/// afternoon never went off because it was not due until next month.
fn as_stored_date(date: &wxdragon::DateTime) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

/// A time from a picker, in the form everything downstream stores.
fn as_stored_time(time: &wxdragon::DateTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

fn read_back(
    built: &[(&'static Field, Control)],
    containers: &[Container],
) -> (Filled, Option<String>) {
    let mut filled = Filled::default();
    let mut chosen_container = None;

    for (field, control) in built {
        match control {
            Control::Line(c) | Control::Paragraph(c) => filled.put(field.name, c.get_value()),
            Control::Date(c) => filled.put(field.name, as_stored_date(&c.get_value())),
            Control::Time(c) => filled.put(field.name, as_stored_time(&c.get_value())),
            // Whatever is in the box, chosen or typed. Tidied where it is
            // read rather than here, because the same tidying has to apply to
            // one that came out of the database.
            Control::Category(c) => filled.put(field.name, c.get_value()),
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
    fn test_a_date_is_stored_as_the_day_that_was_picked() {
        // The month used to have one added to it, because wxWidgets counts
        // months from nought. The binding does not: it adds the one itself.
        // So a reminder set for the thirty-first of July was filed for the
        // thirty-first of August, and it never went off, and nothing said so.
        let midsummer = wxdragon::DateTime::new(2026, 7, 31, 14, 36, 0);

        assert_eq!(as_stored_date(&midsummer), "2026-07-31");
        assert_eq!(as_stored_time(&midsummer), "14:36");
    }

    #[test]
    fn test_the_binding_counts_months_from_one() {
        // The belief this rests on, written down so it fails here rather than
        // in somebody's calendar if the binding ever changes.
        assert_eq!(wxdragon::DateTime::new(2026, 1, 1, 0, 0, 0).month(), 1);
        assert_eq!(wxdragon::DateTime::new(2026, 12, 1, 0, 0, 0).month(), 12);
    }

    #[test]
    fn test_january_and_december_are_padded_and_not_wrapped() {
        // The two the off-by-one hurt worst: December wrapped into a month
        // thirteen, which is not a date at all.
        assert_eq!(
            as_stored_date(&wxdragon::DateTime::new(2026, 1, 5, 0, 0, 0)),
            "2026-01-05"
        );
        assert_eq!(
            as_stored_date(&wxdragon::DateTime::new(2026, 12, 25, 0, 0, 0)),
            "2026-12-25"
        );
    }

    #[test]
    fn test_midnight_and_noon_are_written_the_same_way() {
        assert_eq!(
            as_stored_time(&wxdragon::DateTime::new(2026, 7, 1, 0, 0, 0)),
            "00:00"
        );
        assert_eq!(
            as_stored_time(&wxdragon::DateTime::new(2026, 7, 1, 12, 0, 0)),
            "12:00"
        );
    }

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
