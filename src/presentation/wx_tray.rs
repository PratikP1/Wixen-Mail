//! The notification area icon, so closing the window need not quit.
//!
//! The words and the ids are here as data. Turning them into a real menu is
//! the last part of the file, and everything a person reads or presses is
//! decided before that, where it can be checked without a window on screen.
//!
//! # It has to be reachable without a mouse
//!
//! Windows puts the notification area in the keyboard's reach: `Win+B` moves
//! focus there and the arrow keys walk along it. From an icon, the
//! Applications key or `Shift+F10` asks for its menu, and that is the request
//! this module answers by handing wxWidgets a menu rather than by listening
//! for a mouse button. Once the menu is open it is an ordinary menu: arrow
//! keys, one letter per item, and Enter.
//!
//! One part of that has been read in the source and one part has not. What
//! wxWidgets does is settled: `wxTaskBarIcon::PopupMenu` calls
//! `SetForegroundWindow` before showing the menu, so the menu takes focus and
//! the keyboard reaches it. What the Windows shell sends when the keyboard
//! asks for the menu has not been run here. wxWidgets never asks the shell for
//! the newer notification protocol except while a balloon is on screen, and
//! its window procedure recognises mouse messages only, so this rests on the
//! shell's older behaviour of reporting a keyboard request as a right click.
//! If that turns out not to hold, the icon is still readable and the fix is a
//! handler here, not a change to the words.
//!
//! # Nothing is said by the picture alone
//!
//! A tray icon is a small picture, which is no use to the people this
//! application is for. Everything it communicates is in its tooltip, so that
//! whatever the picture may come to show, the same thing is there in words.
//! Windows shows that text as the icon's label in the notification area and
//! that is what a screen reader has to work from; it has not been listened to
//! here, which is a thing to check with NVDA rather than a thing to assume.

use crate::common::{Error, Result};
use std::rc::Rc;
use wxdragon::prelude::*;

/// The command ids the tray raises.
///
/// Fed in by the frame rather than named here, so the same thing done from the
/// tray and from the menu bar runs one piece of code. A second id for "check
/// mail, but from the tray" would be a second thing to keep working.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrayCommands {
    /// Bring the main window back and put focus in it.
    pub open_window: Id,
    /// Start a new message.
    pub new_message: Id,
    /// Fetch waiting mail now.
    pub check_mail: Id,
    /// Show every account's inbox in one list.
    pub all_inboxes: Id,
    /// Leave Wixen Mail for real, rather than closing the window.
    pub quit: Id,
}

/// One command on the tray menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrayEntry {
    /// The command the frame already handles.
    pub id: Id,
    /// The words shown, with `&` in front of the letter that runs it. This
    /// carries the whole meaning: a menu bar item can lean on the status bar
    /// to finish its sentence and a tray menu has no status bar to lean on.
    pub label: &'static str,
    /// A sentence saying what choosing it does.
    ///
    /// wxWidgets keeps this on the item. A frame would show it in the status
    /// bar; whether anything surfaces it from a tray menu has not been
    /// checked, so it is written as though nobody will read it and the label
    /// stands on its own.
    pub help: &'static str,
}

/// One line of the tray menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayLine {
    /// A command a person can choose.
    Command(TrayEntry),
    /// The rule that divides working in Wixen Mail from leaving it.
    Separator,
}

/// The tray menu, in the order it is read.
pub fn tray_menu(commands: TrayCommands) -> Vec<TrayLine> {
    vec![
        TrayLine::Command(TrayEntry {
            id: commands.open_window,
            label: "&Open Wixen Mail",
            help: "Show the Wixen Mail window again",
        }),
        TrayLine::Command(TrayEntry {
            id: commands.new_message,
            label: "&New Message",
            help: "Write a new email message",
        }),
        TrayLine::Command(TrayEntry {
            id: commands.check_mail,
            label: "Check &Mail",
            help: "Fetch any mail waiting on the server",
        }),
        TrayLine::Command(TrayEntry {
            id: commands.all_inboxes,
            label: "All &Inboxes",
            help: "Show the mail from every account's inbox in one list",
        }),
        TrayLine::Separator,
        TrayLine::Command(TrayEntry {
            id: commands.quit,
            label: "&Quit",
            help: "Close Wixen Mail. It stops checking for mail until you start it again.",
        }),
    ]
}

