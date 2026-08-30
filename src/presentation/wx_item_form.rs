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
//!
//! # Date and time are three and two real controls, not one
//!
//! `wxDatePickerCtrl` and `wxTimePickerCtrl` wrap a single native Windows
//! control that edits month, day and year, or hour and minute and second, as
//! one focusable object with its own internal notion of which part the arrow
//! keys are on. A real screen reader session found that internal state is
//! not exposed anywhere this application can reach: moving between the parts
//! with Left and Right, and changing one with Up and Down, said nothing,
//! neither which part was landed on nor what its new value was. That is a
//! known, long-standing limitation of the control itself, not something a
//! style flag fixes.
//!
//! So a date is three real controls, a month choice, a day spinner and a
//! year spinner, and a time is two, an hour spinner and a minute spinner,
//! each with its own accessible name. Landing on one and changing its value
//! are both things a real, separate control already does correctly, the same
//! way every other spinner and choice in this application does. Seconds are
//! left out entirely: nothing downstream has ever stored them, and asking
//! for a precision nobody uses is one more control to tab past.
//!
//! # Recurrence is a second page, not four fields in the way
//!
//! Most events and most reminders never repeat. How often something comes
//! round again, whether that ever stops, the last day it happens, and how
//! many times in all used to sit as four fields between the location and the
//! alert, on the one page everybody had to get through to finish the form,
//! whether or not they were ever going to touch any of them.
//!
//! A kind that has anything to say about recurrence now gets a second page
//! for it, reached by moving off the first rather than by tabbing past four
//! fields most people never set. A kind with nothing to say about
//! recurrence, a task or a note, keeps the single page it always had: an
//! empty second page is not a courtesy, it is a tab stop that goes nowhere.
//!
//! # Save refuses what it should, in the dialog it was refused in
//!
//! Save used to be a bare button carrying `ID_OK`, closing on either a click
//! or Enter through wxWidgets' own default handling for a dialog's
//! affirmative button, with nothing asking whether the answer made sense
//! first. Save now has a real handler: it reads every field back, asks
//! [`Filled::problems`] what is wrong, and only closes the dialog when
//! nothing is.
//!
//! Refusing means consuming the click, not merely declining to close.
//! Wiring a handler is not enough on its own, and the first version of this
//! shipped believing it was. wxdragon sets `Skip(true)` before it calls a
//! bound handler and only treats the event as consumed if the handler clears
//! it (`wxdragon-sys`, `cpp/src/event.cpp`: "Reset skip to true before each
//! handler call", then `if (!event.GetSkipped()) event_consumed = true;`, and
//! a final `else { event.Skip(true); }`). A plain button event cannot be
//! vetoed, so a handler that returns without saying otherwise lets the click
//! carry on to `wxDialogBase::OnButton`, which sees `wxID_OK`, calls
//! `AcceptAndClose`, and ends the dialog regardless of what the handler just
//! decided. `event.skip(false)` is what makes the refusal real. Reading
//! wxWidgets' own source was not enough to see this: what happens in between
//! is wxdragon's dispatch, and it is the opposite of what wx alone implies.
//! `tests/house_style.rs` guards it now, since nothing here can fire a real
//! click to find out.
//!
//! What refusing looks like: the reason is set as this dialog's own visible
//! problem line and spoken through the accessibility announcement queue at
//! once, through [`crate::presentation::status_line::said_and_shown`], the
//! same pairing every other status line in this application already uses.
//! Focus moves to the first field named, so the person who just pressed Save
//! lands where the fix belongs rather than having to find it by ear or by
//! tabbing back through everything already answered correctly. Nothing here
//! closes and reopens the dialog: that shape has already cost this
//! application a real NVDA announcement once, documented at length in
//! `wx_managers.rs`'s own `delete_selected`, because hiding a window and
//! showing it again with nothing pumped in between drops whatever tried to
//! announce itself in the gap. Refusing a Save never hides anything at all.

use crate::application::item_fields::{Entry, Field, FieldName, Filled, Problem, fields_for};
use crate::application::new_item::ItemKind;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::{
    name_from_label, set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::date_display::{self, Clock, DateOrder, DateSettings};
use crate::presentation::status_line::said_and_shown;
use crate::presentation::theme;
use crate::presentation::wx_app::date_settings_from;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

/// A container the new thing could go in: a calendar, a task list, a folder.
#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
}

/// A date, entered as three separate, separately named controls rather than
/// one packed native picker. See the module documentation for why.
///
/// Fields `pub`, the same reason [`ItemFormWidgets`]'s own are: a test reads
/// them back after building a real dialog, which is the only way to prove
/// what is on screen without a human tabbing through it.
#[derive(Clone, Copy)]
pub struct DateFields {
    pub month: Choice,
    pub day: SpinCtrl,
    pub year: SpinCtrl,
    /// Whether the day is asked for before the month.
    ///
    /// Carried on the fields rather than looked up again where they are laid
    /// out, so what was built and what is shown cannot disagree. The setting
    /// was read, threaded through three layers, and thrown away with a
    /// `let _ = order;`, while two doc comments and a commit message all said
    /// the order was honoured.
    pub day_first: bool,
}

/// A time, entered as two separate controls, or three when the clock reads
/// to twelve and a morning-or-afternoon choice is needed. See the module
/// documentation for why, and for why there is no third spinner for seconds.
#[derive(Clone, Copy)]
pub struct TimeFields {
    pub hour: SpinCtrl,
    pub minute: SpinCtrl,
    pub am_pm: Option<Choice>,
}

/// The controls built for one field, so the value can be read back.
///
/// `Clone, Copy`, the same as every variant it holds: Save's own handler
/// keeps its own copy of the whole built list to read back from, so the one
/// this function goes on building the rest of the dialog from is never
/// moved out from under it.
#[derive(Clone, Copy)]
enum Control {
    Line(TextCtrl),
    Paragraph(TextCtrl),
    /// A list of people, one to a line. A box like a paragraph's, kept apart
    /// so what is said about it is about writing a person down.
    People(TextCtrl),
    Date(DateFields),
    Time(TimeFields),
    Pick(Choice),
    Containers(Choice),
    Whole(SpinCtrl),
    /// Chosen from a list or typed, for a category.
    Category(ComboBox),
    Tick(CheckBox),
}

/// What to fill the form with, for something already made, bundled into one
/// parameter because a container id only ever means anything alongside the
/// answer it goes with.
pub struct Prefill<'a> {
    pub filled: &'a Filled,
    /// The id of the container `filled` is already in, since a container is
    /// chosen by name in the box and known by id everywhere else. `None`
    /// when this kind does not live in a container, or nothing was chosen.
    pub container: Option<&'a str>,
}

/// How this dialog paints itself, and how it reaches the accessibility
/// announcement queue to say a Save was refused while the dialog stays open.
/// Bundled into one parameter for the same reason `Prefill` is: neither
/// travels anywhere without the other, and `build_item_form_dialog` had
/// already reached clippy's own limit on how many separate parameters one
/// function may carry before this was added.
pub struct Chrome<'a> {
    pub palette: Option<theme::Palette>,
    pub a11y: &'a Arc<Accessibility>,
    /// What to do when somebody asks when the people invited are free.
    ///
    /// `None` for a form that cannot ask: a task, a note, or an event form
    /// opened with no account behind it. The controls are then not built at
    /// all, rather than built and refusing, because a button that never works
    /// is three tab stops in the way of finishing the form.
    pub asking: Option<AskWhenPeopleAreFree>,
}

