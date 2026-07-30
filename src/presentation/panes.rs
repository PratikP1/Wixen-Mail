//! Where `F6` stops, and what it says when it gets there.
//!
//! Every module in the main window is a sidebar and a list: folders and
//! messages, calendars and events, task lists and tasks. `F6` moves between
//! those two and `Shift+F6` moves back, which is the Windows convention and
//! what somebody who cannot see the layout reaches for first.
//!
//! # Why this was worth writing down rather than doing inline
//!
//! `F6` was documented in `docs/KEYBOARD_SHORTCUTS.md`, described in three
//! comments in `wx_app.rs`, listed in a fourth as a key that was "taken", and
//! bound to nothing. The page injected into the message preview posted a
//! message back to the host when somebody pressed it, and the host had no
//! handler. Nobody noticed because there was nothing to notice: pressing it
//! did nothing, quietly, which is what an unimplemented shortcut and a working
//! one that lands you somewhere silent look like from the outside.
//!
//! So the announcement is not decoration. Moving focus without saying where it
//! went is the same experience as the key not working.
//!
//! # The preview is not one of these
//!
//! It never takes focus, so `F6` does not stop there. It is a WebView, which
//! hosts a browser out of process: once focus is inside, `Escape`, `F6` and
//! every menu accelerator are consumed there and reach nothing.

use crate::presentation::ui_types::PimModule;

/// One of the two places focus rests in the main window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    /// The tree down the side: folders, calendars, task lists.
    Sidebar,
    /// The list of things in whatever the sidebar has selected.
    List,
}

impl Pane {
    /// Both, in the order `F6` visits them.
    pub const ALL: [Pane; 2] = [Pane::Sidebar, Pane::List];

    /// The next one round, which is where `F6` goes.
    pub const fn next(self) -> Self {
        match self {
            Pane::Sidebar => Pane::List,
            Pane::List => Pane::Sidebar,
        }
    }

    /// The previous one, which is where `Shift+F6` goes.
    ///
    /// The same as [`next`](Self::next) while there are two panes. Written
    /// separately because they are different questions, and a third pane
    /// would make them different answers.
    pub const fn previous(self) -> Self {
        match self {
            Pane::Sidebar => Pane::List,
            Pane::List => Pane::Sidebar,
        }
    }

    /// What to announce on arriving, in this module.
    ///
    /// The same words the control carries as its accessible name, so what
    /// `F6` says and what the screen reader says next are the same thing. A
    /// test below checks they have not drifted apart.
    pub const fn spoken(self, module: PimModule) -> &'static str {
        match (module, self) {
            (PimModule::Mail, Pane::Sidebar) => "Mail folders",
            (PimModule::Mail, Pane::List) => "Messages",
            (PimModule::Calendar, Pane::Sidebar) => "Calendars",
            (PimModule::Calendar, Pane::List) => "Calendar events",
            (PimModule::Contacts, Pane::Sidebar) => "Contact groups",
            (PimModule::Contacts, Pane::List) => "Contacts",
            (PimModule::Reminders, Pane::Sidebar) => "Reminder groups",
            (PimModule::Reminders, Pane::List) => "Reminders",
            (PimModule::Tasks, Pane::Sidebar) => "Task lists",
            (PimModule::Tasks, Pane::List) => "Tasks",
            (PimModule::Notes, Pane::Sidebar) => "Note folders",
            (PimModule::Notes, Pane::List) => "Notes",
        }
    }
    /// What to announce on arriving, including when there is nothing here.
    ///
    /// [`spoken`](Self::spoken) names the pane and stops. That is enough when
    /// the pane has something in it, because the screen reader reads the item
    /// focus landed on straight after. When it is empty there is no such item,
    /// so the name is the whole announcement and arriving sounds identical to
    /// the key doing nothing.
    pub fn arrival(self, module: PimModule, holding: Holding) -> String {
        let name = self.spoken(module);
        match holding {
            Holding::NoAccount => {
                format!("{name}, no account yet. Press Ctrl+A to add one")
            }
            Holding::Items(0) => format!("{name}, empty"),
            Holding::Items(count) => format!("{name}, {count}"),
            Holding::Unknown => name.to_string(),
        }
    }
}

/// What is in a pane at the moment focus arrives at it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holding {
    /// This many rows, which may be none.
    Items(usize),
    /// Nothing, and nothing can be here until a mail account is added.
    ///
    /// Separate from `Items(0)` because they call for different words. An
    /// empty folder is a fact about the mailbox; an empty application is a
    /// setup step nobody has been told about.
    NoAccount,
    /// Nobody counted this pane, so the announcement says only its name.
    ///
    /// Honest rather than tidy. Guessing "empty" for a pane whose contents
    /// were never read would say something false in the one place somebody
    /// has no way to check it.
    Unknown,
}

