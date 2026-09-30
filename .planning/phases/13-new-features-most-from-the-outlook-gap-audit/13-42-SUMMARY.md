---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 42
subsystem: quick steps
tags: [quick-steps, GAP-11, "#60", runner, keys, "ledger 688"]
status: complete
requires: [13-40, 13-41, 13-24.1]
provides:
  - "quick_steps::what_the_account_lacks, Missing, what_is_gone, not_this_accounts and written_by_a_newer_version"
  - "UIUpdate::QuickStepsLoaded, WxUIState::quick_steps, rebuild_the_quick_steps_menu, put_the_quick_steps_on_the_menu, a_quick_step_key_the_menu_does_not_answer and answer_the_quick_step_keys_the_menu_cannot"
  - "run_the_quick_step_at: a step over the selection through run_these_actions_over, one sentence"
  - "moves_waiting::MarksFirst, the read_first and starred_first columns, send_these_marks_before_the_move and the_marks_before_the_move; TheWork::the_marks_that_go_with_the_move; ReplaysAMove::mark_it"
affects: [13-43, 13-44, 13-51]
tech-stack:
  added: []
  patterns:
    - "A run's move carries the marks of each message it moves on the waiting row, and the replay sends them before the move on one session"
    - "A submenu built from data is written in one rebuild function the builder calls with nothing, and is rebuilt only when what it says differs"
key-files:
  created:
    - tests/a_quick_step_runs_over_the_selection.rs
  modified:
    - src/application/quick_steps.rs
    - src/application/acting_on_a_set.rs
    - src/application/moves_waiting.rs
    - src/data/message_cache/moves_waiting.rs
    - src/data/message_cache/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "A step's folder and label are checked before the runner by quick_steps::what_the_account_lacks, found the way a rule finds them, so the refusal names the step"
  - "The item arm is `_ if quick_step_position_of(id).is_some()`, the saved searches' shape, because clippy refuses the plan's `id >= && id <` guard as a manual range"
  - "Ledger 688: marks ride on the waiting move's row as two added columns, handed over by the do-halves through WxUIState::the_next_move_carries, so no do-half's or move's signature changed and no guard anchor on them moved"
  - "A mark the server refuses before a move is logged and the move still goes; any other failure to send the marks is the move's answer, so the two wait or are put back together"
metrics:
  duration: "about 3 hours 30 minutes"
  completed: 2026-09-30
actuals:
  tokens: 38000
  tasks: 5
  commits: 10
---

# Phase 13 Plan 42: Quick Steps run over the selection Summary

Action, Quick Steps lists the account's steps with `Ctrl+Shift+7` to `Ctrl+Shift+9` on the
first three, and choosing one or pressing its key runs it over the selected messages
through 13-24.1's runner, refusing in words what it cannot do whole, then saying one
sentence with one Confirmed. A step that marks and moves now sends the marks to the server
before the move, on one session (ledger 688).

## What works now

- **The submenu.** `folder_tree_updates` sends `UIUpdate::QuickStepsLoaded` with the active
  account's steps beside `LabelsLoaded`; its arm keeps them in `WxUIState::quick_steps` and
  calls `put_the_quick_steps_on_the_menu`, which rebuilds Action, Quick Steps only when the
  step items differ. The items are `quick_steps::what_the_menu_says` lines under a block of
  fifty ids, the first three with `Ctrl+Shift+7` to `Ctrl+Shift+9`, then a separator and
  Manage Quick Steps. Read on the real menu bar: Manage alone as the window starts; four
  steps, the newer version's at its place with its key; the order after a move written to
  the store; the same steps put on again leaving a marked first item as it was.
- **The keys.** Measured in the target: with Ctrl and Shift held in the thread's keyboard
  state, `WM_KEYDOWN` for 9 posted to a built list reaches its key event as Ctrl+Shift+9
  when the menu has no third item, and the handler answers place 3; Ctrl+Shift+7 is taken
  by the menu's accelerator and runs the first step's item. A key past the last step says
  `quick_steps::nothing_there` on the status line, spoken.
- **The run.** `run_the_quick_step_at` reads the step at its place (a newer version's is
  refused by name), reads the selection with `quick_steps::reach` over the D-07 setting,
  refuses nothing chosen and more than 5,000, refuses the whole run when any chosen message
  is another account's (`owner_of`), refuses a folder or label the account lost
  (`what_the_account_lacks`), calls `run_these_actions_over` once, and says
  `what_a_step_did` announced at Normal, shown, with one Confirmed. It writes no mail of
  its own; every write is the runner's, through the gated paths.
- **Undo, read in source.** The runner remembers no action; each do-half remembers its own,
  so Undo after a step takes back its last write (the move of a step that marks and moves).
  The guide and changelog say so, and ledger 747 carries it for whoever builds Undo next.
- **Ledger 688.** A run's move carries the marks of each message it moves on the waiting
  row (`read_first`, `starred_first`), and the replay sends them before the move on one
  session. Held by the scripted-server case reading `UID STORE 42 +FLAGS (\Seen)` before
  `UID MOVE 42`.
