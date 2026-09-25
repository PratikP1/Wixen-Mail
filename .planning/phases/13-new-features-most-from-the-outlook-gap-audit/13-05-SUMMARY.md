---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 5
subsystem: application::text_history, presentation::text_history_keys, presentation::text_undo
status: complete
tags: [undo, redo, edit-menu, keyboard, accessibility, GAP-02]
requires: [13-01]
provides:
  - application::text_history::History, a box's steps back and forward, bounded at 100
  - presentation::text_history_keys, keep_a_history, set_anew, undo_in, redo_in, can_undo_in, can_redo_in
  - text_undo and the Edit menu's greying asking the history, the one-step EM_UNDO path gone
affects: [13-06, 13-07, 13-08, 13-09]
tech-stack:
  added: []
  patterns:
    - "a registry of per-control state keyed by window handle on the interface thread, dropped on the control's destroy event"
    - "a key-down taken after acting so wxWidgets drops the character and the native control does not act as well"
key-files:
  created:
    - src/application/text_history.rs
    - src/presentation/text_history_keys.rs
    - tests/several_steps_come_back.rs
  modified:
    - src/application/mod.rs
    - src/application/editing.rs
    - src/presentation/mod.rs
    - src/presentation/text_undo.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_notes_module.rs
    - src/presentation/wx_contacts_module.rs
    - tests/undo_reaches_the_text.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "No flag in the registry: the history already holds the value it hands back, so the change a restore raises is empty and records nothing; set_anew starts the history before it writes for the same reason. Measured by the break that stops the history taking the value."
  - "History::record takes the value after and the caret, not the value before as well: the history keeps what the box holds, and a second copy of it could only disagree."
  - "Undo brings the words a step removed back selected, and puts the caret where a typed step began; Redo puts the caret after what it puts back."
  - "Where a change is ambiguous, as an a typed into aa, its end is put at the caret."
metrics:
  duration: about 2 hours 30 minutes
  completed: 2026-09-25
actuals:
  tokens: 13600
  tasks: 3
  commits: 5
---

# Phase 13 Plan 05: Several steps of undo in the main window's boxes Summary

The note title, the note body and the contacts search each keep up to 100 steps, and Undo
takes them back one at a time: a word with the space after it, a paste, a cut, or a run of
deleting one way. Redo puts them back until something new is typed. The Edit menu, `Ctrl+Z`,
`Ctrl+Y` and the menu's greying all ask the history, and 13-01's one step through `EM_UNDO`
is gone. A note chosen in the list starts its boxes afresh, so Undo never reaches into the
last note. A restore raises the box's change, so the contacts search runs again on the words
it brings back. On a real box, `tests/several_steps_come_back.rs` types three words a
character at a time and walks them back and forward.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `da429da9`.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `f03c1fcd` | red | `application::text_history` stubbed to answer nothing, and its ten cases; 10 named | 139 s |
| `43520136` | green | The history: the change between two values anchored at the caret, the grouping rule, undo and redo, the bound; two records | 195 s |
| `ceb0a869` | red | `tests/several_steps_come_back.rs`, 13-01's target rewritten, `text_undo` and the Edit menu on the final path over a stubbed `text_history_keys`, 13-01's two records retired; 11 named | 258 s (refused twice, below) |
| `dab72acb` | green | `text_history_keys`, the three boxes through `keep_a_history`, the note selection through `set_anew`; three records new, one re-measured | 228 s |
| this commit | docs | the shortcuts page, the guide, the changelog, the ledger, this summary, the four marks | |

**Test counts, taken again.** `cargo test --lib application::text_history::` 10 (new).
`cargo test --test several_steps_come_back` 11 (new). `cargo test --test undo_reaches_the_text`
15 before and 15 after. `cargo test --lib application::editing::` 13 before and after.
`src/presentation/wx_app.rs` 199 tests before and after, read by the count check on every
commit.

**Acceptance readings.** `grep -c 'EM_UNDO\|EM_CANUNDO' src/presentation/text_undo.rs` is 0.
`grep -c 'keep_a_history'` reports 2 in `wx_notes_module.rs` and 1 in `wx_contacts_module.rs`.
`grep -c 'set_value' src/presentation/wx_app.rs` is 8 at `ceb0a869` and 2 at `dab72acb`: the six
note-selection writes, and nothing else.

## What 13-01's target reads now

Rewritten in place, no test added. Every box is built through `keep_a_history`. Readings
unchanged where the one step and the history agree: Undo after typing, Redo straight after,
Redo refused after typing, a fresh box, the closed menu, and the four raw `EM_UNDO`
measurements, which still read the native box and still say why Cut writes nothing over the
selection. Two changed:

- **The open menu after an undo** read "Undo offered, Redo offered", because the box's own
  step offered Undo again as a second way to redo. With the history, the one step typed is
  gone and there is nothing further back, so it reads "Undo greyed, Redo offered".
- **What the program writes** used `set_value`, which the history keeps as a step. It uses
  `set_anew`, and `can_undo_in` reads false.

## Guard records

