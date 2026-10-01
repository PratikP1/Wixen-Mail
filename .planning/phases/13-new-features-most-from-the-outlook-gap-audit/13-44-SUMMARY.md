---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 44
subsystem: running a rule over a folder by hand
tags: [rules, filter-manager, this-folder, runner, GAP-12, "#61"]
status: complete
requires: [13-24.1, 13-43]
provides:
  - "Action, This Folder, Run a Rule on This Folder (ID_RUN_A_RULE_HERE, L) and run_a_rule_on_this_folder"
  - "wx_app::count_what_a_rule_would_change(app, account, folder, rule), on a worker, sending UIUpdate::ARuleWasCounted(Box<ARuleCounted>)"
  - "ui_types::ARuleCounted { account_id, rule_name, found: WhatTheCountFound::{NothingToChange(String), Ask { question, set, outcome, reaches_the_server }} }"
  - "wx_app::ask_then_run_the_counted_rule: the gate, a native question, run_these_actions_over once, what_the_rule_did"
  - "wx_managers: ManagedRow::runs_on_a_folder, put_the_buttons_on, ManagerButtons, run_the_manager_loop answering ManagerClosed { changed, run_on }, build_filter_manager, FilterManagerAction::RunOnAFolder { rules, which }, the Filter Manager's Run on a Folder (R)"
  - "managers::RunARuleNow { account, rule_id } answered by manage_filters after it saves; wx_app::run_a_rule_on_a_chosen_folder"
affects: [13-51]
tech-stack:
  added: []
  patterns: ["a manager window's buttons are put on by its builder, so a test reads a built window's buttons over MSAA without running a loop only a person can end"]
key-files:
  created:
    - tests/a_rule_runs_over_a_folder_when_asked.rs
  modified:
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_managers.rs
    - src/presentation/managers.rs
    - tests/the_list_at_two_hundred_thousand_rows.rs
    - tests/the_nvda_cases_wait_for_words_the_program_says.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - docs/development/measurements.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The count arrives as its own update and the question is asked in its arm, on the window thread; a count that arrives after another account was opened is refused, because the runner reads a message's account off the one open."
  - "The gate is asked in the arm only when the run reaches the server, the runner's own rule, so a rule that only says a phrase first is not refused on an account whose changes are off."
  - "The Filter Manager gives every row an id before it saves on Run on a Folder, so a rule added in the same visit is run by the id it is saved under."
  - "On the run path a successful save says nothing, since the folder list that opens next is the answer; a failed save is said and nothing runs."
  - "The Label Manager's builder puts its buttons on too, so the reading that it offers no Run on a Folder reads a real window."
metrics:
  duration: "about 6 hours 40 minutes to the documents commit, of which about 4 hours waiting on a locked session"
  completed: 2026-10-01
actuals:
  tokens: 21000
  tasks: 4
  commits: 9
---

# Phase 13 Plan 44: Run a rule over a folder, from This Folder and the Filter Manager

A rule now runs over a folder when somebody asks. Action, This Folder, Run a Rule on This
Folder (`L`) offers the account's rules, a switched-off one marked "(switched off)", and the
Filter Manager's Run on a Folder (`Alt+R`) saves the rules and offers the account's folders.
Either way the rule is counted on a worker through 13-43's count, and the window says what it
found: nothing to change is said and nothing asked; otherwise the account's gate is met, then
a native question with Enter answering No before a rule that deletes, and Yes hands the counted
set to 13-24.1's runner once and one sentence says what it did. The count over 200,000 rows
took 430.02 ms in release. GAP-12 is ticked and #61 closes from the merge.

## What works, and how it is known

- **This Folder's door.** `tests/a_rule_runs_over_a_folder_when_asked.rs`, read over
  `what_ships` of `wx_app.rs`: the item's arm opens `run_a_rule_on_this_folder` once, which
  calls `choose_from_list` before `count_what_a_rule_would_change`; the real menu bar's This
  Folder holds "Run a Ru&le on This Folder..." with its help ending in
  `RUNNING_A_RULE_NOW_IS_EXPERIMENTAL`.
- **The count.** `count_what_a_rule_would_change` calls `spawn_blocking` before
  `what_a_rule_would_change` and sends `UIUpdate::ARuleWasCounted` after it.
- **The question and the run.** `ask_then_run_the_counted_rule` meets `permitted` before
  `MessageDialog::builder`, reads `which_of_the_two` before the runner, calls
  `run_these_actions_over` once, and after it announces once, signals once and puts nothing
  on the status line of its own. None of the three doors and the count writes mail of its
  own.
