---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 41
subsystem: quick steps
tags: [quick-steps, GAP-11, "#60", managers, accessibility, scan-targets]
status: complete
requires: [13-40]
provides:
  - "The Quick Step Manager on Action, Quick Steps, Manage Quick Steps, over the account being worked in"
  - "The step editor, one question per control, and what_the_editor_holds and what_the_editor_keeps"
  - "save_what_the_quick_step_manager_returned, which never writes a newer version's step and writes the whole order"
  - "Two scan targets: quick-steps and quick-step-editor"
affects: [13-42, 13-51]
tech-stack:
  added: []
  patterns:
    - "A manager row whose content this build cannot read is carried as None and listed, moved and removed, never opened or written"
    - "The editor's OK handler is bound by the caller that knows the other rows, so the builder stays a scan target"
key-files:
  created:
    - tests/the_quick_step_manager_says_what_each_step_does.rs
  modified:
    - src/presentation/wx_managers.rs
    - src/presentation/managers.rs
    - src/presentation/manager_words.rs
    - src/presentation/wx_app.rs
    - src/presentation/scan_target.rs
    - src/presentation/scan_fixtures.rs
    - .github/workflows/accessibility.yml
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The quick-steps scan target opens the manager over one made-up step rather than the fresh profile's empty list, so the scan walks a row"
  - "The editor's decision is its own public function, what_the_editor_keeps, returning StepRefused::TheName or WhatItDoes, so the name box takes focus only for a name refusal"
  - "A phrase of nothing but spaces is no phrase, and a kept phrase is trimmed"
metrics:
  duration: "about 2 hours 40 minutes"
  completed: 2026-09-30
actuals:
  tokens: 31000
  tasks: 4
  commits: 8
---

# Phase 13 Plan 41: The Quick Step Manager Summary

Action, Quick Steps, Manage Quick Steps opens the Quick Step Manager for the account being
worked in: its steps with their keys and what each does, moved with the gesture labels use
and saved with their order on close, and a step editor asking one question per control on
N, R, F, L, M, D and H that refuses a step it cannot keep before OK closes. Nothing a step
does runs yet; that is 13-42 (ledger 742).

## What works now

- Action has a Quick Steps submenu on Q after Saved Searches, holding Manage Quick Steps
  on M with the experimental sentence as its help. The arm runs
  `managers::manage_quick_steps` and reads the tree back, as the Label Manager's does.
- `manage_quick_steps` reads the account's steps, its labels in their order and the
  folders its tree shows by path (`wx_app::folders_in_the_tree`, now `pub(crate)`), and
  opens `show_quick_step_manager_dialog` on the loop every manager uses, with
  `manager_words::QUICK_STEP` ("Added the Quick Step: Sort it").
- The list, "Quick Steps", has columns Name, Key and What it does: `Ctrl+Shift+7` to
  `Ctrl+Shift+9` from `quick_steps::key_for`, blank past the third; a newer version's step
  says "Written by a newer version of Wixen Mail; it can be moved or removed here", and
  Edit on it says why it does not open.
- The close deletes what went, writes every readable step over itself or makes it, never
  writes a newer version's step, then writes the whole order with
  `put_quick_steps_in_order`; any failure is named through `report`.
- The editor: Name (`add_field`), three choices named from their labels, Delete it
  (`add_checkbox`), Phrase to say first (`add_field`), OK and Cancel. A label or folder the
  account no longer has is shown chosen as "Old (not in this account any more)" and kept
  if saved again.
- Both windows are scan targets and in the workflow's list. The target reads, on the
  built windows: the rows and keys off the live list, a move stored, every control's name
  and role over MSAA at its own handle in tab order, the choices' entries, the controls
  read into a step three ways, seven letters none twice, and the real menu bar's Action
  menu.
- The full hook on `73b81e1c` (`all`, 801 s) ran the whole suite, the release build and
  the audit green on the branch.

## Commits and hook times

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `0e79fe2b` | red | the manager's rows, keys and order, 5 named | 234 s |
| `1148c183` | green | rows, keys, moves and save; two records | 348 s |
| `58d874ec` | red | the editor, 15 named and the count check | 200 s |
| `b7d8d97b` | green | the editor, the manager's window, `manage_quick_steps`; one record | 450 s |
| `e193d17b` | red | the menu and the arm, 3 named and the count check | 98 s |
| `1766029b` | green | Action, Quick Steps, Manage Quick Steps; one record | 344 s |
| `73b81e1c` | green | the two scan targets and the workflow line | `all`, 801 s |
| documents | docs | the pages, the changelog, the ledger, the marks, this summary | see the merge |

## Counts

- `tests/the_quick_step_manager_says_what_each_step_does.rs`: 27 tests, all pass (the plan
  asked at least 10).
- `presentation::wx_managers::` 44, `presentation::managers::` 137 (the plan's figure of
  2026-09-24 was 137 too; a grep for `#[test]` answers 140), `presentation::manager_words::`
  8, `presentation::scan_target::` 11, `presentation::scan_fixtures::` 13 (the plan's 11
  was the count on 2026-09-24), `presentation::wx_app::` 199: none added, all pass.
- `cargo test --test wired` 77 pass, so Q is Action's once and M the submenu's once.

## Guard records

Four added, the arrived-since count at the top of `guards/guards.toml` 519 to 523. Each
measured through `scripts/guards.sh --remeasure`:

| Record | Break | Red |
|--------|-------|-----|
| the Quick Step Manager's Key column gives a key to the first three steps only | a fourth step given `Ctrl+Shift+0` | the Key reading, the rows, the move |
| the Quick Step Manager's close writes the order its rows came back in | `put_quick_steps_in_order` dropped | the stored-order reading |
| the Quick Step editor reads Mark unread as unread | `Some(2) => Some(true)` in `yes_no_or_leave` | the together reading |
| Action holds the Quick Steps submenu | the `append_submenu` call taken out | the menu reading |

