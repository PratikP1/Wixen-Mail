---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 8
subsystem: the item form's times, and Settings' Calendar and PIM tab
tags: [edit-03, "#41", time-blocks, spin-control, settings, keyboard]
status: complete
requires:
  - phase: 12
    provides: "12-07 merged at dc768391; 12-06.1's names::ready_the_annotation_store, so spin fields are named in the running program"
provides:
  - "src/application/time_blocks.rs: Block, next_boundary, step_by_block, step_by_minute, end_after, follow; 17 cases"
  - "src/presentation/spin_field_keys.rs: the_arrow and take_the_arrows, a subclass on a spin control's typing field above the control's own arrows; 3 cases"
  - "src/data/config.rs: event_length_minutes, 30 by default and for an older file"
  - "src/presentation/wx_settings.rs: New events last, a Choice in the Calendar section of the Calendar and PIM tab"
  - "src/presentation/wx_item_form.rs: Timekeeping read where the form opens, Opening, Moment, Following, take_the_arrows_in"
  - "tests/event_times_move_in_blocks.rs: 16 readings over built event, reminder and task forms"
affects: [12-09, 12-10 and 12-11, which change the item form or Settings next; 12-12, which runs the phase's full gate and reads the pages]
tech-stack:
  added: []
  patterns:
    - "A key the native control handles before wxWidgets forwards it is taken on the control's own window with SetWindowSubclass, above the control's subclass"
    - "A spin control's typed change is heard on its text event; its value event is not raised by typing"
key-files:
  created:
    - src/application/time_blocks.rs
    - src/presentation/spin_field_keys.rs
    - tests/event_times_move_in_blocks.rs
  modified:
    - src/application/mod.rs
    - src/presentation/mod.rs
    - src/data/config.rs
    - src/presentation/wx_settings.rs
    - src/presentation/wx_item_form.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - "The arrow keys are taken on the spin control's typing field, because a handler bound on the control never sees Up or Down (measured). The plan's fallback, named in its premise 1."
  - "A new item's times are decided in the form, from the moment it opened and the block, not in managers.rs: the target builds the form directly, and managers.rs was left unchanged."
  - "New events last sits in the Calendar section of the Calendar and PIM tab, not beside the reminder lead: a new event's length is a question about the calendar."
  - "The label is New events last, not the plan's New events, tasks and reminders last: a task has no time, and a reminder has no length."
requirements-completed: [EDIT-03]
duration: about 200 min
completed: 2026-09-24
estimate:
  tokens: 29000
actuals:
  tokens: 62000
  tasks: 3
  commits: 6
---

# Phase 12 Plan 08: Event Times in Blocks Summary

**Event and reminder times move in blocks of 15, 30 or 60 minutes, set by New events last on the Calendar and PIM tab and 30 by default. A new event starts at the next block boundary after the form opens and ends one block later. Up and Down on a time's minutes move it by the block, Left and Right by a minute, and typing still works. The end follows the start until the person changes it. Read in built forms by the target; not yet heard under NVDA or seen by the pull request's scan when this was written.**

On branch `12-08-time-blocks` from `main` at `dc768391`, 2026-09-24. `Cargo.toml` and `Cargo.lock` unchanged: `git diff main --stat -- Cargo.toml Cargo.lock` prints nothing.

## Where the key arrives

Measured first, as the plan asked, by `test_right_reaches_a_handler_on_the_spin_control_and_up_does_not`: a counter on a bare spin control's key-down, Right and then Up sent to its typing field. The handler saw Right once and Up never, and the value went from 0 to 1. The control's own arrows take Up and Down in the field before wxWidgets forwards the key. So `presentation::spin_field_keys::take_the_arrows` subclasses the field with `SetWindowSubclass`, above the arrows' own subclass, and a key it takes goes no further. Two more measurements on the way:

- A key-down sent without the extended-key bit reaches wxWidgets as the numeric keypad's arrow (378 for Right). The readings send the bit the arrow keys carry. The subclass reads the virtual key, so both kinds of arrow work.
- Typing over a spin control raises its text event and not its value event: with the value event alone, the typed-end readings stayed wrong. The form listens to the text event, which the hour's own step of one raises too. Taken red by hand with the text event unbound: three readings went red, the hour step and both typed-end readings.

## Tests

| Target | Count |
|---|---|
| `application::time_blocks::` | 17, new |
| `presentation::spin_field_keys::` | 3, new |
| `data::config::` | 68, unchanged; the older-file test rewritten in place |
| `presentation::wx_item_form::` | 16, unchanged (the plan's 15 was one short) |
| `presentation::managers::` | 137, unchanged |
| `tests/event_times_move_in_blocks.rs` | 16, new |
| `house_style` 74, `the_words_that_say_nothing` 10, `a_key_is_documented_where_the_surface_that_binds_it_is` 3, `docs_links` 6 | pass |

Every target that builds the item form passed on the green tree: `item_form_date_time_fields`, `item_form_prefill`, `item_form_validation`, `item_form_free_busy`, `item_form_recurrence_tab`, `checkbox_labels`, `every_spin_control_names_the_field_a_person_types_in` (15, 1 ignored), `a_spin_controls_field_is_named_where_the_scan_reads_it` (4, 2 ignored), `theme_reach` (7).

The plan's greps: `pub event_length_minutes` in `config.rs` 1; `event_length_minutes` in `wx_settings.rs` 2 and in `wx_item_form.rs` 1; `load_stored` in `wx_item_form.rs` 1; `time_blocks::` in `wx_item_form.rs` 8; `next_boundary\|end_after` 2; `Left` in The Event Window 2.

## Commits

| Commit | What | Hook |
|---|---|---|
| `0602ca0d` | test: time blocks and the setting, 19 named | red, 156 s |
| `e560fe26` | feat: the rules, the setting, the read where a form opens | affected, 157 s |
| `7f20c7fa` | test: the readings on built forms, 12 named | red, 73 s; the first attempt was refused at 78 s, see deviation 6 |
| `31f3b036` | test: which keys a spin field hands to the form, 3 named | red, 145 s |
| `68990f4c` | feat: the keys, the opening times, the end following | affected, 148 s |

This documents commit, the pull request's runs and the merge are in the executor's report.

## Guard records

Four written and measured, the arrived-since count raised from 274 to 278:

| Record | Break | Red |
|---|---|---|
| a new event starts at the next block boundary after now, not the nearest | rounds to the nearest boundary | 4 `time_blocks` cases |
| an end the person changed does not follow the start | the edited end follows anyway | 1 case |
| an arrow key the form takes in a spin field does not also reach the control's own step | the key goes on after the step | 10 readings, suite the target |
| a new event's form opens on the next block boundary rather than on the moment it opened | opens on the moment | 12 readings, suite the target |

Each agreed exactly on its second run; the two form records' first runs named one test each and reported 9 and 11 more, which were written down.

## Ledger

Opened, both halves: 603, #41 under NVDA; 604, found by this plan and not its subject: a new event or reminder opened while Settings holds a default reminder is headed Edit Event, because `starting_alert_for` hands the alert in as a prefill. The front matter reads 604 entries, 549 open and 55 fixed. None closed.

## Deviations

1. **The key is taken on the typing field, not on the spin control** (above). The plan's own fallback. `spin_field_keys.rs` is a file the plan did not list.
2. **A task has no time.** `item_fields::TASK` holds a due date and nothing else, so the plan's "the same form for a task ... opening at the same times" cannot hold. The reading says so and holds a task's due date to today, even at ten to midnight. Rule 1.
3. **`managers.rs` unchanged; the opening times are the form's.** The target builds the form directly with a fixed moment, so the times are decided there. The plan's guard record on `managers.rs` is on `wx_item_form.rs`.
4. **The older-file test is a different one.** The plan named `test_a_settings_file_written_before_reading_was_a_setting_still_loads`, which is about the nested `reading` key. The one rewritten in place is `permission_tests::test_a_settings_file_written_before_these_existed_reads_the_way_it_should`.
5. **The setting's label and place.** New events last, in the Calendar section, with `&l`, a letter no other control on the tab uses (`o`, `r`, `a`, `e`). The plan's label named tasks, which have no time.
6. **The target's first red commit was refused** by `test_no_comment_says_a_test_cannot_build_a_window`: its head comment repeated the sentence about one application per process. Reworded; nothing else changed.
7. **Keys documented after the code.** `docs/KEYBOARD_SHORTCUTS.md` changes in this documents commit rather than in `68990f4c`, which bound the keys. The rule is the same commit; this broke it.
8. **One reading added at green**, the hour's own step with the end following. It was taken red by hand, not committed red.
9. **Actual tokens 62,000 against an estimate of 29,000**, chars/4 over `git diff main` before this commit: the subclass module and the third red/green pair were not in the estimate.

## Known stubs

`spin_field_keys::the_field::take` does nothing off Windows. Like `set_accessible_name`, it is a Windows-only layer, and whether the arrows reach a key handler there has not been measured. This is the project's standing platform gate, not a stub within Windows.

## Scripted edits

None. Every tracked file was changed with Read, Edit and Write. `cargo fmt` formatted, and `scripts/guards.sh` applied and restored its own breaks. Two hand breaks, each made with Edit: one reverted with Edit, and the other restored with `git checkout -- src/presentation/wx_item_form.rs`. One stray `sed -i.bak '' /dev/null` ran in a shell line; it named no file of the tree and changed nothing.

## What only a person can settle

The time spoken after each key and the end heard following, in the event and reminder windows, and the setting's list, under NVDA: ledger 603.

## Self-Check: PASSED

The three new files and this summary exist; the five commits above are on the branch (`git cat-file -e`). `the_planning_files_agree_with_themselves` passes with this file on disk; no carriage return in any file touched (`tr -cd '\r' | wc -c` 0 each) and no em dash in the diff.