/// Where `F6` should go, given where focus is now.
///
/// `None` for "focus is somewhere else entirely", which happens when it is on
/// the toolbar, on a module button, or nowhere the application put it. Landing
/// on the list from there is right: it is the pane somebody works in, and it
/// is never the wrong answer to the question "get me back into the content".
pub const fn from(here: Option<Pane>, going: Direction) -> Pane {
    match (here, going) {
        (Some(pane), Direction::Forward) => pane.next(),
        (Some(pane), Direction::Back) => pane.previous(),
        (None, _) => Pane::List,
    }
}

/// Which way round the cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// `F6`.
    Forward,
    /// `Shift+F6`.
    Back,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arriving_somewhere_empty_says_so() {
        // The whole reason F6 felt broken. The key fired, focus moved, and the
        // pane it moved to had nothing in it, so the screen reader had nothing
        // to read after our one word. Silence on arrival and a key that does
        // nothing are the same experience from the outside.
        let spoken = Pane::List.arrival(PimModule::Mail, Holding::Items(0));

        assert!(spoken.contains("Messages"), "{spoken}");
        assert!(spoken.contains("empty"), "{spoken}");
    }

    #[test]
    fn test_an_empty_mailbox_says_what_would_fill_it() {
        // "Mail folders, empty" is true and useless. Nothing can appear in
        // either mail pane until an account exists, so the announcement says
        // the one thing that changes that rather than leaving somebody to
        // wonder whether the pane is broken or the mailbox is quiet.
        for pane in Pane::ALL {
            let spoken = pane.arrival(PimModule::Mail, Holding::NoAccount);

            assert!(spoken.contains("no account"), "{spoken}");
            assert!(spoken.contains("Ctrl+A"), "{spoken}");
        }
    }

    #[test]
    fn test_a_pane_whose_contents_are_not_known_just_names_itself() {
        // Not every pane has a count to hand. Saying the name alone is what
        // this did before and is still correct; inventing "empty" for a pane
        // nobody counted would be worse than saying less.
        let spoken = Pane::Sidebar.arrival(PimModule::Tasks, Holding::Unknown);

        assert_eq!(spoken, "Task lists");
    }

    #[test]
    fn test_f6_moves_off_the_pane_you_are_on() {
        // The one thing it has to do. Landing back where you started is the
        // same as the key doing nothing, which is what it did before.
        for pane in Pane::ALL {
            assert_ne!(pane.next(), pane, "{pane:?} did not move");
            assert_ne!(pane.previous(), pane, "{pane:?} did not move");
        }
    }

    #[test]
    fn test_pressing_it_twice_brings_you_back() {
        for pane in Pane::ALL {
            assert_eq!(pane.next().next(), pane);
        }
    }

    #[test]
    fn test_focus_somewhere_else_lands_in_the_list() {
        // From the toolbar, a module button, or nowhere. The list is where
        // the work is, so it is the safe answer to "get me into the content".
        assert_eq!(from(None, Direction::Forward), Pane::List);
        assert_eq!(from(None, Direction::Back), Pane::List);
    }

    #[test]
    fn test_every_module_names_both_of_its_panes() {
        for module in PimModule::ALL {
            for pane in Pane::ALL {
                let said = pane.spoken(module);
                assert!(
                    !said.is_empty(),
                    "{module:?} {pane:?} has nothing to announce"
                );
            }
        }
    }

    #[test]
    fn test_no_two_panes_in_a_module_sound_the_same() {
        // "Tasks" for both the sidebar and the list would make F6 sound like
        // it had not moved.
        for module in PimModule::ALL {
            assert_ne!(
                Pane::Sidebar.spoken(module),
                Pane::List.spoken(module),
                "both panes of {module:?} announce the same thing"
            );
        }
    }

    #[test]
    fn test_what_f6_says_is_what_the_control_is_called() {
        // Two places have to agree: the name given to the control with
        // set_accessible_name, and the name announced on arriving. If they
        // drift, F6 says "Tasks" and the screen reader then says something
        // else about the same control, which is worse than silence.
        let sources = [
            "src/presentation/wx_app.rs",
            "src/presentation/wx_calendar_module.rs",
            "src/presentation/wx_contacts_module.rs",
            "src/presentation/wx_notes_module.rs",
            "src/presentation/wx_reminders_module.rs",
            "src/presentation/wx_tasks_module.rs",
        ];
        let code: String = sources
            .iter()
            .map(|path| std::fs::read_to_string(path).expect(path))
            .collect();

        for module in PimModule::ALL {
            for pane in Pane::ALL {
                let name = pane.spoken(module);
                assert!(
                    code.contains(&format!("set_accessible_name(&tree, \"{name}\")"))
                        || code.contains(&format!("\"{name}\");")),
                    "F6 announces \"{name}\" for {module:?} {pane:?}, but no control is given \
                     that name"
                );
            }
        }
    }
}