/// What the application calls itself out loud.
const APP_NAME: &str = "Wixen Mail";

/// What the icon says about itself when a person points at it or arrows onto
/// it in the notification area.
///
/// This is the icon's text equivalent, and the only one it has. A picture that
/// means "you have mail" and says nothing means nothing to somebody who cannot
/// see it, so whatever the icon shows has to be here in words as well.
///
/// `unread` is `None` until a check has finished. That is a different thing
/// from none waiting and is worded differently, because claiming an empty
/// inbox before looking is a claim the application cannot make yet.
///
/// Kept short on purpose. Windows keeps a fixed amount of a tooltip and
/// wxWidgets copies into it with `wxStrlcpy`, which cuts the end off without
/// saying so, and the end is where the count is. The test below holds it to
/// the length that survives.
pub fn tooltip(unread: Option<usize>) -> String {
    match unread {
        None => APP_NAME.to_string(),
        Some(0) => format!("{APP_NAME}: No unread messages"),
        Some(1) => format!("{APP_NAME}: 1 unread message"),
        Some(waiting) => format!("{APP_NAME}: {waiting} unread messages"),
    }
}

// ── Putting it on screen ─────────────────────────────────────────────────────

/// The icon in the notification area, and the menu it raises.
///
/// Holding this value is what keeps the icon there. Dropping it takes the icon
/// away and frees the menu, in that order, which is why the two live together
/// rather than being handed out separately.
pub struct TrayIcon {
    /// wxWidgets keeps a bare pointer to `menu`, so the icon has to be taken
    /// down before the menu is freed. Declared first for that reason: this
    /// type's `Drop` runs first and destroys the icon, and the fields are then
    /// dropped in the order they are written.
    icon: TaskBarIcon,
    /// The menu the icon raises. wxWidgets copies it every time the menu is
    /// shown and deletes the copy, never this one, so this one has to outlive
    /// the icon and is not freed twice.
    menu: Menu,
    /// The picture, kept so the tooltip can be rewritten later. Windows sets
    /// the picture and the tooltip in the same call, so changing one means
    /// having the other to hand.
    picture: Bitmap,
}

impl TrayIcon {
    /// Put Wixen Mail in the notification area.
    ///
    /// `raise` is handed the command id of whatever was chosen, and is meant to
    /// be the same dispatch the menu bar uses. Nothing is decided here: the
    /// tray offers commands and the frame carries them out, so a command
    /// reached two ways behaves one way.
    ///
    /// Clicking the icon raises the same id as the first line of the menu, for
    /// the same reason.
    ///
    /// # Platforms
    ///
    /// Written for Windows, which is where this application runs. Linux gets
    /// an equivalent through GTK's status icon, and how it behaves is up to
    /// the desktop. macOS shows a menu bar item and delivers no click events
    /// at all, so on macOS the menu is the only way in and the click handler
    /// below is not compiled.
    ///
    /// # An upstream gap, named rather than hidden
    ///
    /// wxdragon panics if wxWidgets will not make the icon, so that one
    /// failure cannot be turned into an error here and would take the
    /// application down with it. Everything after that point is reported
    /// properly.
    pub fn new(
        picture: Bitmap,
        commands: TrayCommands,
        unread: Option<usize>,
        raise: impl Fn(Id) + 'static,
    ) -> Result<Self> {
        let icon = TaskBarIcon::builder().build();

        // One sink, shared by the menu and the click, so both spellings of
        // "open Wixen Mail" end in the same place.
        let raise = Rc::new(raise);
        icon.on_menu({
            let raise = Rc::clone(&raise);
            move |event| raise(event.get_id())
        });

        // Not on macOS, where wxWidgets delivers no taskbar mouse events.
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        icon.on_left_down({
            let raise = Rc::clone(&raise);
            move |_| raise(commands.open_window)
        });

        let mut tray = TrayIcon {
            icon,
            menu: build_menu(commands),
            picture,
        };

        // The menu is handed over rather than popped up by hand, so wxWidgets
        // shows it whenever the platform decides the icon was asked for. A
        // handler bound to a mouse button instead would look finished and
        // leave nothing for somebody who never touches a mouse.
        //
        // Handed over after the menu has reached the field it lives in, so
        // what wxWidgets keeps a bare pointer to is that field and not some
        // earlier copy of the wrapper.
        tray.icon.set_popup_menu(&mut tray.menu);

        tray.show(unread)?;
        Ok(tray)
    }