The records naming the target were re-measured with each commit that added target tests.

## Deviations from Plan

**1. [Order] `show_quick_step_manager_dialog` and `manage_quick_steps` landed with task 2's
green, not task 1's.** The manager's window opens the editor, which task 2 builds; landing
it in task 1 would have meant an Add and Edit that open nothing. Task 1's green held what
its target reads: the rows, the keys, the move and the save.

**2. [Rule 2] The target reads more than the plan listed:** the three answers of the
decision (a taken name, a delete beside a move, a tidied name kept), the editor's two
titles, and that a newer version's step keeps its action through the close.

**3. [Design] The `quick-steps` scan target opens the manager over one made-up step**,
`scan_fixtures::quick_step()`, rather than through `manage_quick_steps` on the fresh
profile's empty list, so the scan walks a row as well as the buttons; nothing reaches the
store. `scan_fixtures::quick_step_labels()` is new beside it, and the two folders are
`folders_a_search_can_look_in()`'s. No fixture test was added: both windows open whatever
the fixture holds, so there is no property that decides whether they open.

**4. [Rule 2] `manager_words::QUICK_STEP` joined `EVERY_KIND`**, so the manager's
sentences are held to the same checks as the other seven kinds. The count stays 8.

**5. [Brief] The mnemonics Q and M landed in `1766029b` and the shortcuts page in the
documents commit**, as the plan's task 4 has it, where the brief asks for the page in the
same commit as a key. No chord changed.

**5a. [Rule 1] The shortcuts page does not name `Ctrl+Shift+7` to `Ctrl+Shift+9`.**
`wired`'s `test_the_shortcuts_document_names_no_key_the_code_has_never_heard_of` refuses a
key the page names and nothing binds, and nothing binds these until 13-42. The page says
the Key column shows them and that the page lists them when running arrives; the guide
and the changelog name them, as keys the Key column shows.

**6. [Tooling] Two extra `--remeasure` calls.** On task 2 the first call failed every
record with `OSError(22, 'Invalid argument')` before building, left no break in the tree,
and passed when run again. On task 3 the first call named a record not yet in the file.

**7. [Line numbers] Anchors had moved since 2026-09-24:** `folders_in_the_tree` at
`wx_app.rs:13529`, the Action menu's submenus at `:7550` onwards, the manager arms at
`:5729`, `ScanTarget::ALL` at `:257` with 46 targets. Each was found by its text.

### Found and left

- Ledger 744: a step given the name another step had when the manager opened can fail to
  save, because each step is written in its own transaction in the list's order and the
  store keeps one name per account; a swap of two names always leaves one unsaved. The
  close names the step that failed, and the guide and changelog say how to rename across
  two openings.

## TDD Gate Compliance

`0e79fe2b` (test) before `1148c183` (feat); `58d874ec` (test) before `b7d8d97b` (feat);
`e193d17b` (test) before `1766029b` (feat). `73b81e1c` carries its own checks in the same
commit (`test_the_workflow_asks_for_every_target` and the list in
`test_every_window_a_fresh_profile_can_reach_has_a_name`), so the workflow check is never
red. Unnamed in the reds, passing against the stubs: the order companion, the
newer-version case and the empty-rows Key check in the first; the menu companion in the
third. Every expected value was reasoned before its green: `key_for` gives 7, 8 and 9 for
places 1 to 3; `what_it_does_in_words` joins "mark read" and "move to INBOX/Archive" as
"Mark read, move to INBOX/Archive"; the move says "Archive and read, 2 of 4." from
`reordering::moved`; `name_from_label` turns "&Flag:" into "Flag," and "&Delete it" into
"Delete it"; the refusals come from `name_for` and `what_stops_a_step_being_saved`
themselves.

## Ledger

Opened, both halves: 743 (`unrun-verify`, the tester's ear on the submenu, the manager's
rows and Key column, the editor's seven questions in order and a refusal) and 744 (`todo`,
the rename collision above). None closed; 742 stays for 13-42. Counts 744 in all, 663
open, 81 fixed, 0 waived.

## Known Stubs

None added. Running a step from the submenu or by its key is 13-42's, under ledger 742,
and the changelog, the guide and the shortcuts page say it is not built yet.

## Threat Flags

None beyond the register. T-13-41-01 by `what_the_editor_holds` read in the target and
its record; T-13-41-02 by `show_quick_step_edit` refusing a `None` row and the save never
writing one, read by the target; T-13-41-03 by the folders coming from
`folders_in_the_tree` for the step's own account; T-13-41-04 by the MSAA readings, the
letter reading and the two scan targets. No crate added (T-13-41-SC).

## Self-Check: PASSED

`tests/the_quick_step_manager_says_what_each_step_does.rs` exists; `0e79fe2b`, `1148c183`,
`58d874ec`, `b7d8d97b`, `e193d17b`, `1766029b` and `73b81e1c` are on the branch.
`pub fn build_quick_step_manager` once, `impl ManagedRow for QuickStepEntry` once,
`pub fn build_quick_step_edit_dialog` once, `pub fn what_the_editor_holds` once and
`"Mark as &read or unread:"` once in `wx_managers.rs`; `pub fn manage_quick_steps` once and
`put_quick_steps_in_order(` once in `managers.rs`; `"&Manage Quick Steps..."` once and
`"&Quick Steps"` once in `wx_app.rs`; `'quick-steps', 'quick-step-editor'` once in the
workflow and both anchored lines once each; carriage returns 0 in every file touched.
