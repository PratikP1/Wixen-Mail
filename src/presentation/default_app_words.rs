//! What the settings screen says about which program Windows opens things with.
//!
//! The facts come from [`crate::service::default_apps`]. The words are here,
//! because they are read out to somebody and the sentence has to make sense on
//! its own: a screen reader announces one row at a time and nothing around it.
//!
//! # The sentence has to be honest about what this program can do
//!
//! Since Windows 8 an application cannot make itself the default. It can be
//! listed as a candidate and it can send somebody to the Windows screen where
//! they choose, and that is all. So nothing here says "set as default", because
//! a button saying that would be a button that cannot do what it says. It says
//! where it takes you.
//!
//! # Three of the six have no answer to give
//!
//! Windows keeps a default for email, for calendar files and for contact cards.
//! It keeps none for tasks, reminders or notes: there is no protocol and no
//! file type that maps to a default program for those. Saying so is better than
//! leaving three rows out, because somebody looking for tasks in this list
//! would otherwise be left wondering whether they had missed it.

use crate::service::default_apps::{DefaultKind, Status};

/// The name of a kind, as somebody would say it.
pub fn kind_name(kind: DefaultKind) -> &'static str {
    match kind {
        DefaultKind::Mail => "Email",
        DefaultKind::Calendar => "Calendar",
        DefaultKind::Contacts => "Contacts",
        DefaultKind::Tasks => "Tasks",
        DefaultKind::Reminders => "Reminders",
        DefaultKind::Notes => "Notes",
    }
}

/// One row of the list, as a whole sentence.
///
/// Whole, rather than a name in one column and a state in another, because a
/// screen reader reads a row and the two halves have to arrive together for it
/// to mean anything.
pub fn row(kind: DefaultKind, status: &Status) -> String {
    let name = kind_name(kind);
    match status {
        Status::Ours => format!("{name}: Wixen Mail"),
        // The identifier is deliberately not read out when there is a real
        // name for it. `AppXbx2ce4vcxjdhff3d1ms66qqzk12zn827` is not something
        // to say to anybody.
        Status::AnotherProgram {
            program_name: Some(program),
            ..
        } => format!("{name}: {program}"),
        Status::AnotherProgram {
            program_name: None, ..
        } => format!("{name}: another program"),
        Status::Partial { held, not_held } => format!(
            "{name}: Wixen Mail for {}, another program for {}",
            a_list_of(held),
            a_list_of(not_held)
        ),
        Status::NoWindowsSlot => {
            format!("{name}: Windows has no default program for this")
        }
        // The reason is carried rather than swallowed. "Could not be
        // determined" on its own is the kind of sentence somebody reads twice
        // and learns nothing from.
        Status::Undetermined { reason } => format!("{name}: could not be checked, {reason}"),
    }
}

/// What the button says, and what it does.
///
/// Not "Set as default". Windows does not let a program do that, so a button
/// promising it would be a promise this cannot keep. This one says where it
/// goes, which is the only thing on offer.
pub fn button_label() -> &'static str {
    "&Choose default programs in Windows..."
}

/// The sentence above the list, explaining why there is no button that just
/// does it.
pub fn why_windows_asks() -> &'static str {
    "Windows decides which program opens email, calendar files and contact \
     cards, and only you can change it. Wixen Mail can show you what is set \
     now and take you to the Windows screen where you choose."
}

/// What to announce when the startup check finds this program is not the
/// default for something it could be.
///
/// Empty when there is nothing to say, which is the ordinary case. Somebody who
/// has chosen another mail program on purpose should not be told about it every
/// time they start this one, which is why this is off unless it is asked for.
pub fn what_the_startup_check_found(found: &[(DefaultKind, Status)]) -> String {
    let missing: Vec<&'static str> = found
        .iter()
        .filter(|(_, status)| {
            matches!(
                status,
                Status::AnotherProgram { .. } | Status::Partial { .. }
            )
        })
        .map(|(kind, _)| kind_name(*kind))
        .collect();
    if missing.is_empty() {
        return String::new();
    }
    format!(
        "Wixen Mail is not the default for {}. Settings, General has the list \
         and the way to change it.",
        a_list_of_static(&missing)
    )
}

/// "a, b and c", so a reading sounds like a sentence rather than a table.
fn a_list_of(items: &[String]) -> String {
    let borrowed: Vec<&str> = items.iter().map(String::as_str).collect();
    a_list_of_static(&borrowed)
}