    /// Show the icon, saying what is waiting.
    ///
    /// Also the way to change what it says: Windows takes the picture and the
    /// words in one call, so showing it again with a new count is how the
    /// count is updated.
    pub fn show(&self, unread: Option<usize>) -> Result<()> {
        if self.icon.set_icon(&self.picture, &tooltip(unread)) {
            return Ok(());
        }
        Err(Error::Other(
            "Windows would not put the Wixen Mail icon in the notification area, \
             so closing the window would leave no way back to it"
                .to_string(),
        ))
    }

    /// Take the icon out of the notification area, keeping it ready to show
    /// again.
    ///
    /// This is what a handler raised by the tray itself should call. Dropping
    /// the whole value from inside one of its own handlers would free the
    /// object the event is still running in.
    ///
    /// Asking twice is not an error. wxWidgets reports removing an icon that
    /// was never added as a failure, and "it is not there" is the answer the
    /// caller wanted either way.
    pub fn remove(&self) -> Result<()> {
        if !self.icon.is_icon_installed() || self.icon.remove_icon() {
            return Ok(());
        }
        Err(Error::Other(
            "Windows would not take the Wixen Mail icon out of the notification \
             area, so it may still be shown"
                .to_string(),
        ))
    }

    /// Whether the icon is in the notification area at the moment.
    pub fn is_showing(&self) -> bool {
        self.icon.is_icon_installed()
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        // The icon goes first because wxWidgets is holding a bare pointer to
        // the menu, which is freed as the fields drop straight after this.
        //
        // wxWidgets has a deferred `Destroy()` for exactly the case of an
        // event handler destroyed while one of its own events is in flight.
        // wxdragon does not expose it, and this call deletes the object there
        // and then, so a tray handler must call `remove()` and leave the value
        // to be dropped once the event is over.
        self.icon.remove_icon();
        self.icon.destroy();
    }
}

