//! What a person is asked for when they make an event, task, reminder or note.
//!
//! Until now they were asked for a title and nothing else. Everything the
//! database can hold was filled in with a guess: an event an hour from now, a
//! task with no due date, a reminder that never goes off, a note in no folder.
//! The columns were all there. Nothing put anything in them.
//!
//! # Why this is data rather than four dialogs
//!
//! Four hand-written dialogs drift. The tab order in one ends up different
//! from the next, one gets a help line and the others do not, one loses its
//! accessible names in a refactor. Described as data and rendered by one
//! builder, every form is laid out the same way, named the same way, and the
//! only thing that differs is the list of fields, which is the only thing that
//! should differ.
//!
//! It is also the half that can be tested. A dialog cannot be, but "does a
//! task ask which list it goes in" can.
//!
//! # Where the field lists come from
//!
//! Not invented. Each is the set the format and both providers agree on:
//!
//! - Events: RFC 5545 `VEVENT`, and what Google Calendar and Microsoft Graph
//!   put on a create form. Summary, start, end, all day, location,
//!   description, which calendar, busy or free, status, an alert, and a
//!   repeat.
//! - Tasks: RFC 5545 `VTODO`, Google Tasks and Microsoft To Do. Title, list,
//!   due date, notes. Google Tasks has no priority and no repeat, so those two
//!   are asked for but only Microsoft carries them up.
//! - Reminders: To Do's reminder half. Title, when, priority, repeat, notes.
//! - Notes: title, folder, body, pinned. Notes sync nowhere yet, so there is
//!   no third party shaping this one.

use crate::application::new_item::ItemKind;

/// Which field, so a filled-in form can be read back without matching strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldName {
    Title,
    Container,
    AllDay,
    StartDate,
    StartTime,
    EndDate,
    EndTime,
    DueDate,
    DueTime,
    Location,
    Notes,
    Priority,
    ShowAs,
    Status,
    AlertMinutes,
    Repeat,
    /// Whether a repeating series ever stops, and how.
    RepeatUntil,
    /// The last day it happens, when it stops on a date.
    RepeatUntilDate,
    /// How many times in all, when it stops after a count.
    RepeatTimes,
    /// What kind of day this is: a birthday, a holiday, a deadline.
    Category,
    Pinned,
}

/// How a field is filled in, which decides the control it gets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// One line of text.
    Line,
    /// Several lines.
    Paragraph,
    /// A date, entered with a date control rather than typed into a box.
    Date,
    /// A time of day.
    Time,
    /// One of a fixed set, in the order given. The first is the default.
    Pick(&'static [&'static str]),
    /// One of the containers this module has: a calendar, a task list, a note
    /// folder. Filled in when the form opens, because they are the person's
    /// own and not known here.
    PickContainer,
    /// What kind of day this is, chosen from a list or typed.
    ///
    /// Typed as well as chosen, because a fixed list of categories is one that
    /// is wrong for everybody eventually, and the ones somebody adds are
    /// offered back to them afterwards.
    PickCategory,
    /// A whole number, with the range it is allowed.
    Whole { least: i32, most: i32 },
    /// On or off.
    Tick,
}

/// One thing to fill in.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: FieldName,
    /// What the control is called. Carries its own mnemonic.
    pub label: &'static str,
    /// The sentence a screen reader reads after the label, on focus.
    ///
    /// Empty where the label says everything. A description that only repeats
    /// the label is noise on every visit to the control.
    pub help: &'static str,
    pub entry: Entry,
    /// Whether the form refuses to save without it.
    pub required: bool,
}

/// Busy or free, as both providers put it.
const SHOW_AS: &[&str] = &["Busy", "Free"];
/// RFC 5545 event status, minus the ones only a server sets.
const STATUS: &[&str] = &["Confirmed", "Tentative", "Cancelled"];
const PRIORITY: &[&str] = &["Normal", "High", "Low"];
/// How often something repeats, taken from the one list that also writes the
/// rule, so the words offered here and the rules stored cannot come apart.
const REPEAT: &[&str] = &[
    "Does not repeat",
    "Every day",
    "Every weekday, Monday to Friday",
    "Every week",
    "Every two weeks",
    "Every month, on this date",
    "Every month, on this weekday",
    "Every three months",
    "Every year",
];