- **The Filter Manager's door.** A built Filter Manager holds `&Run on a Folder...`, named
  "Run on a Folder..." over MSAA at its own handle; a built Label Manager holds Delete and no
  such button. `manage_filters` calls `save_what_the_filter_manager_returned` before it
  answers a `RunARuleNow`; Tools, Message Filters' arm hands that to
  `run_a_rule_on_a_chosen_folder`, which calls `choose_from_list` before the count.
- **Companions.** Seven, each planting one fault into a snippet and seen refused: the runner
  before the question, the gate after it, the count skipped, the count on the window thread,
  a write round the runner, an answer before the save, a count before the folder is chosen.
- **The target:** 22 passed (the plan asked at least 14).
- **Nothing else moved.** `presentation::wx_app::` 199 before and after,
  `presentation::ui_types::` 79, `presentation::wx_managers::` 44, `presentation::managers::`
  137, `wired` 77, `manager_dialog_labels` 1, `the_label_menu_says_the_labels_an_account_has`
  16, `the_nvda_cases_wait_for_words_the_program_says` 8,
  `the_list_at_two_hundred_thousand_rows` 8 passed and 2 ignored before and after.
- **The scale row.** `cargo test --release --test the_list_at_two_hundred_thousand_rows --
  --ignored --nocapture --test-threads=1` at `0fca6f7d`: 430.02 ms, takes 430.02, 419.26 and
  435.26 ms, 40,000 matched and 13,334 would change, written on
  `docs/development/measurements.md` with its conditions.
- **Reachability.** Both doors are reached from the menu bar and from the Filter Manager's
  button in the running program; the Filter Manager is a scan target, so the Accessibility
  workflow walks the new button. The branch goes to a pull request after this commit, since
  what is spoken and shown changed.

## Commits and hook times

| Commit | Kind | What | Hook |
|---|---|---|---|
| `ba018fcf` | red | This Folder's readings, 9 named | red, 109 s (a first try stopped at rustfmt after 15 s) |
| `8ad48b5f` | green | the item, the count, the question and the run; two records | affected, 377 s |
| `54439c62` | refactor | the managers' builders put their buttons on | affected, 134 s (a first try failed 136 s on the NVDA case's tie for Delete) |
| `e06cc451` | red | the Filter Manager's readings, 5 named and the count check | red, 114 s |
| `24a93908` | green | Run on a Folder, saving first, and the second door; two records, two re-measured | affected, 353 s (two tries refused, see deviation 6) |
| `2c11a1f4` | red | the scale harness owes the row, 1 named | red, 94 s |
| `0fca6f7d` | green | the timed step | affected, 78 s |
| documents | docs | the pages, the ledger, the marks, this summary | see the merge |

## Guard records

Four added, the arrived-since count at the top of `guards/guards.toml` 531 to 535, each
reddening exactly the one test named:

| Record | Break | Red |
|---|---|---|
| a rule run by hand is asked about before the runner runs | the runner called before the question | the question-before-run reading |
| a rule run by hand meets the account's gate before it asks | the gate dropped | the gate reading |
| the Filter Manager saves its rules before it answers one to run | the answer returned before the save | the save reading |
| only the Filter Manager offers Run on a Folder | `true.then(` for every row type | the Label Manager reading |

Two `--remeasure` calls: 2 records in 52 s with task 1's green; 4 in 92 s with task 2's, the
two new ones and the two task 1 records the count check flagged when the target gained tests.
Premise 7's gate re-taken before the first commit: `wx_app.rs` 199 tests, `ui_types.rs` 79,
`wx_managers.rs` 44, `managers.rs` 137, the scale harness 10 with 2 ignored, none added to
any of them. The anchor reading (records whose `before` lies in the regions edited, through
`tomllib`) found three in the files touched, the scale harness's row format at `:706` and the
Saved Searches and Quick Steps loaded arms in `wx_app.rs`; the new arm went after the saved
search's arm, so none moved.

## Ledger

