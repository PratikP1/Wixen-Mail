# Deferred items, phase 11

Out-of-scope findings met while executing a plan, recorded here and, where
they are defects, in `.planning/WINDOWS.md`, not fixed in the plan that found
them.

## Found by 11-06 task 3, 2026-09-18

- **`docs/USER_GUIDE.md` names three keys that are not bound.** Its Message
  Actions list under Keyboard Shortcuts says `S` stars a message, and its
  Navigation list says `N` and `P` move to the next and previous unread
  message. Star is `Ctrl+Shift+S` and the unread moves are `Ctrl+U` and
  `Ctrl+Shift+U` (`docs/KEYBOARD_SHORTCUTS.md:553` and `:555`), and no
  `KEY_DOWN` closure on the message list answers 78, 80 or 83
  (`grep -n 'Some(78)\|Some(80)\|Some(83)' src/presentation/wx_app.rs` answers
  nothing). 11-06 corrected the two read-state lines beside them by dating and
  left these, since a plan's page changes are the ones its keys owe; 11-12's
  read of the four pages as one is where they belong. Not a ledger entry: a
  page claim, not a defect in the program.