/// Turn the menu description into a menu wxWidgets can show.
///
/// Separate from [`tray_menu`] so that everything a person reads or presses is
/// decided in a function that needs no window, and this is left with nothing to
/// get wrong.
fn build_menu(commands: TrayCommands) -> Menu {
    let menu = Menu::builder().build();
    for line in tray_menu(commands) {
        match line {
            TrayLine::Command(entry) => {
                menu.append(entry.id, entry.label, entry.help, ItemKind::Normal);
            }
            TrayLine::Separator => menu.append_separator(),
        }
    }
    menu
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The most text Windows keeps of a notification area tooltip.
    ///
    /// `szTip` in `NOTIFYICONDATA` holds 128 characters including the
    /// terminator, and wxWidgets copies into it with `wxStrlcpy`, which
    /// truncates rather than refusing.
    const TOOLTIP_LIMIT: usize = 127;

    /// Ids that cannot be confused with each other or with a real wxWidgets id,
    /// so a test that reads the wrong field says so instead of passing.
    const COMMANDS: TrayCommands = TrayCommands {
        open_window: 9001,
        new_message: 9002,
        check_mail: 9003,
        all_inboxes: 9004,
        quit: 9005,
    };

    fn commands_in_order(lines: &[TrayLine]) -> Vec<Id> {
        entries(lines).map(|entry| entry.id).collect()
    }

    fn entries(lines: &[TrayLine]) -> impl Iterator<Item = &TrayEntry> {
        lines.iter().filter_map(|line| match line {
            TrayLine::Command(entry) => Some(entry),
            TrayLine::Separator => None,
        })
    }

    /// The letter a label claims, when it claims one.
    ///
    /// The same rule `tests/wired.rs` reads the menu bar by, and the same rule
    /// Windows itself follows: `&&` is a literal ampersand, not a mnemonic.
    fn alt_key_of(label: &str) -> Option<char> {
        let letters: Vec<char> = label.chars().collect();
        let mut at = 0;
        while at < letters.len() {
            if letters[at] != '&' {
                at += 1;
                continue;
            }
            match letters.get(at + 1) {
                Some('&') => at += 2,
                Some(letter) if letter.is_alphanumeric() => {
                    return Some(letter.to_ascii_lowercase());
                }
                _ => at += 1,
            }
        }
        None
    }

    /// No two entries on the tray menu claim the same letter.
    ///
    /// Inside an open menu a mnemonic is meant to be one keystroke: press the
    /// letter and the item runs. Two items sharing a letter quietly downgrades
    /// both to press, press again, then Enter, and nothing tells the person
    /// that the letter they were taught no longer finishes the job. It is a
    /// fault that only shows up under the keyboard, which is how this
    /// application is meant to be used.
    ///
    /// `tests/wired.rs` makes this check over the menu bar by reading the
    /// source of `wx_app.rs`. The tray menu is data rather than a builder
    /// chain, so it can be asked directly.
    #[test]
    fn test_no_two_entries_on_the_tray_menu_claim_the_same_letter() {
        let lines = tray_menu(COMMANDS);

        let mut claimed: Vec<(char, &str)> = Vec::new();
        for entry in entries(&lines) {
            let Some(letter) = alt_key_of(entry.label) else {
                continue;
            };
            if let Some((_, earlier)) = claimed.iter().find(|(seen, _)| *seen == letter) {
                panic!(
                    "\"{}\" and \"{earlier}\" both claim {letter}, so pressing {letter} \
                     runs neither and cycles between them instead",
                    entry.label
                );
            }
            claimed.push((letter, entry.label));
        }
    }

    /// Every entry claims a letter.
    ///
    /// An entry with no mnemonic is still reachable, by arrowing down to it,
    /// so this is not about it being usable. It is about it being the odd one
    /// out: four items answer to a letter and the fifth silently does not, and
    /// the only way to find that out is to press the letter and watch nothing
    /// happen.
    #[test]
    fn test_every_entry_on_the_tray_menu_claims_a_letter() {
        for entry in entries(&tray_menu(COMMANDS)) {
            assert!(
                alt_key_of(entry.label).is_some(),
                "\"{}\" claims no letter, so it is the one item on this menu that \
                 cannot be run with a single keystroke",
                entry.label
            );
        }
    }

    /// No entry promises an accelerator.
    ///
    /// A label may carry a shortcut after a tab, and on the menu bar that is
    /// worth doing because the key really does work from the window. The tray
    /// menu is reached when the window may be closed, and a key printed beside
    /// a command that the closed window cannot receive is a promise the
    /// application does not keep.
    #[test]
    fn test_no_tray_entry_prints_a_key_the_closed_window_could_not_receive() {
        for entry in entries(&tray_menu(COMMANDS)) {
            assert!(
                !entry.label.contains('\t'),
                "\"{}\" prints a shortcut, which reads as a key that works from \
                 here and does not",
                entry.label
            );
        }
    }

    /// Every entry says in words what choosing it does.
    ///
    /// The label is short by necessity and the help string is where the rest
    /// of the meaning lives: "Quit" on its own does not say that mail stops
    /// arriving. Nothing has been shown to read a tray menu's help string out
    /// loud, so this is not the guard that makes the menu usable. It is the
    /// guard that keeps the sentence written down, both for the moment
    /// something does read it and for whoever changes these words next.
    #[test]
    fn test_every_tray_entry_says_what_choosing_it_does() {
        for entry in entries(&tray_menu(COMMANDS)) {
            assert!(
                entry.help.len() > entry.label.len(),
                "\"{}\" is described by \"{}\", which says no more than the label does",
                entry.label,
                entry.help
            );
        }
    }

    /// No two entries raise the same command.
    ///
    /// Two lines on one id means choosing either does whatever the handler was
    /// written for, which is one of them.
    #[test]
    fn test_no_two_tray_entries_raise_the_same_command() {
        let ids = commands_in_order(&tray_menu(COMMANDS));
        for (index, id) in ids.iter().enumerate() {
            assert!(
                !ids[..index].contains(id),
                "two entries on the tray menu raise {id}, so one of them does the \
                 other's work"
            );
        }
    }

    /// The tooltip names the application before anything else.
    ///
    /// Somebody arrowing along the notification area hears one icon after
    /// another. The first words have to say which application this is, or the
    /// count arrives attached to nothing.
    #[test]
    fn test_the_tooltip_names_the_application_before_it_says_anything_else() {
        for unread in [None, Some(0), Some(1), Some(40)] {
            let said = tooltip(unread);
            assert!(
                said.starts_with("Wixen Mail"),
                "with {unread:?} unread the icon introduces itself as \"{said}\""
            );
        }
    }

    /// The count reaches the tooltip as text.
    ///
    /// This is the whole reason the tooltip is rebuilt rather than set once.
    /// Whatever the icon may come to show, nothing it communicates is allowed
    /// to exist only as a picture.
    #[test]
    fn test_the_unread_count_reaches_the_tooltip_as_words() {
        assert!(
            tooltip(Some(7)).contains('7'),
            "seven messages are waiting and the icon does not say so in words"
        );
    }

    /// One message is not counted in the plural.
    #[test]
    fn test_the_tooltip_counts_one_message_in_the_singular() {
        let said = tooltip(Some(1));
        assert!(
            said.contains("1 unread message") && !said.contains("messages"),
            "one message is announced as \"{said}\""
        );
    }

    /// An empty inbox is said in words rather than as a nought.
    ///
    /// "0 unread messages" is a sentence a person has to parse. "No unread
    /// messages" is one they hear.
    #[test]
    fn test_the_tooltip_says_an_empty_inbox_in_words() {
        let said = tooltip(Some(0));
        assert!(
            said.contains("No unread messages") && !said.contains('0'),
            "an empty inbox is announced as \"{said}\""
        );
    }

    /// Before the first check the tooltip does not claim an empty inbox.
    ///
    /// Not knowing and knowing there is nothing are different, and only one of
    /// them is true at startup. Saying "no unread messages" while the first
    /// check is still running is the kind of confident wrong answer that stops
    /// somebody opening the window.
    #[test]
    fn test_the_tooltip_does_not_claim_an_empty_inbox_before_the_first_check() {
        let said = tooltip(None);
        assert!(
            !said.to_lowercase().contains("unread"),
            "nothing has been checked yet and the icon already says \"{said}\""
        );
    }

    /// The tooltip fits in the space Windows keeps for it.
    ///
    /// wxWidgets truncates a long tooltip silently, and what would be cut is
    /// the end, which is where the count is. A mailbox with a preposterous
    /// number in it should shorten the wording, never lose it.
    #[test]
    fn test_the_tooltip_fits_the_space_windows_keeps_for_it() {
        for unread in [None, Some(0), Some(1), Some(usize::MAX)] {
            let said = tooltip(unread);
            assert!(
                said.chars().count() <= TOOLTIP_LIMIT,
                "with {unread:?} unread the tooltip is {} characters and Windows \
                 keeps {TOOLTIP_LIMIT}",
                said.chars().count()
            );
        }
    }

    /// The tray offers the four things somebody reaches for without opening
    /// the window, and the way out.
    ///
    /// Written first because the order is the whole design. Open comes first
    /// because it is what most tray clicks mean, Quit last and behind a rule
    /// because it is the one choice on this menu that cannot be taken back.
    #[test]
    fn test_the_tray_menu_offers_the_commands_it_promises_in_the_order_it_promises() {
        let lines = tray_menu(COMMANDS);

        assert_eq!(
            commands_in_order(&lines),
            vec![
                COMMANDS.open_window,
                COMMANDS.new_message,
                COMMANDS.check_mail,
                COMMANDS.all_inboxes,
                COMMANDS.quit,
            ],
            "the tray menu is not offering what it says it offers"
        );
    }
}