/// When a repeating series stops.
///
/// A series could not be told to stop at all before this, so anything set to
/// repeat repeated for ever and a six week course had to be entered six times.
const REPEAT_UNTIL: &[&str] = &["Never", "On a date", "After a number of times"];

/// What to ask for, to make one of these.
///
/// Order matters: it is the tab order, and it is the order a screen reader
/// reads the form. Title first because it is what the thing is, then where it
/// goes, then when, then the rest.
pub fn fields_for(kind: ItemKind) -> &'static [Field] {
    match kind {
        ItemKind::Event => EVENT,
        ItemKind::Task => TASK,
        ItemKind::Reminder => REMINDER,
        ItemKind::Note => NOTE,
        // Both have their own path: mail opens the composer and a contact
        // opens the contact dialog, which asks for far more than a form like
        // this could.
        ItemKind::Mail | ItemKind::Contact => &[],
    }
}

static EVENT: &[Field] = &[
    Field {
        name: FieldName::Title,
        label: "&Title",
        help: "",
        entry: Entry::Line,
        required: true,
    },
    Field {
        name: FieldName::Container,
        label: "&Calendar",
        help: "Which calendar this goes in",
        entry: Entry::PickContainer,
        required: false,
    },
    Field {
        name: FieldName::AllDay,
        label: "&All day",
        help: "The times are ignored when this is ticked",
        entry: Entry::Tick,
        required: false,
    },
    Field {
        name: FieldName::StartDate,
        label: "&Starts",
        help: "",
        entry: Entry::Date,
        required: true,
    },
    Field {
        name: FieldName::StartTime,
        label: "Start t&ime",
        help: "",
        entry: Entry::Time,
        required: false,
    },
    Field {
        name: FieldName::EndDate,
        label: "&Ends",
        help: "",
        entry: Entry::Date,
        required: true,
    },
    Field {
        name: FieldName::EndTime,
        label: "End ti&me",
        help: "",
        entry: Entry::Time,
        required: false,
    },
    Field {
        name: FieldName::Location,
        label: "&Location",
        help: "A room, an address, or a link to a meeting",
        entry: Entry::Line,
        required: false,
    },
    Field {
        name: FieldName::Repeat,
        label: "&Repeat",
        help: "",
        entry: Entry::Pick(REPEAT),
        required: false,
    },
    Field {
        name: FieldName::RepeatUntil,
        label: "Repeat &for",
        help: "Whether the repeat ever stops",
        entry: Entry::Pick(REPEAT_UNTIL),
        required: false,
    },
    Field {
        name: FieldName::RepeatUntilDate,
        label: "Last da&y",
        help: "The last day it happens, when it stops on a date",
        entry: Entry::Date,
        required: false,
    },
    Field {
        name: FieldName::RepeatTimes,
        label: "Ho&w many times",
        help: "How many times in all, when it stops after a count",
        entry: Entry::Whole {
            least: 1,
            most: 999,
        },
        required: false,
    },
    Field {
        name: FieldName::AlertMinutes,
        label: "Alert minutes &before",
        help: "Nought for no alert",
        entry: Entry::Whole {
            least: 0,
            most: 40_320,
        },
        required: false,
    },
    Field {
        name: FieldName::ShowAs,
        label: "S&how as",
        help: "What other people see when they look at your free time",
        entry: Entry::Pick(SHOW_AS),
        required: false,
    },
    Field {
        name: FieldName::Status,
        label: "Stat&us",
        help: "",
        entry: Entry::Pick(STATUS),
        required: false,
    },
    Field {
        name: FieldName::Category,
        label: "Cate&gory",
        help: "What kind of day this is: a birthday, a holiday, a deadline",
        entry: Entry::PickCategory,
        required: false,
    },
    Field {
        name: FieldName::Notes,
        label: "&Description",
        help: "Markdown headings and lists are read back when this is read aloud",
        entry: Entry::Paragraph,
        required: false,
    },
];