- The branch goes to a pull request after this documents commit, since what is spoken and
  shown changed; CI, NVDA and Accessibility are read there before the local merge.

## Commits and hook times

| Commit | Kind | What | Hook |
|---|---|---|---|
| `a5401968` | red | the three refusals' sentences and the lacks rule, 5 named and the count check | red, 149 s (a first try stopped at rustfmt after 12 s) |
| `7ab77e22` | green | the sentences; one record re-measured | affected, 253 s (a first try stopped at rustfmt after 7 s) |
| `e880196c` | red | the submenu and key readings, 7 named | red, 239 s |
| `d4556723` | green | the submenu, the ids, the keys, the loaded arm; two records | affected, 358 s (a first try stopped at clippy after 25 s) |
| `8d11b015` | red | the run readings, 2 named and the count check | red, 102 s |
| `d1658c70` | green | `run_the_quick_step_at`; two records, two re-measured | affected, 354 s |
| `934eeed4` | red | ledger 688: the pure rule, the store rule and the scripted-server order, 3 named and the count check | red, 148 s |
| `480cc932` | green | ledger 688's fix; two records, eighteen re-measured | affected, 348 s (a first try stopped after 356 s on the planning check, see deviation 7) |
| documents | docs | the pages, the changelog, the ledger, the marks, this summary | see the merge |

## Counts

- `application::quick_steps::` 22 before, 27 after. `application::acting_on_a_set::` 23 to
  24. `application::moves_waiting::` 49 to 51. `data::message_cache::moves_waiting::` 15 to
  16.
- `tests/a_quick_step_runs_over_the_selection.rs` 33 (the plan asked at least 14).
- Unchanged and passing: `presentation::wx_app::` 199, `presentation::ui_types::` 79,
  `wired` 77, `every_command_acts_on_the_selection` 16 and 1 ignored,
  `several_actions_reach_the_server_in_order` 18, `each_set_command_has_a_quiet_do_half`
  10, `a_move_completes_here_first` 15, `a_move_across_accounts_completes_here_first` 11,
  `mark_as_read_says_which_way_it_will_go` 14, `a_block_moves_the_mail_already_here` 15,
  `undoing_a_mark_names_the_message` 21, `nothing_sends_a_flag_change_unasked` 3,
  `the_quick_step_manager_says_what_each_step_does` 27.

## Guard records

Six added, two a commit, the arrived-since count at the top of `guards/guards.toml` 523 to
529, each measured through `scripts/guards.sh --remeasure`, each reddening exactly the test
named:

| Record | Break | Red |
|---|---|---|
| the Quick Steps that load are put on the menu | the put call dropped from the arm | the loaded-arm reading |
| a Quick Step key past the last step is answered from the list | the handler asks for Alt instead of Shift | the key reading |
| a Quick Step meets the Select All bound before it runs | `too_many` skipped | the run's order reading |
| a Quick Step refuses messages of another account | the `owner_of` count replaced by nought | the run's order reading |
| a waiting move sends the marks it carries before the move | the marks' send replaced by `Ok(())` | the scripted-server order case |
| a move made here keeps the marks handed to it | the keep call dropped | the keep-before-push reading |

Re-measured because a file they name gained tests:
"a Quick Step that deletes and moves is refused before it is saved" (task 1, 122 s), and in
one background call on the 688 green, the ten records naming
`src/application/moves_waiting.rs`, the two naming `acting_on_a_set.rs` and the six
naming the target. One came back short: "a run over a set drops a mark a message already
has" now also reddens `test_the_marks_of_a_message_that_moves_go_with_its_move`, which was
added to its red list by hand and measured again.

## Deviations from Plan

**1. [Ledger 688, from the brief] Its own red and green pair.** `934eeed4` red and
`480cc932` green, as the brief asked. The failing case builds a Quick Step that marks read
and files into Archive, decides it with `what_each_message_needs` and `the_work`, makes the
move here with the marks the move carries, replays it against the scripted IMAP server and
reads the order: `UID STORE 42 +FLAGS (\Seen)` before `UID MOVE 42`. It was red because
nothing carried the marks, so no STORE was sent. Design: two added columns on
`moves_waiting` rather than a field on `AWaitingMove` (29 construction sites) and no new
table; the do-halves hand the mark over through `WxUIState::the_next_move_carries`, which
the runner names per account and empties before it returns, so no do-half's or move's
signature changed and no guard anchor on their calls moved. A refused mark lets the move
go; an unreached one makes the move wait with it. Left: a label beside a move in a run and
a mark by hand before a move by hand still travel on their own worker (ledger 748).

**2. [Rule 2] `what_the_account_lacks` in `application::quick_steps`.** The plan wanted the
folder and label resolved before the runner "the way 13-24.1's runner resolves them". That
decision is pure, so it is a function beside the sentences with its own case, found by
`the_folder_a_rule_names` and `the_label_a_rule_names` like the runner's.

**3. [Rule 3] One green for task 2, not four.** The hook runs every target that reaches the
changed code; a green that built only the update and its arm would have left the menu and
key readings red in the same target, and the hook refuses a green with a failing test. The
plan's split could not be committed.