Closed, both halves: 749 (the count and the question asked by nothing). Opened, both halves:
750 (`unrun-verify`, phase 14: a rule run over a folder of thousands at a real provider, a move
and a delete, on Gmail and a server with folders), 751 (`unrun-verify`, the tester's ear), 752
(`todo`, found and left: the Filter Manager's folder list names folders by the path the server
spells). Premise 1's grep for RESEARCH-4 F2 found ledger 678, so no entry was added for it.
Counts 752 in all, 668 open, 84 fixed, 0 waived.

## Deviations from Plan

**1. [Rule 3] A refactor commit before task 2's red.** A built manager held no buttons: the
loop made them as it started, so no test could read the Filter Manager's button without
running a loop only a person can end. `put_the_buttons_on` makes them now, `run_manager_loop`
still calls it for windows with no builder, and `build_filter_manager`, split out as the plan
asked, and `build_tag_manager` call it and hand the buttons to `run_the_manager_loop`. The
NVDA case's tie for Delete names `put_the_buttons_on`, where the label is written now, and the
two Delete wiring readings in `wx_managers`' tests read `run_the_manager_loop`, where the
click is bound; both had passed against the old name only because their reader runs on to
the next `fn`.

**2. [Rule 3] One green for task 1, not three.** As 13-42 found, the hook refuses a green that
leaves readings red in the same target.

**3. [Shape] `count_what_a_rule_would_change(app, account, folder, rule)`, no store argument.**
The worker opens its own store, as the saved search's worker does, because a store cannot
cross threads; one handed in would be unused.

**4. [Rule 2] A count that arrives after another account was opened is refused.** The runner
finds a message's account through `owner_of`, which falls back to the account open for a
message not on the list, and a folder's messages are usually not on it. Without the check a
count taken in one account could be run with another's folders.

**5. [Words] "Choose a filter first."** The button with no row chosen says
`manager_words::nothing_selected` for the manager's kind, as Edit and Delete do, rather than
the plan's "Choose a rule first.", so the window speaks of its rows one way.

**6. [Environment] The session locked for about four hours.** Task 2's green was refused at
20:03 by the six cases of `a_locked_key_asks_for_its_passphrase`, "OpenClipboard failed",
which is ledger 716 exactly: `LogonUI.exe` and `LockApp.exe` were running. Nothing was
committed while it was locked; the tree was polled every thirty seconds until 01:02. The first
try after the unlock was refused by key-posting cases in five targets (Alt+R, Alt+C, M, Space,
Enter posted to built controls answered nothing) that passed when run again a minute later
with nobody at the keyboard; the commit went through on the next try.

**7. [Scope] The menu item's help** is built with `format!` from
`RUNNING_A_RULE_NOW_IS_EXPERIMENTAL`, since a builder takes a string and no constant joins
two constants.

**8. [Accessibility] An account name in the chooser's label has its `&` doubled,** so a name
like "Smith & Co" is not read as a mnemonic.

**9. [Brief] Read-only tooling.** One `awk` filter and one `sed` call in diagnostic pipelines
early on, against the brief; neither wrote to a file. `rustfmt` and `cargo fmt` formatted the
files, as the gate's formatter. Throwaway probes went in the scratchpad.

**10. [Docs] The changelog does not carry the figure;** it points at the measurements page,
which holds it with its command and conditions.

### Found and left

- Ledger 752: the Filter Manager's folder list is by path as the server spells it, as the
  plan decided.
- A run's marks go one server change per message, so a run of 5,000 marks is 5,000 workers;
  carried in ledger 750 for phase 14.
- The question's dialog is not destroyed after it closes, as the two update questions beside
  it are not; a modal message box is the toolkit's own and has not been seen to leak.

## TDD Gate Compliance

Three red commits, each before its green: `ba018fcf` then `8ad48b5f`; `e06cc451` then
`24a93908`; `2c11a1f4` then `0fca6f7d`. Every named reading failed for want of its anchor or
its button, the MSAA reading listing Add, Edit, Delete and Close; nothing unnamed failed, and
the second red also named the count check, since the target gained tests that two records
name. The companions passed in each red, since they read snippets. Every expected value was
reasoned before its green: the order of the calls from the code to be written, "Run on a
Folder..." as the name Windows gives a button labelled with a mnemonic, and the scale row's
kind from the list it joins.

## Threat Flags

None beyond the register. T-13-44-01: the gate before the question and again in the runner;
a reading and a record. T-13-44-02: the question before the runner; a reading and a record.
T-13-44-03: `yes_no_where_enter_answers_no` from 13-43's `enter_answers_yes`. T-13-44-04: the
save before the answer; a reading and a record. T-13-44-05: the count on a worker, a step
sentence, 430.02 ms at 200,000 rows. T-13-44-06: the folder on screen within the account
being worked in, or chosen from the rule's own account's folders, and a count refused when
the account changed. T-13-44-SC: no crate added to `Cargo.toml`.

## Known Stubs

None. Ledger 749's stub is closed: a rule is chosen, counted, asked about and run from either
door.

## Self-Check: PASSED

`tests/a_rule_runs_over_a_folder_when_asked.rs` exists; `ba018fcf`, `8ad48b5f`, `54439c62`,
`e06cc451`, `24a93908`, `2c11a1f4` and `0fca6f7d` are on the branch.
`grep -c '"Run a Ru&le on This Folder..."' src/presentation/wx_app.rs` 1;
`fn count_what_a_rule_would_change` 1; `UIUpdate::ARuleWasCounted` 3, the arm, the send and a
doc comment;
`"&Run on a Folder..."` in `wx_managers.rs` 1; `fn runs_on_a_folder` 2;
`pub fn build_filter_manager` 1; `a rule counted over the folder` in the scale harness 2 and
on the measurements page 1.