static TASK: &[Field] = &[
    Field {
        name: FieldName::Title,
        label: "&Title",
        help: "",
        entry: Entry::Line,
        required: true,
    },
    Field {
        name: FieldName::Container,
        label: "&List",
        help: "Which task list this goes in",
        entry: Entry::PickContainer,
        required: false,
    },
    Field {
        name: FieldName::DueDate,
        label: "&Due",
        help: "Leave today's date if it has no deadline",
        entry: Entry::Date,
        required: false,
    },
    Field {
        name: FieldName::Priority,
        label: "&Priority",
        help: "Google Tasks does not carry this; Microsoft To Do does",
        entry: Entry::Pick(PRIORITY),
        required: false,
    },
    Field {
        name: FieldName::Notes,
        label: "&Notes",
        help: "Markdown headings and lists are read back when this is read aloud",
        entry: Entry::Paragraph,
        required: false,
    },
];

static REMINDER: &[Field] = &[
    Field {
        name: FieldName::Title,
        label: "&Title",
        help: "",
        entry: Entry::Line,
        required: true,
    },
    Field {
        name: FieldName::DueDate,
        label: "&Date",
        help: "",
        entry: Entry::Date,
        required: false,
    },
    Field {
        name: FieldName::DueTime,
        label: "T&ime",
        help: "When this should go off",
        entry: Entry::Time,
        required: false,
    },
    Field {
        name: FieldName::Priority,
        label: "&Priority",
        help: "",
        entry: Entry::Pick(PRIORITY),
        required: false,
    },
    Field {
        name: FieldName::Repeat,
        label: "&Repeat",
        help: "",
        entry: Entry::Pick(REPEAT),
        required: false,
    },
    Field {
        name: FieldName::RepeatUntil,
        label: "Repeat &for",
        help: "Whether the repeat ever stops",
        entry: Entry::Pick(REPEAT_UNTIL),
        required: false,
    },
    Field {
        name: FieldName::RepeatUntilDate,
        label: "Last da&y",
        help: "The last day it happens, when it stops on a date",
        entry: Entry::Date,
        required: false,
    },
    Field {
        name: FieldName::RepeatTimes,
        label: "Ho&w many times",
        help: "How many times in all, when it stops after a count",
        entry: Entry::Whole {
            least: 1,
            most: 999,
        },
        required: false,
    },
    Field {
        name: FieldName::Notes,
        label: "&Notes",
        help: "Markdown headings and lists are read back when this is read aloud",
        entry: Entry::Paragraph,
        required: false,
    },
];

static NOTE: &[Field] = &[
    Field {
        name: FieldName::Title,
        label: "&Title",
        help: "",
        entry: Entry::Line,
        required: true,
    },
    Field {
        name: FieldName::Container,
        label: "&Folder",
        help: "Which folder this note goes in",
        entry: Entry::PickContainer,
        required: false,
    },
    Field {
        name: FieldName::Pinned,
        label: "&Pin to the top",
        help: "Pinned notes are listed first",
        entry: Entry::Tick,
        required: false,
    },
    Field {
        name: FieldName::Notes,
        label: "&Body",
        help: "Markdown headings and lists are read back when this is read aloud",
        entry: Entry::Paragraph,
        required: false,
    },
];

/// A form somebody has filled in.
///
/// Values are kept as text because that is what every control gives back and
/// what the database columns hold. The typed readers below are where a wrong
/// assumption about the shape of one shows up, rather than three layers later
/// as a row nobody can explain.
#[derive(Debug, Default, Clone)]
pub struct Filled {
    values: Vec<(FieldName, String)>,
}

