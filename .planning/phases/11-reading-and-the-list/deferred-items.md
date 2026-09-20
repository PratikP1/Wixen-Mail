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

## Found by 11-07 task 2, 2026-09-19

- **`test_every_guard_record_still_names_one_place_in_the_tree` is blind for a
  whole file while any one record of that file looks mid-measurement.** Its
  exemption, `mid_measurement`, is keyed on the file: when any record's `after`
  is in the tree and its `before` is not, every record of that file whose
  `before` is missing is passed over. After 11-07 rewrote the cursor handler,
  11-06's record on that handler had an `after` (the handler without the
  refresh line) that matched the new text by coincidence, and the check passed
  while six records of `wx_app.rs` had lost their `before`. Found by a one-place
  count over the TOML by hand; the six were rewritten and measured, and are in
  the 11-07 summary. The fix is an exemption per record, and a pass through it
  saying so, which is guardrail 4; ledger 545. Not fixed here: `house_style.rs`
  is named by 27 records, and a test changed there is a plan of its own.
- **A re-read of the folder keeps the cursor and not the selection.** The
  control keeps a virtual list's selected rows by position, so the watch's
  re-read after a delete or an arrival leaves the selection on whichever
  messages now sit at those positions; 11-06.1 made the cursor follow its
  message by identity and the selection has no such rule. Said in the
  changelog's known limitations. A rule for it is work for whoever next
  changes what the load arm does, likely 11-07.1 or 11-08.
- **A stale `a_set_leaving` after a local delete's error.** `delete_if_local`'s
  `Err` path sends a spoken `StatusUpdated` rather than a refusal, so a set
  whose local delete errs on one message waits for a row that will not leave
  until the next set replaces it or a refusal arrives; the rows that did leave
  are landed then. Reachable only when the local store errs mid-set. 11-07.1,
  which completes a delete here first, changes this path.

## Found by 11-11.0 task 2, 2026-09-20

- **The plain-text reader's conversation heading numbers a single message
  too.** `reader_text::conversation` (`src/presentation/reader_text.rs`, the
  heading built from `position + 1`) heads one message "1. Message from ..."
  the way the page did until 11-11.0, and its title line counts the messages
  for one as well. #90 is about the formatted view, and that is what 11-11.0
  changed; the text reader is a separate surface, `reader_text.rs` is named by
  13 records and holds 125 tests, and two of them find "1. Message from" in a
  two-part conversation. Whoever next changes the text reader's heading drops
  the number for one message the same way, with the two tests rewritten in
  place. Not a ledger entry: nothing is wrong that a reader would call a
  defect, and it is said here so it is not found again by ear.
