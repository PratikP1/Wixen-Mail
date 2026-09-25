---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 1
subsystem: presentation, application::editing
status: complete
tags: [undo, redo, edit-menu, keyboard, accessibility, GAP-02]
requires: []
provides:
  - EditCommand::Undo and EditCommand::Redo with an answer for every place
  - presentation::text_undo, the box's own one step reached through EM_CANUNDO and EM_UNDO
  - keep_undo_and_redo_honest_on_the_menu, greying on open and offering again on close
  - Cut as one step the box can undo
affects: [13-05, 13-06, 13-07, 13-08, 13-09]
tech-stack:
  added: []
  patterns:
    - "grey a menu item on open and offer it again on close, so its key is never swallowed"
key-files:
  created:
    - src/presentation/text_undo.rs
    - tests/undo_reaches_the_text.rs
  modified:
    - src/application/editing.rs
    - src/presentation/mod.rs
    - src/presentation/wx_app.rs
    - tests/undo_send_is_where_somebody_looks.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "Tasks 2 and 3's code landed as one red and one green: &Undo collides with &Undo Send on U in wired's letter check, and Undo first reddens the old Undo Send reading, so no commit between them could be green."
  - "Cut goes through text_undo::remove_the_selection (write_text of nothing over the chosen words) instead of replace, because replace left Undo nothing to put back, measured on both kinds of box."
  - "Undo and Redo sentences are spoken and shown on the status bar; the other edit sentences are spoken only, which the plan's premise had wrong."
  - "With focus nowhere known, Undo and Redo say to reach a box with Tab or F6 and do not name a list, where they would refuse too."
metrics:
  duration: about 3 hours
  completed: 2026-09-24
actuals:
  tokens: 12258
  tasks: 3
  commits: 5
---

# Phase 13 Plan 01: Undo and Redo on the Edit menu Summary

Edit, Undo (`Ctrl+Z`, U) and Edit, Redo (`Ctrl+Y`, R) now open the main window's Edit menu
and act on the note title, the note body and the contacts search through the box's own one
step. While a menu is open they are greyed when the box has nothing for them, and as it closes
both are offered again, so the keys always reach the handler and always say something. Undo
Send is third, still on `Ctrl+Shift+Z`, on the letter N. Cut now takes words out as one step
the box can undo, because the measurement found the old Cut left Undo nothing to put back.

`actuals.tokens` is chars/4 over the added lines under `src`, `tests`, `guards` and `docs`
against `45049360`. With `.planning` included it is 56,747, most of it the rewritten
`stopped_at` and focus lines of `STATE.md`, which count whole.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `22aea967` | red | Undo and Redo in `editing.rs`'s tests, the members stubbed to refuse in silence; 5 named | 159 s |
| `6956b949` | green | `what_to_do` for Undo and Redo per place, the two nothing-to-do sentences | 76 s |
| `0d46dfeb` | red | `tests/undo_reaches_the_text.rs` against a stubbed `text_undo` and binding, and the Undo Send reading rewritten; 13 named | 182 s (refused once at 232 s, below) |
| `fe87ae00` | green | `text_undo`, the ids, the items, the dispatch, the open and close handlers, Cut through `remove_the_selection`, Undo Send relabelled, four records, the shortcuts page | 203 s (refused once at 233 s, below) |
| this commit | docs | the guide, the changelog, the ledger, this summary, the four marks | |

**Test counts, taken again, not quoted from the plan.** `cargo test --lib application::editing::`
13 (9 at the start, 4 added, one renamed from `test_a_text_box_does_all_four_itself` to
`test_a_text_box_does_every_command_itself`). `cargo test --test undo_reaches_the_text` 15.
`cargo test --test wired` 77 before and 77 after. `cargo test --test undo_send_is_where_somebody_looks`
7 before and 7 after. `cargo test --test house_style` 74. `src/presentation/wx_app.rs` 199
tests, unchanged.

**The letters on Edit**, read again with Grep over the menu's labels: U (Undo), R (Redo), N
(Undo Send), T (Cut), C (Copy), P (Paste), A (Select All), S (Search), V (Save This Search),
each once.

## What was measured on a built control

Each with `EM_UNDO` sent straight to the box, so the measurement does not lean on the module
it describes. "beta" chosen in "alpha beta":

| What | One-line box | Box of several lines |
|------|--------------|----------------------|
| Cut as it was, `replace(from, to, "")`, then Undo | cut left `"alpha "`, undo left `"alpha "` | the same |
| `write_text("")` over the chosen words, then Undo | cut left `"alpha "`, undo left `"alpha beta"` | the same |
| `WM_CLEAR`, then Undo (probe only, not kept) | cut left `"alpha "`, undo left `"alpha beta"` | the same |
| Paste as `write_text` after `"alpha "`, then Undo | | paste left `"alpha pasted"`, undo left `"alpha "` |
| `set_value`, then `EM_CANUNDO` | | `false` |

So the old Cut followed by Undo put nothing back, and Undo would have said "Undone" over a box
that did not change. Cut now goes through `text_undo::remove_the_selection`, the write of
nothing, which the target reads as putting the words back in both boxes; the old shape stays
measured in `test_the_old_cut_left_nothing_for_undo_to_put_back`, so the day it stops holding
is said. And a value the program writes, such as another note opened into the body, leaves
nothing to undo, so Ctrl+Z after choosing a note does not bring the last one's words back.

## Guard records

Four written or rewritten, and one `scripts/guards.sh --remeasure` call over six, 132 s,
every one reddening exactly what it names:

| Record | Red |
|--------|-----|
| Undo Send is on the Edit menu after Undo and Redo, not off it and back where the tester found it (rewritten: name, `before` to "Undo Se&nd", the red list's renamed test) | `test_undo_and_redo_come_first_on_edit_and_undo_send_third_not_on_tools`, `test_the_reading_complains_when_undo_send_is_put_back_on_tools`, `test_the_reading_complains_when_undo_send_is_offered_on_both_menus` |
| the schedule routine's comment names Alt+H, the key that reaches it, not Alt+E (names the renamed file, re-measured) | its 2 named |
| answering a meeting with the hold off hands the answer to the server, not to the next Send of anything (names the renamed file, re-measured) | its 2 named |
| Undo asks the box whether it has anything to take back before it acts (new, `text_undo.rs`) | `test_undo_in_a_box_with_nothing_to_undo_says_so` |
| Undo and Redo are offered again whenever a menu closes, so their keys always speak (new, `wx_app.rs`) | `test_both_are_offered_again_whenever_the_menu_closes` |
| Redo is refused once something is typed after the Undo (new, `text_undo.rs`) | `test_redo_is_refused_once_something_is_typed_after_the_undo` |

The three new breaks were taken by hand on the target first, each reddening one reading and
nothing else, then written down. The arrived-since line at the head of `guards/guards.toml`
went from 291 to 294; the file holds 1,091 records. "Cut and Copy ask the box what is
selected", anchored inside the Cut arm, still names one place: the arm's first two lines did
not change, and `test_every_guard_record_still_names_one_place_in_the_tree` passed on the
green.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Tasks 2 and 3's code merged into one red and one green.**
Task 2 as written could not land green. Its items put "&Undo" beside "&Undo Send", both on U,
which `tests/wired.rs`'s `test_no_two_items_on_one_menu_claim_the_same_letter` refuses, and
Undo placed first reddens the old Undo Send reading, which task 3 was to rewrite. So the
target, the rewritten reading and the record-name check went into one red (`0d46dfeb`,
thirteen named), and the transport, the items, the relabel and the record rewrite into one
green (`fe87ae00`). The plan's acceptance line "wired passes with 77 ... so the letter and
shortcut readings accept the two new items" held at the green.

**2. [Rule 1 - Bug] Cut followed by Undo put nothing back.**
Found by the measurement the plan asked for. `replace(from, to, "")` is a removal and an
empty write, and the box's one step remembers the write. Fixed by `remove_the_selection`,
held by `test_cut_then_undo_puts_the_words_back`, red in `0d46dfeb`.

**3. [Rule 1 - Bug] Undo and Redo with focus nowhere known.**
The plan said to use the existing sentence naming "a box, a list or the sidebar". Undo in a
list refuses, so that sentence would send somebody to a refusal. The sentence is "Undo works
in a box you can type in. Tab or F6 moves between the parts of the window.", held by
`test_undo_and_redo_with_focus_nowhere_known_say_how_to_reach_a_box`.

**4. The red of task 1 needed stubs.** Adding two members to `EditCommand` breaks the
exhaustive matches in `what_to_do` and `do_an_edit_command`, so the red compiled with a stub
arm refusing Undo and Redo with an empty sentence and two no-op arms in `wx_app.rs`. That
put `wx_app.rs` in task 1's commits, which the plan did not list for task 1.

**5. The plan's "shown on the status bar ... as the other edit sentences are".** The other
edit sentences are spoken only. Undo and Redo's are spoken and shown, through
`say_what_the_undo_did`; the other four are unchanged.

**6. Two hook refusals, both fixed on the branch.** The red was refused once by
`test_no_comment_says_a_test_cannot_build_a_window`, over the target's head comment saying
"wxWidgets allows one application per process"; it now points at the label-menu target's
shape instead. The green was refused once by `the_planning_files_agree_with_themselves`,
because the ledger rows written for the documents commit sat unstaged in the working tree
without their JSON half; both halves were written before the retry.

**7. The non-Windows fallback.** The plan said it "says Undo is not offered on this
platform". Off Windows `can_undo` answers false, so Undo says there is nothing to undo, and
the module's head says the box's own keys are what undo there and a port needs its own
bridge. No sentence was added for a platform nothing ships to.

**8. One `sed` run while reading.** The brief forbids running `sed` at all, even to read. One
read of wxdragon's `menu_events.rs` in the registry went through `sed -n 1,140p ... | head -0`,
which printed nothing, before the same file was read with grep. No file was written by it,
and no tracked file was touched by any script; every edit was made with Edit or Write.

## Threat Flags

None beyond the plan's register. T-13-01 is held by the close-handler record, T-13-02 by the
Redo record and its companion, T-13-03 by reading the handle from the focused box at the
moment of the command and comparing the stored one before Redo uses it. No crate, feature or
`Cargo.toml` line was added (T-13-SC).

## Known Stubs

None. The two no-op arms for Undo and Redo on the preview page are unreachable in effect:
`what_to_do` refuses both in a read-only place with a sentence before the arm is reached.

## Ledger

Opened: 610 (`unrun-verify`, the tester's ear: the items, letters, greyed state and
sentences), 611 (`todo`, Mark Done and Pin greyed by module swallow `Ctrl+Shift+K` and
`Ctrl+Shift+P` in silence, `framecmn.cpp:364`; the fix is this plan's open-and-close
pattern). Closed: none.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move; the changelog entry says so.

## Self-Check: PASSED

- `src/presentation/text_undo.rs`, `tests/undo_reaches_the_text.rs`: present.
- `22aea967`, `6956b949`, `0d46dfeb`, `fe87ae00`: in `git log` on `13-01-undo-redo`.
- Carriage returns 0 on every file touched, by `tr -cd '\r' < FILE | wc -c`; no em dash on an
  added line.