impl Filled {
    /// Record what was typed into one field.
    pub fn put(&mut self, name: FieldName, value: impl Into<String>) {
        let value = value.into();
        match self.values.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = value,
            None => self.values.push((name, value)),
        }
    }

    /// What was typed, trimmed, or empty if the field was not on this form.
    pub fn text(&self, name: FieldName) -> &str {
        self.values
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v.trim())
            .unwrap_or("")
    }

    /// The same, but nothing at all rather than an empty string.
    ///
    /// The database columns that take an `Option` mean "not set" by `None` and
    /// "set to nothing" by `Some("")`, and those are different. A description
    /// somebody left blank is not a description that is blank on purpose.
    pub fn filled_in(&self, name: FieldName) -> Option<String> {
        Some(self.text(name))
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    }

    /// A ticked box.
    pub fn ticked(&self, name: FieldName) -> bool {
        matches!(self.text(name), "true" | "1" | "yes")
    }

    /// A number, or the fallback when the field was not on the form.
    pub fn whole(&self, name: FieldName, if_missing: i32) -> i32 {
        self.text(name).parse().unwrap_or(if_missing)
    }

    /// Which of a field's offered choices was made, lowercased.
    ///
    /// A form that never asked a question must not be read as somebody having
    /// answered it with nothing. Nothing is a value both calendar providers and
    /// the task provider refuse, and a refused change is sent again on every
    /// sync for as long as the row exists, so the empty answer used to be a
    /// task or an event that could never leave this computer and never said so.
    /// Copying a message into a task or an event is exactly that case: there is
    /// no form, so only a title and a body are filled in.
    ///
    /// Falls back to the first choice, which is the one the form shows before
    /// anybody touches it, so a value stored without asking is the value
    /// somebody would have got by not answering.
    pub fn chosen(&self, kind: ItemKind, name: FieldName) -> String {
        let offered = fields_for(kind)
            .iter()
            .find(|field| field.name == name)
            .and_then(|field| match field.entry {
                Entry::Pick(options) => Some(options),
                _ => None,
            })
            .unwrap_or(&[]);
        let answered = self.text(name).to_lowercase();
        if offered.iter().any(|o| o.to_lowercase() == answered) {
            return answered;
        }
        offered.first().unwrap_or(&"").to_lowercase()
    }

    /// What is missing, of the fields that were required.
    ///
    /// Named rather than counted, so the message can say which one and focus
    /// can be put there. "Some required fields are empty" is a message that
    /// makes somebody hunt.
    pub fn missing(&self, kind: ItemKind) -> Vec<&'static Field> {
        fields_for(kind)
            .iter()
            .filter(|field| field.required && self.text(field.name).is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_long_field_says_that_markdown_is_understood() {
        // A feature nobody is told about is one nobody uses. The description
        // is what a screen reader reads after the field's name, so this is
        // where somebody finds out without having to be told twice.
        for kind in [
            ItemKind::Event,
            ItemKind::Task,
            ItemKind::Reminder,
            ItemKind::Note,
        ] {
            for field in fields_for(kind)
                .iter()
                .filter(|f| matches!(f.entry, Entry::Paragraph))
            {
                assert!(
                    field.help.to_lowercase().contains("markdown"),
                    "{:?} {} does not say markdown is understood",
                    kind,
                    field.label
                );
            }
        }
    }

    #[test]
    fn test_the_four_that_were_a_title_in_a_box_now_ask_for_more() {
        // The whole point. Each of these used to be one text prompt, and
        // everything else was a guess written straight into the database.
        for kind in [
            ItemKind::Event,
            ItemKind::Task,
            ItemKind::Reminder,
            ItemKind::Note,
        ] {
            let fields = fields_for(kind);
            assert!(
                fields.len() >= 4,
                "{kind:?} asks for only {} things",
                fields.len()
            );
        }
    }

    #[test]
    fn test_mail_and_contacts_are_left_to_their_own_windows() {
        assert!(fields_for(ItemKind::Mail).is_empty());
        assert!(fields_for(ItemKind::Contact).is_empty());
    }

    #[test]
    fn test_everything_asks_for_a_title_first() {
        // First because it is what the thing is, and because a list read
        // aloud has nothing to say about a row with no title.
        for kind in [
            ItemKind::Event,
            ItemKind::Task,
            ItemKind::Reminder,
            ItemKind::Note,
        ] {
            let first = &fields_for(kind)[0];
            assert_eq!(first.name, FieldName::Title, "{kind:?}");
            assert!(first.required, "{kind:?} would allow an untitled row");
        }
    }

    #[test]
    fn test_everything_that_lives_in_a_container_asks_which_one() {
        // The bug this prevents has already happened twice: a task with no
        // list can never leave the machine, and a note with no folder is
        // filed nowhere. Both were fixed by picking a default silently, which
        // is better than losing it and worse than asking.
        for kind in [ItemKind::Event, ItemKind::Task, ItemKind::Note] {
            assert!(
                fields_for(kind)
                    .iter()
                    .any(|f| f.name == FieldName::Container),
                "{kind:?} does not ask where it goes"
            );
        }
    }

    #[test]
    fn test_an_event_asks_when_it_starts_and_when_it_ends() {
        let names: Vec<FieldName> = fields_for(ItemKind::Event).iter().map(|f| f.name).collect();

        for wanted in [
            FieldName::StartDate,
            FieldName::StartTime,
            FieldName::EndDate,
            FieldName::EndTime,
            FieldName::AllDay,
        ] {
            assert!(
                names.contains(&wanted),
                "an event does not ask for {wanted:?}"
            );
        }
    }

    #[test]
    fn test_no_field_is_asked_for_twice() {
        for kind in ItemKind::ALL {
            let mut names: Vec<FieldName> = fields_for(kind).iter().map(|f| f.name).collect();
            let count = names.len();
            names.sort_by_key(|n| format!("{n:?}"));
            names.dedup();
            assert_eq!(names.len(), count, "{kind:?} asks for something twice");
        }
    }

    #[test]
    fn test_every_label_has_a_mnemonic_and_they_do_not_clash() {
        // A form somebody moves through by keyboard needs Alt plus a letter
        // per field, and two fields sharing a letter means one of them cannot
        // be reached that way.
        for kind in ItemKind::ALL {
            let mut letters: Vec<char> = Vec::new();
            for field in fields_for(kind) {
                let mnemonic = field
                    .label
                    .split('&')
                    .nth(1)
                    .and_then(|rest| rest.chars().next())
                    .unwrap_or_else(|| panic!("{kind:?} {:?} has no mnemonic", field.name));
                let lowered = mnemonic.to_ascii_lowercase();
                assert!(
                    !letters.contains(&lowered),
                    "{kind:?}: two fields both use Alt+{mnemonic}"
                );
                letters.push(lowered);
            }
        }
    }

    #[test]
    fn test_a_pick_offers_something_to_pick() {
        for kind in ItemKind::ALL {
            for field in fields_for(kind) {
                if let Entry::Pick(options) = field.entry {
                    assert!(
                        options.len() >= 2,
                        "{kind:?} {:?} is a choice of {}",
                        field.name,
                        options.len()
                    );
                }
            }
        }
    }

    #[test]
    fn test_every_priority_the_form_offers_is_one_the_task_service_knows() {
        // Green the day it was written, and that is what it is for. The words
        // this form offers and the words a task can be sent to a provider with
        // are two lists of the same three, and nothing else joins them. Adding
        // a fourth word here would otherwise compile, pass, store fine, look
        // right on screen, and be refused by the provider on every sync of that
        // task for as long as it existed.
        use crate::service::tasks_api::TaskPriority;
        for word in PRIORITY {
            assert!(
                TaskPriority::stored(&word.to_lowercase()).is_some(),
                "the form offers {word:?} and a task carrying it cannot be sent anywhere"
            );
        }
    }

    #[test]
    fn test_a_choice_the_form_never_asked_for_falls_back_to_the_one_it_would_have_offered() {
        // The empty answer is what a form that was never shown gives back for
        // every choice on it, and it is the one answer no provider takes.
        let never_asked = Filled::default();
        assert_eq!(
            never_asked.chosen(ItemKind::Task, FieldName::Priority),
            "normal"
        );
        assert_eq!(
            never_asked.chosen(ItemKind::Event, FieldName::Status),
            "confirmed"
        );
        assert_eq!(
            never_asked.chosen(ItemKind::Event, FieldName::ShowAs),
            "busy"
        );
        assert_eq!(
            never_asked.chosen(ItemKind::Reminder, FieldName::Priority),
            "normal"
        );

        // What somebody did choose comes back, and comes back lowercased,
        // because the words are offered with a capital and stored without one.
        let mut answered = Filled::default();
        answered.put(FieldName::Priority, "High");
        answered.put(FieldName::Status, "Tentative");
        answered.put(FieldName::ShowAs, "Free");
        assert_eq!(answered.chosen(ItemKind::Task, FieldName::Priority), "high");
        assert_eq!(
            answered.chosen(ItemKind::Event, FieldName::Status),
            "tentative"
        );
        assert_eq!(answered.chosen(ItemKind::Event, FieldName::ShowAs), "free");

        // Something the form never offered is not an answer either. It can only
        // arrive from a caller building the form's values by hand, and letting
        // it through would put the same unsendable value in the database by a
        // longer road.
        let mut invented = Filled::default();
        invented.put(FieldName::Priority, "Urgent");
        assert_eq!(
            invented.chosen(ItemKind::Task, FieldName::Priority),
            "normal"
        );
    }

    #[test]
    fn test_a_form_with_no_title_says_which_field_is_missing() {
        // Not "some fields are empty", which makes somebody hunt through a
        // form they cannot see.
        let mut filled = Filled::default();
        filled.put(FieldName::Notes, "buy milk");

        let missing = filled.missing(ItemKind::Task);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].name, FieldName::Title);
    }

    #[test]
    fn test_a_field_left_blank_is_not_set_rather_than_set_to_nothing() {
        // The database columns tell those apart, and so should this. A
        // description nobody typed is absent; one typed as spaces is absent
        // too, which is why it trims first.
        let mut filled = Filled::default();
        filled.put(FieldName::Notes, "   ");
        filled.put(FieldName::Location, "Room 2");

        assert_eq!(filled.filled_in(FieldName::Notes), None);
        assert_eq!(
            filled.filled_in(FieldName::Location),
            Some("Room 2".to_string())
        );
        assert_eq!(filled.filled_in(FieldName::Title), None);
    }

    #[test]
    fn test_writing_a_field_twice_keeps_the_second() {
        let mut filled = Filled::default();
        filled.put(FieldName::Title, "first");
        filled.put(FieldName::Title, "second");

        assert_eq!(filled.text(FieldName::Title), "second");
    }

    #[test]
    fn test_a_number_that_was_never_asked_for_falls_back() {
        // A task form has no alert field, so reading one has to give the
        // caller its default rather than nought, which would mean "no alert"
        // when nobody was asked.
        let filled = Filled::default();

        assert_eq!(filled.whole(FieldName::AlertMinutes, 15), 15);
    }

    #[test]
    fn test_help_says_something_the_label_does_not() {
        // A description that repeats the label is read out on every visit to
        // the control and tells nobody anything.
        for kind in ItemKind::ALL {
            for field in fields_for(kind) {
                if field.help.is_empty() {
                    continue;
                }
                let label = field.label.replace('&', "").to_lowercase();
                assert_ne!(field.help.to_lowercase(), label, "{:?}", field.name);
            }
        }
    }
}