fn a_list_of_static(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [only] => (*only).to_string(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_kind_has_a_name_somebody_would_say() {
        // Walking the list rather than naming six, so a kind added later
        // cannot arrive without words.
        for kind in DefaultKind::ALL {
            let said = kind_name(kind);
            assert!(!said.is_empty(), "{kind:?} has no name");
            assert!(
                said.chars().next().is_some_and(char::is_uppercase),
                "{kind:?} reads as {said:?} in the middle of a sentence"
            );
        }
    }

    #[test]
    fn test_a_row_names_the_kind_and_says_what_holds_it() {
        // One sentence, because a screen reader reads a row and the two halves
        // have to arrive together.
        let said = row(DefaultKind::Mail, &Status::Ours);

        assert!(said.contains("Email"), "{said}");
        assert!(said.contains("Wixen Mail"), "{said}");
    }

    #[test]
    fn test_a_program_with_no_readable_name_is_not_read_out_as_its_identifier() {
        // The programs holding these out of the box on Windows 10 and 11 are
        // Store apps whose identifiers look like
        // AppXbx2ce4vcxjdhff3d1ms66qqzk12zn827. Reading that to somebody is
        // worse than saying nothing specific.
        let said = row(
            DefaultKind::Mail,
            &Status::AnotherProgram {
                prog_id: "AppXbx2ce4vcxjdhff3d1ms66qqzk12zn827".to_string(),
                program_name: None,
            },
        );

        assert!(!said.contains("AppX"), "{said}");
        assert!(said.contains("another program"), "{said}");
    }

    #[test]
    fn test_a_program_with_a_real_name_is_named() {
        let said = row(
            DefaultKind::Calendar,
            &Status::AnotherProgram {
                prog_id: "Outlook.File.ics.15".to_string(),
                program_name: Some("Outlook".to_string()),
            },
        );

        assert!(said.contains("Outlook"), "{said}");
    }

    #[test]
    fn test_the_three_windows_has_no_answer_for_say_so_rather_than_being_left_out() {
        // Leaving them out would leave somebody looking for tasks in this list
        // wondering whether they had missed it.
        let said = row(DefaultKind::Tasks, &Status::NoWindowsSlot);

        assert!(said.contains("Tasks"), "{said}");
        assert!(said.contains("no default program"), "{said}");
    }

    #[test]
    fn test_a_check_that_failed_says_why_rather_than_only_that_it_failed() {
        let said = row(
            DefaultKind::Mail,
            &Status::Undetermined {
                reason: "the registry key could not be opened".to_string(),
            },
        );

        assert!(said.contains("could not be checked"), "{said}");
        assert!(said.contains("registry key"), "{said}");
    }

    #[test]
    fn test_holding_one_slot_of_two_is_said_as_both_halves() {
        // Calendar has two slots and somebody really can hold .ics here while
        // webcal goes to a browser. Rounding that either way is a false
        // statement about what will happen when they click a link.
        let said = row(
            DefaultKind::Calendar,
            &Status::Partial {
                held: vec![".ics files".to_string()],
                not_held: vec!["webcal: links".to_string()],
            },
        );

        assert!(said.contains(".ics files"), "{said}");
        assert!(said.contains("webcal: links"), "{said}");
    }

    #[test]
    fn test_the_button_does_not_promise_to_do_what_windows_forbids() {
        // Windows has not let a program make itself the default since Windows
        // 8. A button saying "Set as default" would be a button that cannot do
        // what it says.
        let label = button_label();

        assert!(
            !label.to_lowercase().contains("set as"),
            "the button promises something Windows does not allow: {label}"
        );
        assert!(label.contains("Windows"), "{label}");
    }

    #[test]
    fn test_the_startup_check_says_nothing_when_everything_is_ours() {
        // Somebody who is already the default does not need telling at every
        // start, and a message that appears every time is one that gets
        // dismissed without being read.
        let found = vec![
            (DefaultKind::Mail, Status::Ours),
            (DefaultKind::Tasks, Status::NoWindowsSlot),
        ];

        assert!(what_the_startup_check_found(&found).is_empty());
    }

    #[test]
    fn test_the_startup_check_names_what_is_not_ours_and_where_to_change_it() {
        let found = vec![
            (
                DefaultKind::Mail,
                Status::AnotherProgram {
                    prog_id: "x".to_string(),
                    program_name: None,
                },
            ),
            (DefaultKind::Contacts, Status::Ours),
        ];

        let said = what_the_startup_check_found(&found);

        assert!(said.contains("Email"), "{said}");
        assert!(!said.contains("Contacts"), "{said}");
        assert!(said.contains("Settings"), "{said}");
    }

    #[test]
    fn test_a_kind_windows_keeps_no_default_for_is_not_reported_as_missing() {
        // Tasks, reminders and notes can never be "ours", so reporting them at
        // startup would be a complaint about something nobody can fix.
        let found = vec![
            (DefaultKind::Tasks, Status::NoWindowsSlot),
            (DefaultKind::Notes, Status::NoWindowsSlot),
        ];

        assert!(what_the_startup_check_found(&found).is_empty());
    }

    #[test]
    fn test_a_list_reads_as_a_sentence() {
        assert_eq!(a_list_of_static(&["one"]), "one");
        assert_eq!(a_list_of_static(&["one", "two"]), "one and two");
        assert_eq!(
            a_list_of_static(&["one", "two", "three"]),
            "one, two and three"
        );
    }
}