**4. [Rule 3] The item arm's guard.** Clippy refuses `id >= FIRST && id < FIRST + N` as a
manual range; the arm is `_ if quick_step_position_of(id).is_some()`, the saved searches'
shape. Nothing raises `ID_QUICK_STEP_FIRST` by name, so `wired`'s handled-id check is
unaffected, and `wired` passes at 77.

**5. [Measurement] The key probe binds before the handler.** The red measured with the probe
bound after a stub; once the handler consumed Ctrl+Shift+9 the probe saw nothing, because
handlers on one control run in the order they were bound. The probe was moved before the
handler in the green and the header says why.

**6. [Interim stub] `run_the_quick_step_at` answered nothing between `d4556723` and
`d1658c70`,** as the plan's task 3 red requires ("present and answering nothing"). It
never reached `main`.

**7. [Tooling] The planning check read the working tree.** The first try of the 688 green
failed `the_planning_files_agree_with_themselves` because this summary's draft was already
on disk and `STATE.md`'s `completed_plans` still said 222. The count was set to 223 in the
working tree and the commit went through; the count lands in the documents commit.

**8. [Anchors] The plan's line numbers had moved**, as it said they would: the labels'
functions at `wx_app.rs:10993` to `:11061`, `folder_tree_updates` at `:13425`, the
`LabelsLoaded` arm at `:21542`. Each was found by its text.

### Found and left

- Ledger 748: a label a run puts on a message it also moves, and a mark by hand before a
  move by hand, still go on their own worker. Recommendation for Pratik: the waiting move
  carries the keywords too, and a hand mark on a row with a waiting move joins its marks.
- Ledger 687 is narrowed, not closed: the order at the server is now proved for a run's
  move against a scripted server, and still not for the two workers a hand mark and a hand
  move use.

## TDD Gate Compliance

Four red commits, each before its green: `a5401968` then `7ab77e22`; `e880196c` then
`d4556723`; `8d11b015` then `d1658c70`; `934eeed4` then `480cc932`. Unnamed in the reds and
passing against the stubs: the menu-as-it-starts reading, the same-steps-again reading and
the companions (task 2); the writes-nothing-of-its-own reading and the Undo reading (task
3), which read facts about code that already existed; the refused-mark case (688). The
window's half of 688 (the runner naming the rows, the do-halves handing over, the move
keeping the marks before the push) was read by three readings written with the green, each
taken red by its companion and the keep-before-push one by its guard record. Every expected
value was reasoned before its green: the sentences' words, "1 of the chosen messages is",
Ctrl+Shift+9 for place 3, "false true" for places 2 and 3 with two steps, the STORE naming
42 and `\Seen` from `ImapSession::set_flag`.

## Ledger

Closed, both halves: 688 (the marks carried by the move) and 742 (Quick Steps run).
Opened, both halves: 745 (`unrun-verify`, phase 14, a step reaching a real IMAP server in
order, on Gmail and on a server with folders), 746 (`unrun-verify`, the tester's ear), 747
(`todo`, a step is not one undo), 748 (`todo`, a label beside a move and a hand mark before
a hand move still race). Counts 748 in all, 665 open, 83 fixed, 0 waived.

## Threat Flags

None beyond the register. T-13-42-01: the runner's gate, and `test_a_run_writes_no_mail_of_its_own`.
T-13-42-02: `owner_of` per message, its reading and record. T-13-42-03:
`what_the_account_lacks` before the runner, its case. T-13-42-04: one sentence and one
Confirmed, its reading; the bound, its reading and record. T-13-42-05: one list in
`WxUIState::quick_steps` for the menu, the key and the run. T-13-42-SC: no crate added.
The 688 fix adds two columns to `moves_waiting` (additive, `ensure_column_exists`) and a
new trait method at the replay boundary; nothing new crosses a trust boundary.

## Known Stubs

None. Ledger 742's stub is closed: a step is made, shown and run.

## Self-Check: PASSED

`tests/a_quick_step_runs_over_the_selection.rs` exists; `a5401968`, `7ab77e22`,
`e880196c`, `d4556723`, `8d11b015`, `d1658c70`, `934eeed4` and `480cc932` are on the
branch. `grep -c 'UIUpdate::QuickStepsLoaded' src/presentation/wx_app.rs` 2;
`fn rebuild_the_quick_steps_menu` 1; `ID_QUICK_STEP_FIRST[QUICK_STEPS_ON_THE_MENU]` 1;
`    if shown != wanted {` 1; `quick_steps::key_for` in `tests/wired.rs` 1;
`fn run_the_quick_step_at` 1 and `run_the_quick_step_at(` 2; `what_a_step_did(` 1;
`pub fn what_is_gone` and `pub fn not_this_accounts` 1 each; `Ctrl+Shift+7` on the
shortcuts page 3; `Quick Step` on the alpha page 2; GAP-11's ticked line 1. Carriage
returns 0 in every file touched, counted with `tr`; no em dash and none of the six words in
any added line.