/// What came back when somebody asked when the people invited are free.
#[derive(Debug, Clone, Default)]
pub struct WhatCameBack {
    /// The whole answer, in sentences somebody can listen to. Said as it
    /// arrives and never shortened: the half that names who could not be
    /// checked is the half a summary would drop.
    pub said: String,
    /// The times offered, the most useful first, each already worded.
    pub times: Vec<OfferedTime>,
}

/// One time the meeting could be, ready to be put into the form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferedTime {
    /// The time, said the way this person reads dates and times.
    pub said: String,
    pub starts: chrono::DateTime<chrono::Local>,
    pub ends: chrono::DateTime<chrono::Local>,
}

/// What the form does when somebody asks when the people invited are free.
///
/// Handed in rather than done here. Working out where to ask needs the account
/// list, its calendars, the machine's credential store and a runtime to wait on
/// without stopping the window, and none of those belong in a dialog builder.
///
/// `Rc` rather than `Arc`: this is pressed by a button, so it is only ever
/// called on the one thread that draws the window. The asking it does happens
/// somewhere else, and what crosses the thread is the answer, not this.
pub type AskWhenPeopleAreFree = Rc<dyn Fn(&Filled) -> WhatCameBack>;

/// The controls that ask when the people invited are free, and show what came
/// back.
///
/// A panel inside the event rather than a window of its own. Somebody is part
/// way through filling an event in, and the answer is about the guest list they
/// have just typed and the times already in the boxes above: a separate window
/// takes them away from all three, and coming back from it means finding their
/// place again. Everything here is an ordinary control in the tab order.
#[derive(Clone, Copy)]
pub struct FreeBusyControls {
    /// The button that asks.
    pub ask: Button,
    /// Where the answer is read. Read-only and several lines, so a screen
    /// reader moves through it a line at a time, which is how somebody checks a
    /// time they half heard.
    pub answer: TextCtrl,
    /// The times that came back, to choose one from. Empty until an answer
    /// arrives.
    pub times: Choice,
    /// The button that puts the chosen time into the event's own boxes.
    pub put_it_in: Button,
}

/// Ask for everything needed to make one of these, or to change one that
/// already exists.
///
/// `None` when the person cancelled. The containers are the ones this module
/// has; an empty list is fine and means the choice is left out rather than
/// offered with nothing in it.
///
/// `known_categories` are the ones already in use, offered alongside the
/// built-in ones. A fixed list of categories is one that is wrong for everybody
/// eventually.
///
/// `prefill` fills the form from something already made rather than leaving
/// it blank, and the heading says Edit rather than New. `None` for something
/// new.
///
/// `parent` is generic rather than fixed to the application's own frame so
/// this can be opened nested under another dialog, the way the Calendar
/// window's own New and Edit Event buttons do, and not only from the main
/// window's New menu.
pub fn ask_for<W: WxWidget>(
    parent: &W,
    kind: ItemKind,
    containers: &[Container],
    known_categories: &[String],
    prefill: Option<Prefill>,
    a11y: &Arc<Accessibility>,
    asking: Option<AskWhenPeopleAreFree>,
) -> Option<(Filled, Option<String>)> {
    let widgets = build_item_form_dialog(
        parent,
        kind,
        containers,
        known_categories,
        Chrome {
            palette: theme::current_from_stored_config(),
            a11y,
            asking,
        },
        current_date_settings(),
        prefill,
    )?;

    // Somebody is filling in an event, a contact, a task or a note. Nothing
    // raised on a timer may open over that (D-27); held across the call, so the
    // answer comes back however the window ends.
    let _typing = crate::presentation::one_question_at_a_time::while_somebody_types();

    let answer = widgets.dialog.show_modal();
    // `ID_OK` only happens at all once Save's own handler has already found
    // nothing wrong with what `read_back` would give back, so this second
    // read is never the one telling somebody their answer was refused; it
    // is the same read done once more because its first result was never
    // kept.
    let filled = if answer == ID_OK {
        Some(read_back(&widgets.built, containers))
    } else {
        None
    };
    widgets.dialog.destroy();
    filled
}

/// The date order and clock this computer, or somebody's own setting, reads.
///
/// Read fresh rather than threaded in from the caller, the same way
/// [`theme::current_from_stored_config`] already is a few lines above this:
/// one place to load it means an event's date fields and the list row it
/// lands in cannot come to disagree about which order the day and month go
/// in.
fn current_date_settings() -> DateSettings {
    crate::data::config::ConfigManager::load_stored()
        .map(|mgr| date_settings_from(mgr.app_config()))
        .unwrap_or_default()
}