| Record | Red | Run |
|--------|-----|-----|
| a space ends the step of typing it is typed in (new, `text_history.rs`) | the three cases that type words | task 1, 89 s |
| typing after an undo leaves nothing to redo (new, `text_history.rs`) | its one case | task 1, 93 s |
| Ctrl+Z in a box is taken after the history acts, so the box does not undo again (new, `text_history_keys.rs`, suite the new target) | the Ctrl+Z reading | task 2, 27 s |
| a restore is not kept as a step, because the history already holds what it handed back (new, `text_history.rs`, suite the new target) | four readings | task 2, 19 s |
| a chosen note is written as the program's own value, not as a step (new, `wx_app.rs`, suite the new target) | the note-selection reading | task 2 |
| Undo and Redo are offered again whenever a menu closes (13-01's, re-measured against its rewritten target) | as recorded | task 2 |

Each break was taken by hand first, then all were measured with one `scripts/guards.sh
--remeasure` call per task, every record reddening exactly what it names. **Retired**, in
`ceb0a869`, with a comment where they stood: "Undo asks the box whether it has anything to take
back before it acts" and "Redo is refused once something is typed after the Undo", both
anchored on the `EM_UNDO` path and the `LastUndo` record, which went. What they guarded is held
by the history now. The arrived-since line at the head of `guards/guards.toml` went from 308 to
310 in task 1, back to 308 with the two retired, and to 311 with task 2's three.

## Deviations from Plan

### Auto-fixed Issues

**1. [Shape] No flag in the registry.** The plan asked for "a flag in the registry" to stop the
history recording its own restore, and a record whose break lets it. Written that way and then
measured, the flag guarded nothing: `History::undo` already takes the value it hands back, so
when the box raises its change the change is empty. The flag went, `set_anew` starts the history
before it writes, and the record's break is on the line that makes the history take the value
(`text_history.rs`), with the new target as its suite. It reddens four readings.

**2. [Shape] `History::record(after, caret)`, not `record(before, after, caret)`.** The history
holds what the box holds, so the value before is its own; passed in, it could only disagree.

**3. [Rule 1] Where a change sits is anchored at the caret.** A common prefix and suffix alone
put an "a" typed at the start of "aa" at the end, and the step's position with it. The change's
end is put at the caret, which is where a typed or pasted run ends and where a deletion leaves
it.

**4. [Test] Characters sent, not posted.** The plan said key presses posted to the box, each
effect waited for. `SendMessageW` with `WM_CHAR` returns once the box has taken the character
and raised its change, which is the waiting. Control is set in this thread's keyboard state for
Ctrl+Z, never through `SendInput`.

**5. [Test] The Ctrl+Z reading counts characters.** The box's own undo after a restore changes
nothing, because a restore writes the whole value and that empties the box's undo buffer, so
reading the value alone could not tell a key passed on from a key taken. A counter on the box's
character event reads whether the character reached the box, and the companion hands the check
a character that did.

**6. [Test] The note selection is read from `wx_app.rs`.** Building the main window to choose a
note was not needed to ask how the handler writes; the reading counts `set_value` and
`set_anew` in the handler, 0 and 6, with a companion planting one `set_value`.

**7. The red of task 2 was refused twice.** First by rustfmt at 10 s, a line the formatter
wanted joined; then at 288 s because the trailers named integration tests as
`target::test`, where the red gate reads bare names. Both fixed before the retry.

**8. `application/editing.rs`'s doc comment** said Undo and Redo take back the box's own one
step; it names the history now. A file the plan did not list.

### Found and left

- **The key-down path is reached only by the target until 13-06.** In the main window the Edit
  menu's accelerator takes `Ctrl+Z` and `Ctrl+Y` before the box sees them, so Undo there always
  goes through the menu, which speaks. A box in a dialog has no menu, and there the key-down
  answers without a sentence, as Windows' own step does today. Whether a dialog's Undo speaks
  is 13-06's to decide.
- `Alt+Backspace` is Windows' own undo in an edit box and is not taken. Pressed, the box undoes
  its own one step and the history keeps that as a change, so the two stay in step.

## Threat Flags

None beyond the register. T-13-15: 100 steps per box, held by
`test_the_hundred_and_first_step_drops_the_first`. T-13-16: the key-down is taken after acting,
held by the Ctrl+Z reading and its record. T-13-17: a box's history goes on its destroy event,
and nothing in either module writes to disk or to the log. T-13-SC: nothing added to the
manifest; `Cargo.lock` unchanged.

## Known Stubs

None. Every box the Edit menu reaches keeps a history from where it is built, a non-test path
at startup.

## Ledger

Opened: 619 (`unrun-verify`, each step heard coming back in the three boxes, the caret, the
search re-running). Closed: none. Both halves.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does not
move.

## Self-Check: PASSED

- `src/application/text_history.rs`, `src/presentation/text_history_keys.rs`,
  `tests/several_steps_come_back.rs`: present.
- `f03c1fcd`, `43520136`, `ceb0a869`, `dab72acb`: in `git log` on `13-05-several-steps-of-undo`.
- Carriage returns 0 on every file touched, by `tr -cd '\r' < FILE | wc -c`; no em dash on an
  added line.