/// The notebook a form gets when it has anything to say about how often
/// something comes round again, and the two pages behind it.
///
/// Only built for a kind with a recurrence field at all: a task or a note
/// has nothing to put on a second page, and a page nobody can fill anything
/// in on is not a page, it is a tab stop that goes nowhere.
#[derive(Clone, Copy)]
pub struct RecurrencePages {
    pub notebook: Notebook,
    /// Everything about the thing itself: title, when, where. Selected when
    /// the dialog opens, so recurrence is reached by moving to it rather
    /// than sitting in the way of finishing the form for everybody who was
    /// never going to touch it.
    pub one_time_page: Panel,
    /// How often, and when it stops.
    pub recurrence_page: Panel,
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
    /// Every date this form built, paired with the field it belongs to, for
    /// the same reason `text_fields` is public.
    pub date_fields: Vec<(&'static Field, DateFields)>,
    /// Every time this form built, paired with the field it belongs to, for
    /// the same reason `text_fields` is public.
    pub time_fields: Vec<(&'static Field, TimeFields)>,
    /// Every `Entry::Pick` field this form built, for the same reason
    /// `text_fields` is public. Not the one container choice or the one
    /// category field; those are their own, singular, fields below, because
    /// a form has at most one of each and a test asking for "the" one does
    /// not want to search a list to find it.
    pub pick_fields: Vec<(&'static Field, Choice)>,
    /// Every `Entry::Whole` field this form built, for the same reason
    /// `text_fields` is public.
    pub whole_fields: Vec<(&'static Field, SpinCtrl)>,
    /// Every `Entry::Tick` field this form built, for the same reason
    /// `text_fields` is public.
    pub tick_fields: Vec<(&'static Field, CheckBox)>,
    /// The one container choice this form built, or `None` when this kind
    /// does not live in a container or there was nothing to offer.
    pub container_field: Option<Choice>,
    /// The one category field this form built, or `None` when this kind has
    /// no category.
    pub category_field: Option<ComboBox>,
    /// `Some` for a kind that asks anything about recurrence, `None` for one
    /// that does not. See [`RecurrencePages`].
    pub recurrence: Option<RecurrencePages>,
    /// The controls that ask when the people invited are free, for a form that
    /// was given somewhere to ask. `None` for every other form. See
    /// [`FreeBusyControls`].
    pub free_busy: Option<FreeBusyControls>,
    built: Vec<(&'static Field, Control)>,
}

impl ItemFormWidgets {
    /// Read every field back into a filled form, the same way Save's own
    /// handler does before deciding whether to accept it.
    ///
    /// `pub` so a test can read back what a live dialog really holds, and
    /// ask [`Filled::problems`] the same question Save does, without a
    /// human clicking a real Save button to find out.
    pub fn filled(&self, containers: &[Container]) -> (Filled, Option<String>) {
        read_back(&self.built, containers)
    }
}

/// Everything every field in one form is built from, bundled so the
/// functions that build a field do not each grow a parameter for every one
/// of these; nothing outside this module ever builds one, so none of its
/// fields need to be `pub`.
struct FormContext<'a> {
    containers: &'a [Container],
    known_categories: &'a [String],
    date_settings: DateSettings,
    /// What to fill the form with, for something already made. `None` for
    /// something new, which is every field left at its ordinary default.
    existing: Option<&'a Filled>,
    /// The id of the container `existing` is already in, since a container
    /// is chosen by name in the box and known by id everywhere else.
    existing_container: Option<&'a str>,
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
///
/// `prefill` is `ask_for`'s own prefill parameter, passed straight through;
/// see its doc comment for what it means.
pub fn build_item_form_dialog<W: WxWidget>(
    parent: &W,
    kind: ItemKind,
    containers: &[Container],
    known_categories: &[String],
    chrome: Chrome,
    date_settings: DateSettings,
    prefill: Option<Prefill>,
) -> Option<ItemFormWidgets> {
    let fields = fields_for(kind);
    if fields.is_empty() {
        return None;
    }
    let ctx = FormContext {
        containers,
        known_categories,
        date_settings,
        existing: prefill.as_ref().map(|p| p.filled),
        existing_container: prefill.as_ref().and_then(|p| p.container),
    };

    let heading = match prefill {
        Some(_) => format!("Edit {}", kind.label()),
        None => format!("New {}", kind.label()),
    };
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

    // Recurrence is asked for on its own page, reached by moving off the
    // first one rather than by tabbing past it: see the module doc comment
    // for why. A kind with nothing to say about recurrence keeps the one
    // page it always had, so `main_fields` still gets every field when
    // `recurrence_fields` comes back empty.
    let (main_fields, recurrence_fields): (Vec<&'static Field>, Vec<&'static Field>) = fields
        .iter()
        .partition(|field| !field.name.is_about_recurrence());

    // Built onto whichever page the fields themselves went on, right under
    // them, so it is reached by carrying on through the form rather than by
    // knowing it is there. Filled in after this so it can read every field back.
    let mut free_busy = None;

    let recurrence = if recurrence_fields.is_empty() {
        build_fields_onto(&dialog, &sizer, &main_fields, &ctx, &mut built);
        if chrome.asking.is_some() {
            free_busy = Some(build_asking_when_free_onto(&dialog, &sizer));
        }
        None
    } else {
        let notebook = Notebook::builder(&dialog).build();

        let one_time_page = Panel::builder(&notebook).build();
        let one_time_sizer = BoxSizer::builder(Orientation::Vertical).build();
        build_fields_onto(
            &one_time_page,
            &one_time_sizer,
            &main_fields,
            &ctx,
            &mut built,
        );
        if chrome.asking.is_some() {
            free_busy = Some(build_asking_when_free_onto(&one_time_page, &one_time_sizer));
        }
        one_time_page.set_sizer(one_time_sizer, true);
        notebook.add_page(&one_time_page, "One-Time", true, None);

        let recurrence_page = Panel::builder(&notebook).build();
        let recurrence_sizer = BoxSizer::builder(Orientation::Vertical).build();
        build_fields_onto(
            &recurrence_page,
            &recurrence_sizer,
            &recurrence_fields,
            &ctx,
            &mut built,
        );
        recurrence_page.set_sizer(recurrence_sizer, true);
        notebook.add_page(&recurrence_page, "Recurrence", false, None);

        sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 8);

        Some(RecurrencePages {
            notebook,
            one_time_page,
            recurrence_page,
        })
    };

    // Where Save says why it refused an answer, both on screen and, through
    // `said_and_shown` below, to the accessibility announcement queue.
    // Empty until then: there is nothing wrong with a form nobody has tried
    // to save yet. Built before the buttons so tab order meets it right
    // before the row that can fill it in, the same order a sighted person
    // reading top to bottom would meet it in too.
    let problem_line = StaticText::builder(&dialog).with_label("").build();
    sizer.add(
        &problem_line,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        8,
    );

    // Wired here rather than where the controls were built, because the button
    // that puts a time into the form clears the line above, and that line could
    // not exist yet: a control takes its place in the tab order when it is
    // built, so building it earlier would put it in front of the whole form.
    if let (Some(controls), Some(asking)) = (free_busy.as_ref(), chrome.asking.as_ref()) {
        wire_asking_when_free(
            controls,
            WhatAskingNeeds {
                asking,
                a11y: chrome.a11y,
                built: &built,
                containers,
                settings: date_settings,
            },
            problem_line,
        );
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

    // Takes the click (and Enter, which reaches a default button the same
    // way) away from wxWidgets' own default handling for `ID_OK`. Consuming
    // it is what makes a refusal stick: left unconsumed, the click carries on
    // to `wxDialogBase::OnButton` and the dialog closes whatever this decides.
    // See the module doc comment for the fuller reasoning, and
    // `tests/house_style.rs` for the check that keeps it.
    let built_for_save = built.clone();
    let containers_for_save: Vec<Container> = containers.to_vec();
    let a11y_for_save = Arc::clone(chrome.a11y);
    save.on_click(move |event| {
        // `.event.` because a button event wraps the command event that
        // carries the flag; wxdragon exposes it as a plain field.
        event.event.skip(false);
        let (filled, _) = read_back(&built_for_save, &containers_for_save);
        let problems = filled.problems(kind);
        if problems.is_empty() {
            dialog.end_modal(ID_OK);
            return;
        }
        said_and_shown(
            &problem_line,
            &a11y_for_save,
            &complaint_about(&problems),
            Priority::High,
        );
        if let Some(first) = problems.first()
            && let Some((_, control)) = built_for_save
                .iter()
                .find(|(field, _)| field.name == first.field.name)
        {
            // Bring the page the field is on forward first. Recurrence lives
            // behind a tab, so moving focus without this puts the keyboard on
            // a control nobody can see while the other page is still the one
            // showing: the person is told what to fix and then landed
            // somewhere that looks like nowhere.
            if let Some(pages) = recurrence {
                pages
                    .notebook
                    .set_selection(usize::from(first.field.name.is_about_recurrence()));
            }
            focus(control);
        }
    });

    // Sized by what is in it. Anything else is a guess that is wrong on a
    // display it was not guessed on, and the part that falls off the bottom is
    // the part added last.
    dialog.set_sizer_and_fit(sizer, true);

    // Focus starts on the first field rather than on Save, so the first thing
    // heard is what to fill in. Always a field on the page already showing:
    // `main_fields` is built, and pushed onto `built`, before
    // `recurrence_fields` is, so the first entry is never one of them.
    if let Some((_, first)) = built.first() {
        focus(first);
    }

    let text_fields: Vec<(&'static Field, TextCtrl)> = built
        .iter()
        .filter_map(|(field, control)| match control {
            Control::Line(c) | Control::Paragraph(c) | Control::People(c) => Some((*field, *c)),
            _ => None,
        })
        .collect();
    let date_fields: Vec<(&'static Field, DateFields)> = built
        .iter()
        .filter_map(|(field, control)| match control {
            Control::Date(f) => Some((*field, *f)),
            _ => None,
        })
        .collect();
    let time_fields: Vec<(&'static Field, TimeFields)> = built
        .iter()
        .filter_map(|(field, control)| match control {
            Control::Time(f) => Some((*field, *f)),
            _ => None,
        })
        .collect();
    let pick_fields: Vec<(&'static Field, Choice)> = built
        .iter()
        .filter_map(|(field, control)| match control {
            Control::Pick(c) => Some((*field, *c)),
            _ => None,
        })
        .collect();
    let whole_fields: Vec<(&'static Field, SpinCtrl)> = built
        .iter()
        .filter_map(|(field, control)| match control {
            Control::Whole(c) => Some((*field, *c)),
            _ => None,
        })
        .collect();
    let tick_fields: Vec<(&'static Field, CheckBox)> = built
        .iter()
        .filter_map(|(field, control)| match control {
            Control::Tick(c) => Some((*field, *c)),
            _ => None,
        })
        .collect();
    // At most one of each, so found rather than collected: a form asks for
    // one container and one category, never two.
    let container_field: Option<Choice> = built.iter().find_map(|(_, control)| match control {
        Control::Containers(c) => Some(*c),
        _ => None,
    });
    let category_field: Option<ComboBox> = built.iter().find_map(|(_, control)| match control {
        Control::Category(c) => Some(*c),
        _ => None,
    });

    // Painted last. `None` means high contrast is on, or the system is set
    // up in a way this application should not paint over, so nothing is set
    // here and Windows decides. Every other kind of field this form can
    // build (the date and time controls, `Choice`, `ComboBox`, `SpinCtrl`,
    // `CheckBox`) is left to Windows, matching every one of those elsewhere
    // in this round; only a real `TextCtrl` gets its own call. The notebook
    // and its two pages, when there is one, are painted the same way
    // `wx_settings::build_settings_dialog` paints its own tabs: a `Panel`
    // does not inherit a colour set on its parent, so left alone it would
    // stay the one colour this dialog no longer is.
    if let Some(palette) = chrome.palette {
        theme::paint(&dialog, palette.main_surface());
        if let Some(pages) = recurrence {
            theme::paint(&pages.notebook, palette.main_surface());
            theme::paint(&pages.one_time_page, palette.main_surface());
            theme::paint(&pages.recurrence_page, palette.main_surface());
        }
        for (_, field) in &text_fields {
            theme::paint(field, palette.main_surface());
        }
    }

    Some(ItemFormWidgets {
        dialog,
        text_fields,
        date_fields,
        time_fields,
        pick_fields,
        whole_fields,
        tick_fields,
        container_field,
        category_field,
        recurrence,
        free_busy: free_busy.map(|controls| controls.shown),
        built,
    })
}

/// Build every field in `fields` onto `parent`, in order, adding each one to
/// `sizer` and recording it in `built`.
///
/// The one place a field becomes a label and a control, whichever page it
/// ends up on: a kind with nothing to say about recurrence calls this once,
/// straight onto the dialog; one that does calls it twice, once per page of
/// the notebook that splits recurrence off. Same building, same naming, same
/// tab order either way.
fn build_fields_onto<W: WxWidget>(
    parent: &W,
    sizer: &BoxSizer,
    fields: &[&'static Field],
    ctx: &FormContext,
    built: &mut Vec<(&'static Field, Control)>,
) {
    for &field in fields {
        // A container choice with nothing to choose from is left out rather
        // than shown empty. An empty combo box is a control somebody lands on,
        // hears nothing from, and cannot leave a value in.
        if field.entry == Entry::PickContainer && ctx.containers.is_empty() {
            continue;
        }

        // A check box carries its own label, so the text beside it would be the
        // same words twice: read out once as a label and again as the control's
        // name. An empty one is kept rather than dropped so the two columns
        // still line up down the form.
        let beside = if field.entry == Entry::Tick {
            ""
        } else {
            field.label
        };
        let label = StaticText::builder(parent).with_label(beside).build();
        sizer.add(&label, 0, SizerFlag::Left | SizerFlag::Top, 8);

        let spoken = name_from_label(field.label);
        let control = build_control(parent, field, ctx);
        name_it(&control, &spoken, field.help);
        add_to_sizer(sizer, &control);

        built.push((field, control));
    }
}

/// How many days a month has, February aware of the year it is in.
///
/// Worked out from the calendar itself, first of the next month minus one
/// day, rather than a table of lengths with February special-cased by hand:
/// a table is one more place a leap year rule can be gotten wrong.
fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_of_next = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("the first of a valid month is always a valid date");
    let first_of_this = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .expect("the first of a valid month is always a valid date");
    (first_of_next - first_of_this).num_days() as u32
}

/// Keep the day spinner's range, and its value if need be, true to the month
/// and year currently chosen either side of it.
///
/// Called after either changes. Clamping down rather than refusing: picking
/// the thirty-first, then February, is not a mistake somebody needs stopped,
/// it is a date that no longer exists and has to become the nearest one that
/// does.
///
/// `pub` so a test can call it directly, which is the only way to prove what
/// it does without a human turning the month spinner with a real keyboard.
pub fn clamp_day_to_month(fields: DateFields) {
    let month = fields.month.get_selection().map_or(1, |i| i + 1);
    let max = days_in_month(fields.year.value(), month) as i32;
    fields.day.set_range(1, max);
    if fields.day.value() > max {
        fields.day.set_value(max);
    }
}

/// The earliest year offered. Wide enough for a birthday or an anniversary
/// entered as an event, narrow enough that a stray keystroke cannot produce
/// a year nobody meant.
const EARLIEST_YEAR: i32 = 1900;
/// The latest year offered, for the same reason.
const LATEST_YEAR: i32 = 2100;

/// Build the three controls a date is entered with, laid out in the order
/// this computer, or somebody's own setting, reads a date in.
///
/// `existing` is the stored `YYYY-MM-DD` this date already holds, when it
/// already holds one. Unparseable or absent, the controls open on `now`, the
/// same as before there was anything to prefill from.
fn build_date_fields(
    parent: &dyn WxWidget,
    order: DateOrder,
    now: chrono::DateTime<chrono::Local>,
    existing: Option<&str>,
) -> DateFields {
    use chrono::Datelike;

    let parsed = existing.and_then(|text| chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").ok());
    let anchor_year = parsed.map_or(now.year(), |d| d.year());
    let anchor_month = parsed.map_or(now.month(), |d| d.month());
    let anchor_day = parsed.map_or(now.day(), |d| d.day());

    // Built in the order they are asked for, not only laid out in it.
    // wxWidgets gives a window its place in the tab order when it is created,
    // so building month first and showing day first would read one way and
    // tab the other.
    let day_first = order == DateOrder::DayFirst;
    let a_month = || {
        Choice::builder(parent)
            .with_choices(date_display::MONTHS.iter().map(|m| m.to_string()).collect())
            .with_selection(Some(anchor_month - 1))
            .build()
    };
    let a_day = || {
        let day = SpinCtrl::builder(parent).build();
        day.set_range(1, days_in_month(anchor_year, anchor_month) as i32);
        day.set_value(anchor_day as i32);
        day
    };
    let (month, day) = match day_first {
        true => {
            let day = a_day();
            (a_month(), day)
        }
        false => (a_month(), a_day()),
    };
    let year = SpinCtrl::builder(parent).build();
    year.set_range(EARLIEST_YEAR, LATEST_YEAR);
    year.set_value(anchor_year);

    let fields = DateFields {
        month,
        day,
        year,
        day_first,
    };
    month.on_selection_changed(move |_| clamp_day_to_month(fields));
    year.on_value_changed(move |_| clamp_day_to_month(fields));

    // Year is last either way, the way almost every calendar written out in
    // words already puts it, so there is nothing to decide for it.
    fields
}

/// Build the two or three controls a time is entered with. Two when the
/// clock reads to twenty-four, three when it reads to twelve and a
/// morning-or-afternoon choice is needed alongside the hour.
///
/// `existing` is the stored `HH:MM` this time already holds, when it already
/// holds one. Unparseable or absent, the controls open on `now`, the same as
/// before there was anything to prefill from.
fn build_time_fields(
    parent: &dyn WxWidget,
    clock: Clock,
    now: chrono::DateTime<chrono::Local>,
    existing: Option<&str>,
) -> TimeFields {
    use chrono::Timelike;

    let parsed = existing.and_then(|text| {
        let (h, m) = text.split_once(':')?;
        Some((h.parse::<u32>().ok()?, m.parse::<u32>().ok()?))
    });
    let (anchor_hour, anchor_minute) = parsed.unwrap_or((now.hour(), now.minute()));

    let (displayed_hour, is_pm) = on_the_face_of(anchor_hour, clock);

    let hour = SpinCtrl::builder(parent).build();
    match clock {
        Clock::TwentyFourHour => hour.set_range(0, 23),
        Clock::TwelveHour => hour.set_range(1, 12),
    }
    hour.set_value(displayed_hour as i32);

    let minute = SpinCtrl::builder(parent).build();
    minute.set_range(0, 59);
    minute.set_value(anchor_minute as i32);

    let am_pm = match clock {
        Clock::TwentyFourHour => None,
        Clock::TwelveHour => {
            let choice = Choice::builder(parent)
                .with_choices(vec!["AM".to_string(), "PM".to_string()])
                .with_selection(Some(if is_pm { 1 } else { 0 }))
                .build();
            Some(choice)
        }
    };

    TimeFields {
        hour,
        minute,
        am_pm,
    }
}

/// An hour of the day as the clock in front of somebody shows it, and whether
/// that clock says afternoon.
///
/// One function rather than the conversion written out wherever an hour is put
/// into these controls. [`hour_24`] reads it back, and the two are a pair: a
/// second copy of either drifts from the other, and what that looks like is a
/// meeting agreed for ten in the morning and stored for ten at night.
fn on_the_face_of(hour: u32, clock: Clock) -> (u32, bool) {
    match clock {
        Clock::TwentyFourHour => (hour, false),
        Clock::TwelveHour => (
            match hour % 12 {
                0 => 12,
                other => other,
            },
            hour >= 12,
        ),
    }
}

/// Build the control one field asks for, filled from `ctx.existing` when
/// there is one, otherwise left at the same defaults as before there was
/// anything to prefill from.
fn build_control(parent: &dyn WxWidget, field: &Field, ctx: &FormContext) -> Control {
    let now = chrono::Local::now();
    // `Some("")` for something being edited whose box happens to be blank,
    // `None` for something new. The two have to stay different: a blank box
    // is still a real answer to prefill a `Pick` or a `Tick` from, and only
    // the second one falls back to this field's ordinary default.
    let existing_text = ctx.existing.map(|filled| filled.text(field.name));
    match &field.entry {
        Entry::Line => {
            let c = TextCtrl::builder(parent).build();
            if let Some(text) = existing_text {
                c.set_value(text);
            }
            Control::Line(c)
        }
        Entry::Paragraph => {
            let c = TextCtrl::builder(parent)
                .with_style(TextCtrlStyle::MultiLine)
                .with_size(Size::new(480, 90))
                .build();
            if let Some(text) = existing_text {
                c.set_value(text);
            }
            Control::Paragraph(c)
        }
        // Several lines, like a paragraph, and a control of its own so that
        // what is read out about it is how to write a person down rather than
        // what formatting a description understands.
        Entry::People => {
            let c = TextCtrl::builder(parent)
                .with_style(TextCtrlStyle::MultiLine)
                .with_size(Size::new(480, 90))
                .build();
            if let Some(text) = existing_text {
                c.set_value(text);
            }
            Control::People(c)
        }
        Entry::Date => Control::Date(build_date_fields(
            parent,
            ctx.date_settings.order,
            now,
            existing_text,
        )),
        Entry::Time => Control::Time(build_time_fields(
            parent,
            ctx.date_settings.clock,
            now,
            existing_text,
        )),
        Entry::Pick(options) => {
            let choice = Choice::builder(parent).build();
            for option in *options {
                choice.append(option);
            }
            // The offered word an existing answer matches, case insensitively
            // since a stored answer is lowercase and the box shows a
            // capital. The first is the default either way, new or existing:
            // a choice control with nothing selected reads as blank, and an
            // answer this list does not offer is not a reason to leave the
            // box looking unset.
            let selected = existing_text
                .and_then(|text| options.iter().position(|o| o.eq_ignore_ascii_case(text)))
                .unwrap_or(0);
            choice.set_selection(selected as u32);
            Control::Pick(choice)
        }
        Entry::PickContainer => {
            let choice = Choice::builder(parent).build();
            for container in ctx.containers {
                choice.append(&container.name);
            }
            let selected = ctx
                .existing_container
                .and_then(|id| ctx.containers.iter().position(|c| c.id == id))
                .unwrap_or(0);
            choice.set_selection(selected as u32);
            Control::Containers(choice)
        }
        // Chosen or typed. A ComboBox rather than a Choice, because a fixed
        // list of categories is one that is wrong for everybody eventually and
        // the ones somebody adds are offered back to them afterwards.
        Entry::PickCategory => {
            let box_ = ComboBox::builder(parent).build();
            for category in crate::application::categories::offered(ctx.known_categories) {
                box_.append(&category);
            }
            if let Some(text) = existing_text {
                box_.set_value(text);
            }
            Control::Category(box_)
        }
        Entry::Whole { least, most } => {
            let spin = SpinCtrl::builder(parent).build();
            spin.set_range(*least, *most);
            let value = existing_text
                .and_then(|text| text.parse::<i32>().ok())
                .filter(|answered| (*least..=*most).contains(answered))
                .unwrap_or(*least);
            spin.set_value(value);
            Control::Whole(spin)
        }
        Entry::Tick => {
            // The label goes on the control, not on the static text beside it.
            // Windows takes a native control's UI Automation name from its own
            // window text, and `set_accessible_name` writes MSAA and nothing
            // else, so a check box with an empty label is named for NVDA and
            // unnamed for Narrator. A check box is the one field kind with
            // somewhere of its own to put a label, so it uses it.
            let c = CheckBox::builder(parent).with_label(field.label).build();
            let checked = existing_text.is_some_and(|text| matches!(text, "true" | "1" | "yes"));
            c.set_value(checked);
            Control::Tick(c)
        }
    }
}

/// Give every control a field builds its accessible name, and its help
/// sentence as a description where there is one.
///
/// A date or a time is more than one control, and each of them is named with
/// the field's own name first, "Starts, Month" rather than a bare "Month",
/// because a form can ask for two dates at once, an event's start and its
/// end, and "Month" heard twice with nothing to tell them apart is a name
/// that has stopped naming anything. The help sentence, where the field has
/// one, is attached to the first control only: read once on the way into the
/// group rather than again on every one of two or three controls in a row.
fn name_it(control: &Control, name: &str, help: &str) {
    match control {
        Control::Date(fields) => {
            name_part(fields.month, name, "Month", help);
            name_part(fields.day, name, "Day", "");
            name_part(fields.year, name, "Year", "");
        }
        Control::Time(fields) => {
            name_part(fields.hour, name, "Hour", help);
            name_part(fields.minute, name, "Minute", "");
            if let Some(am_pm) = fields.am_pm {
                name_part(am_pm, name, "AM or PM", "");
            }
        }
        _ => {
            let widget = as_widget(control);
            if help.is_empty() {
                set_accessible_name(widget, name);
            } else {
                set_accessible_name_and_description(widget, name, help);
            }
        }
    }
}

/// Name one control of a date or a time: the field's own name, then which
/// part of it this is.
fn name_part(widget: impl WxWidget, field_name: &str, part: &str, help: &str) {
    let full = format!("{field_name} {part}");
    if help.is_empty() {
        set_accessible_name(&widget, &full);
    } else {
        set_accessible_name_and_description(&widget, &full, help);
    }
}

/// What the button that asks is called, and what it says while it waits.
const FIND_WHEN_EVERYONE_IS_FREE: &str = "Find whe&n everyone is free";
/// What the button that applies a time is called.
const PUT_THIS_TIME_IN: &str = "&Put this time in the event";
/// What the answer box says before anybody has asked.
const NOBODY_HAS_ASKED_YET: &str =
    "Nothing asked yet. Fill in who is coming, then choose Find when everyone is free.";
/// What is said when there is no chosen time to put in.
///
/// Covers both ways there can be none, because from where somebody is sitting
/// they are the same silence: nobody has asked yet, and nothing was offered.
const NO_TIME_TO_PUT_IN: &str =
    "No time is chosen. Choose one from Times offered, or find some first.";

/// Build the controls that ask when the people invited are free.
///
/// Every one of them is an ordinary control with a visible label beside it and
/// a name of its own, so the group is reached by tabbing and read like the rest
/// of the form rather than being a region somebody has to know about.
/// Nothing is wired here. What the buttons do is wired by
/// [`wire_asking_when_free`], once the line that says why a Save was refused
/// exists: a control takes its place in the tab order when it is built, so that
/// line cannot be built before the page these sit on, and the button that puts a
/// time into the form has to be able to clear it.
fn build_asking_when_free_onto<W: WxWidget>(parent: &W, sizer: &BoxSizer) -> TheAskingControls {
    let across = SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top;

    let ask = Button::builder(parent)
        .with_label(FIND_WHEN_EVERYONE_IS_FREE)
        .build();
    set_accessible_name_and_description(
        &ask,
        &name_from_label(FIND_WHEN_EVERYONE_IS_FREE),
        "Asks your calendar server about everybody on the guest list at once. \
         This can take a few seconds.",
    );
    sizer.add(&ask, 0, SizerFlag::Left | SizerFlag::Top, 8);

    let answer_label = StaticText::builder(parent)
        .with_label("What came back")
        .build();
    sizer.add(&answer_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let answer = TextCtrl::builder(parent)
        .with_style(TextCtrlStyle::ReadOnly | TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .with_size(Size::new(480, 110))
        .build();
    answer.set_value(NOBODY_HAS_ASKED_YET);
    set_accessible_name_and_description(
        &answer,
        "What came back",
        "The answer in full, including anybody whose calendar could not be \
         checked. Somebody nobody could check is never counted as free.",
    );
    sizer.add(&answer, 0, across, 8);

    let times_label = StaticText::builder(parent)
        .with_label("Times offered")
        .build();
    sizer.add(&times_label, 0, SizerFlag::Left | SizerFlag::Top, 8);
    let times = Choice::builder(parent).build();
    set_accessible_name_and_description(
        &times,
        "Times offered",
        "The times that work, the most useful first. Nothing is changed until \
         Put this time in the event is chosen.",
    );
    sizer.add(&times, 0, across, 8);

    let put_it_in = Button::builder(parent).with_label(PUT_THIS_TIME_IN).build();
    set_accessible_name_and_description(
        &put_it_in,
        &name_from_label(PUT_THIS_TIME_IN),
        "Writes the chosen time over the start and end already in this form.",
    );
    sizer.add(
        &put_it_in,
        0,
        SizerFlag::Left | SizerFlag::Top | SizerFlag::Bottom,
        8,
    );

    TheAskingControls {
        shown: FreeBusyControls {
            ask,
            answer,
            times,
            put_it_in,
        },
        // What came back, held so the button that applies a time can read it.
        // Not read back out of the choice control: a label is words, and the
        // instant behind it is what the form needs.
        offered: Rc::new(RefCell::new(Vec::new())),
    }
}

/// The free/busy controls and the answer they are showing.
///
/// Kept together between building them and wiring them, because the button that
/// applies a time needs both, and the two happen at different points in
/// [`build_item_form_dialog`].
struct TheAskingControls {
    shown: FreeBusyControls,
    offered: Rc<RefCell<Vec<OfferedTime>>>,
}

/// Everything the free/busy buttons do when they are pressed.
///
/// Split from building them so it can run after the line that says why a Save
/// was refused exists. See [`build_asking_when_free_onto`].
fn wire_asking_when_free(
    controls: &TheAskingControls,
    what_it_needs: WhatAskingNeeds<'_>,
    problem_line: StaticText,
) {
    wire_asking(
        AskingWiring {
            ask: controls.shown.ask,
            answer: controls.shown.answer,
            times: controls.shown.times,
            offered: &controls.offered,
        },
        what_it_needs.asking,
        what_it_needs.a11y,
        what_it_needs.built,
        what_it_needs.containers,
    );
    wire_putting_it_in(
        PuttingItIn {
            put_it_in: controls.shown.put_it_in,
            times: controls.shown.times,
            offered: &controls.offered,
            problem_line,
        },
        what_it_needs.a11y,
        what_it_needs.built,
        what_it_needs.settings,
    );
}

/// What the wiring needs from the form around it.
struct WhatAskingNeeds<'a> {
    asking: &'a AskWhenPeopleAreFree,
    a11y: &'a Arc<Accessibility>,
    built: &'a [(&'static Field, Control)],
    containers: &'a [Container],
    settings: DateSettings,
}

/// What the button that applies a time works on.
struct PuttingItIn<'a> {
    put_it_in: Button,
    times: Choice,
    offered: &'a Rc<RefCell<Vec<OfferedTime>>>,
    /// Where Save says why it refused this form. Cleared when a time is put in,
    /// because whatever it was refused for was about the start and the end.
    problem_line: StaticText,
}

/// The controls one wiring function needs, bundled so it does not carry six
/// parameters of its own.
struct AskingWiring<'a> {
    ask: Button,
    answer: TextCtrl,
    times: Choice,
    offered: &'a Rc<RefCell<Vec<OfferedTime>>>,
}

/// What pressing the button that asks does.
fn wire_asking(
    controls: AskingWiring<'_>,
    asking: &AskWhenPeopleAreFree,
    a11y: &Arc<Accessibility>,
    built: &[(&'static Field, Control)],
    containers: &[Container],
) {
    let AskingWiring {
        ask,
        answer,
        times,
        offered,
    } = controls;
    let asking = Rc::clone(asking);
    let a11y = Arc::clone(a11y);
    let built: Vec<(&'static Field, Control)> = built.to_vec();
    let containers: Vec<Container> = containers.to_vec();
    let offered = Rc::clone(offered);

    ask.on_click(move |event| {
        // This button does not carry `ID_OK`, so nothing here would close the
        // dialog. Consumed anyway, for the reason the module doc gives about
        // Save: a click left unconsumed carries on to the dialog's own default
        // handling, and asking a question is not an answer to the form.
        event.event.skip(false);

        let (filled, _) = read_back(&built, &containers);
        let came_back = asking(&filled);

        answer.set_value(&came_back.said);
        times.clear();
        for time in &came_back.times {
            times.append(&time.said);
        }
        if !came_back.times.is_empty() {
            times.set_selection(0);
        }
        *offered.borrow_mut() = came_back.times;

        // The whole answer, never a summary of it. The sentence that names
        // who could not be checked is the one a summary would drop, and it is
        // the one somebody has to hear before they book anything.
        //
        // High rather than Normal: this is the answer to something the person
        // asked for and waited on, and the queue holds Normal back behind
        // whatever else is going on. It supersedes an earlier answer on the
        // same topic instead of queueing behind it, so asking twice reads out
        // twice rather than four times.
        let _ = a11y.announce_topic(&came_back.said, Priority::High, WHEN_EVERYONE_IS_FREE);
    });
}

/// The one topic every announcement from this part of the form is on, so a
/// second answer replaces the first rather than being read out after it.
const WHEN_EVERYONE_IS_FREE: &str = "when-everyone-is-free";

/// What pressing the button that applies a time does.
///
/// Nothing happens until it is pressed. Writing the chosen time in as soon as
/// the choice changes would overwrite what somebody typed while they were still
/// reading the list, which is the one thing a control like this must not do.
fn wire_putting_it_in(
    controls: PuttingItIn<'_>,
    a11y: &Arc<Accessibility>,
    built: &[(&'static Field, Control)],
    settings: DateSettings,
) {
    let PuttingItIn {
        put_it_in,
        times,
        offered,
        problem_line,
    } = controls;
    let a11y = Arc::clone(a11y);
    let built: Vec<(&'static Field, Control)> = built.to_vec();
    let offered = Rc::clone(offered);

    put_it_in.on_click(move |event| {
        event.event.skip(false);

        let chosen = times
            .get_selection()
            .and_then(|which| offered.borrow().get(which as usize).cloned());
        let Some(chosen) = chosen else {
            let _ = a11y.announce_topic(NO_TIME_TO_PUT_IN, Priority::High, WHEN_EVERYONE_IS_FREE);
            return;
        };

        put_the_time_into_the_form(&built, &chosen, settings);
        // Whatever Save last refused this form for was about the start and the
        // end, and those have just been replaced. Left on screen it is a
        // sentence about values that are no longer there, which is the same
        // stale answer as saying something changed and not re-reading it.
        problem_line.set_label("");

        // Said, because the boxes that changed are elsewhere in the form and
        // somebody working by ear has no other way to learn that four of them
        // just did. The old values are gone, so this says what they are now
        // rather than that something happened.
        let _ = a11y.announce_topic(
            &format!(
                "The event now starts {} and ends {}.",
                date_display::absolute(chosen.starts, settings),
                date_display::absolute(chosen.ends, settings),
            ),
            Priority::High,
            WHEN_EVERYONE_IS_FREE,
        );
    });
}

/// Write a chosen time over the start and end already in the form.
fn put_the_time_into_the_form(
    built: &[(&'static Field, Control)],
    chosen: &OfferedTime,
    settings: DateSettings,
) {
    for (field, control) in built {
        match (field.name, control) {
            (FieldName::StartDate, Control::Date(fields)) => {
                put_the_date_in(*fields, chosen.starts.date_naive());
            }
            (FieldName::EndDate, Control::Date(fields)) => {
                put_the_date_in(*fields, chosen.ends.date_naive());
            }
            (FieldName::StartTime, Control::Time(fields)) => {
                put_the_clock_in(*fields, chosen.starts.time(), settings.clock);
            }
            (FieldName::EndTime, Control::Time(fields)) => {
                put_the_clock_in(*fields, chosen.ends.time(), settings.clock);
            }
            _ => {}
        }
    }
}

/// Put a day into the three controls a date is entered with.
fn put_the_date_in(fields: DateFields, day: chrono::NaiveDate) {
    use chrono::Datelike;

    fields.year.set_value(day.year());
    fields.month.set_selection(day.month() - 1);
    // The day's range follows the month it is in, and setting month and year
    // from code does not fire the handlers that keep it following. Without
    // this, moving to a short month from a long one leaves a range that allows
    // the thirty-first of February.
    clamp_day_to_month(fields);
    fields.day.set_value(day.day() as i32);
}

/// Put a time of day into the controls a time is entered with.
fn put_the_clock_in(fields: TimeFields, at: chrono::NaiveTime, clock: Clock) {
    use chrono::Timelike;

    let (displayed, is_pm) = on_the_face_of(at.hour(), clock);
    if let Some(am_pm) = fields.am_pm {
        am_pm.set_selection(u32::from(is_pm));
    }
    fields.hour.set_value(displayed as i32);
    fields.minute.set_value(at.minute() as i32);
}

/// The one widget that stands for a control, for the cases where there is
/// only one. A date or a time, which are several, are matched before this is
/// reached.
fn as_widget(control: &Control) -> &dyn WxWidget {
    match control {
        Control::Line(c) | Control::Paragraph(c) | Control::People(c) => c,
        Control::Date(_) | Control::Time(_) => {
            unreachable!("date and time fields are named and focused as themselves")
        }
        Control::Pick(c) | Control::Containers(c) => c,
        Control::Whole(c) => c,
        Control::Category(c) => c,
        Control::Tick(c) => c,
    }
}

/// Put the control in the sizer.
///
/// A date or a time is laid out as its own row of controls side by side,
/// month before day or day before month as the date order asks for, always
/// ending in year; hour, minute, and a morning-or-afternoon choice when the
/// clock reads to twelve.
fn add_to_sizer(sizer: &BoxSizer, control: &Control) {
    let flags = SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom;
    match control {
        Control::Line(c) | Control::Paragraph(c) | Control::People(c) => sizer.add(c, 0, flags, 8),
        Control::Date(fields) => {
            let row = BoxSizer::builder(Orientation::Horizontal).build();
            match fields.day_first {
                true => {
                    row.add(&fields.day, 0, SizerFlag::All, 4);
                    row.add(&fields.month, 0, SizerFlag::All, 4);
                }
                false => {
                    row.add(&fields.month, 0, SizerFlag::All, 4);
                    row.add(&fields.day, 0, SizerFlag::All, 4);
                }
            }
            row.add(&fields.year, 0, SizerFlag::All, 4);
            sizer.add_sizer(&row, 0, flags, 8)
        }
        Control::Time(fields) => {
            let row = BoxSizer::builder(Orientation::Horizontal).build();
            row.add(&fields.hour, 0, SizerFlag::All, 4);
            row.add(&fields.minute, 0, SizerFlag::All, 4);
            if let Some(am_pm) = fields.am_pm {
                row.add(&am_pm, 0, SizerFlag::All, 4);
            }
            sizer.add_sizer(&row, 0, flags, 8)
        }
        Control::Pick(c) | Control::Containers(c) => sizer.add(c, 0, flags, 8),
        Control::Whole(c) => sizer.add(c, 0, flags, 8),
        Control::Category(c) => sizer.add(c, 0, flags, 8),
        Control::Tick(c) => sizer.add(c, 0, flags, 8),
    };
}

fn focus(control: &Control) {
    match control {
        Control::Date(fields) => fields.month.set_focus(),
        Control::Time(fields) => fields.hour.set_focus(),
        other => as_widget(other).set_focus(),
    }
}

/// A date read from the month, day and year controls, in the form everything
/// downstream stores.
fn as_stored_date(year: i32, month: u32, day: u32) -> String {
    format!("{year:04}-{month:02}-{day:02}")
}

/// A time read from the hour and minute controls, in the form everything
/// downstream stores.
fn as_stored_time(hour: u32, minute: u32) -> String {
    format!("{hour:02}:{minute:02}")
}

/// The twenty-four hour reading of an hour a twelve-hour clock displayed.
///
/// `displayed` is what the hour spinner holds, one to twelve. Midnight is
/// entered as 12 AM and stored as 0; noon is entered as 12 PM and stored as
/// 12, which `% 12` alone does not give: it takes 12 back to 0 for both, and
/// only one of them should become 0.
fn hour_24(displayed: u32, is_pm: bool) -> u32 {
    match (displayed % 12, is_pm) {
        (0, false) => 0,
        (h, false) => h,
        (0, true) => 12,
        (h, true) => h + 12,
    }
}

/// The hour a time's controls hold, already in twenty-four hour form.
fn hour_from(fields: &TimeFields) -> u32 {
    match fields.am_pm {
        None => fields.hour.value().max(0) as u32,
        Some(am_pm) => hour_24(
            fields.hour.value().max(1) as u32,
            am_pm.get_selection() == Some(1),
        ),
    }
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
            Control::Line(c) | Control::Paragraph(c) | Control::People(c) => {
                filled.put(field.name, c.get_value());
            }
            Control::Date(fields) => {
                let month = fields.month.get_selection().map_or(1, |i| i + 1);
                filled.put(
                    field.name,
                    as_stored_date(fields.year.value(), month, fields.day.value().max(1) as u32),
                );
            }
            Control::Time(fields) => {
                filled.put(
                    field.name,
                    as_stored_time(hour_from(fields), fields.minute.value().max(0) as u32),
                );
            }
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

/// What Save says when it refuses an answer: every problem in one sentence.
///
/// Here rather than in the dialog so the wording can be tested. Each
/// `Problem` already carries its own full reason; this only joins them, one
/// sentence per problem, because somebody who got three things wrong
/// deserves to hear all three at once rather than trying Save three times to
/// find out about each in turn.
pub fn complaint_about(problems: &[Problem]) -> String {
    if problems.is_empty() {
        return String::new();
    }
    let said: Vec<&str> = problems.iter().map(|p| p.said.as_str()).collect();
    format!("{}.", said.join(". "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::item_fields::Filled;

    #[test]
    fn test_a_date_is_stored_from_its_three_controls() {
        assert_eq!(as_stored_date(2026, 7, 31), "2026-07-31");
    }

    #[test]
    fn test_putting_an_hour_into_the_boxes_and_reading_it_back_gives_the_same_hour() {
        // Two functions that have to be exact opposites: one fills the hour
        // box and the morning-or-afternoon choice, the other reads them. Every
        // hour of the day, both clocks, because the ones that go wrong are the
        // two nobody thinks about: midnight, which shows as 12 in the morning,
        // and noon, which shows as 12 in the afternoon.
        //
        // The writing half used to be spelled out inside the builder, so
        // putting a suggested time into the form would have been a third copy.
        // A third copy drifting from these two looks like a meeting agreed for
        // ten in the morning and stored for ten at night.
        for clock in [Clock::TwelveHour, Clock::TwentyFourHour] {
            for hour in 0..24 {
                let (displayed, is_pm) = on_the_face_of(hour, clock);
                // The two branches `hour_from` reads the controls back
                // through. `hour_24` is only ever reached when there is a
                // morning-or-afternoon choice to read: on a twenty-four hour
                // clock the spinner already holds the hour, and putting it
                // through `hour_24` would turn noon into midnight.
                let read_back = match clock {
                    Clock::TwelveHour => hour_24(displayed, is_pm),
                    Clock::TwentyFourHour => displayed,
                };
                assert_eq!(
                    read_back, hour,
                    "{hour} shown as {displayed} came back wrong on {clock:?}"
                );
            }
        }
        // And the two the clock face names oddly really are named that way,
        // so the round trip above is not two matching mistakes.
        assert_eq!(on_the_face_of(0, Clock::TwelveHour), (12, false));
        assert_eq!(on_the_face_of(12, Clock::TwelveHour), (12, true));
        assert_eq!(on_the_face_of(13, Clock::TwelveHour), (1, true));
        assert_eq!(on_the_face_of(13, Clock::TwentyFourHour), (13, false));
    }

    #[test]
    fn test_a_time_is_stored_from_its_two_controls() {
        assert_eq!(as_stored_time(14, 36), "14:36");
    }

    #[test]
    fn test_january_and_december_are_padded_and_not_wrapped() {
        assert_eq!(as_stored_date(2026, 1, 5), "2026-01-05");
        assert_eq!(as_stored_date(2026, 12, 25), "2026-12-25");
    }

    #[test]
    fn test_midnight_and_noon_are_written_the_same_way() {
        assert_eq!(as_stored_time(0, 0), "00:00");
        assert_eq!(as_stored_time(12, 0), "12:00");
    }

    #[test]
    fn test_the_binding_counts_months_from_one() {
        // The belief every date field in this file rests on, written down so
        // it fails here rather than in somebody's calendar if the crate ever
        // changes. A reminder set for the thirty-first of July was once filed
        // for the thirty-first of August, silently, because an earlier
        // version of this file added one to a month that already counted
        // from one.
        assert_eq!(wxdragon::DateTime::new(2026, 1, 1, 0, 0, 0).month(), 1);
        assert_eq!(wxdragon::DateTime::new(2026, 12, 1, 0, 0, 0).month(), 12);
    }

    #[test]
    fn test_days_in_month_knows_february() {
        assert_eq!(days_in_month(2026, 2), 28, "2026 is not a leap year");
        assert_eq!(days_in_month(2024, 2), 29, "2024 is a leap year");
        assert_eq!(days_in_month(2000, 2), 29, "divisible by 400, a leap year");
        assert_eq!(days_in_month(1900, 2), 28, "divisible by 100 and not 400");
    }

    #[test]
    fn test_days_in_month_knows_thirty_and_thirty_one() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 4), 30);
    }

    #[test]
    fn test_days_in_month_carries_december_into_the_next_year() {
        // The off-by-one this would hit if the next month wrapped to 13
        // instead of rolling the year over.
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn test_twelve_hour_midnight_and_noon_convert_correctly() {
        // The one pair `% 12` alone gets wrong: both reduce to zero, and only
        // midnight should stay there.
        assert_eq!(hour_24(12, false), 0, "12 AM is midnight, stored as 0");
        assert_eq!(hour_24(12, true), 12, "12 PM is noon, stored as 12");
    }

    #[test]
    fn test_twelve_hour_ordinary_hours_convert_correctly() {
        assert_eq!(hour_24(9, false), 9, "9 AM");
        assert_eq!(hour_24(9, true), 21, "9 PM");
        assert_eq!(hour_24(1, false), 1, "1 AM");
        assert_eq!(hour_24(11, true), 23, "11 PM");
    }

    #[test]
    fn test_the_complaint_names_the_field() {
        let problems = Filled::default().problems(ItemKind::Task);

        let said = complaint_about(&problems);

        assert!(said.contains("Title"), "{said}");
        assert!(
            !said.contains('&'),
            "the mnemonic should not be read out: {said}"
        );
    }

    #[test]
    fn test_nothing_wrong_says_nothing() {
        assert_eq!(complaint_about(&[]), "");
    }

    #[test]
    fn test_several_problems_are_all_named() {
        let problems = Filled::default().problems(ItemKind::Event);

        let said = complaint_about(&problems);

        // An event needs a title and both dates.
        assert!(said.contains("Title"), "{said}");
        assert!(said.contains("Starts"), "{said}");
        assert!(said.contains("Ends"), "{said}");
    }
}
