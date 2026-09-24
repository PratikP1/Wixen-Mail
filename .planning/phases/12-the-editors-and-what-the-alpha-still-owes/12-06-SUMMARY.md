---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 06
subsystem: spin controls, the field a person types in, and the numbers in Settings and the account editor
tags: [edit-01, spin-control, msaa, uia, annotation, settings, "#35", "#73"]
status: complete

requires:
  - phase: 12-the-editors-and-what-the-alpha-still-owes
    provides: "12-05 merged at 6562e753; 12-04's measurement that a naming object can erase a control's items"
provides:
  - "src/presentation/accessibility/names.rs: name_the_spin_control, name_and_describe_the_spin_control, what_the_typing_field_carries, the typing_field module over IAccPropServices; 12 cases"
  - "src/application/reading_habits.rs: MarkReadWay, MarkRead::parts, from_parts, DEFAULT_WAIT_SECONDS, LONGEST_WAIT_SECONDS, offered_index onto the three ways; MarkRead::ALL and MarkRead::label gone; 25 cases"
  - "src/presentation/wx_settings.rs: Font size, Default reminder and Mark as read after's seconds as spin controls, held, chosen_way, within"
  - "src/presentation/wx_account_manager.rs: the check interval as a spin control on the schedule's own bounds, the spin builder taking a range and a description"
  - "tests/every_spin_control_names_the_field_a_person_types_in.rs: 15 readings over 25 spin controls in five built windows, passing in a test process while the running program fails the naming (ledger 593)"
  - "four guard records written, one existing one re-measured"
affects: [12-08, which binds keys on the item form's spin controls and meets Page Up and Page Down doing nothing; 12-12, which reads the pages as one and runs the whole suite]

actuals:
  tokens: 18500
  tasks: 3
  commits: 8

tech-stack:
  added: []
  patterns:
    - "A spin control is named on both of its windows: the arrows through the accessible object, the buddy edit through IAccPropServices::SetHwndPropStr, which MSAA and UI Automation both read in a test process and which the running program's scan did not see (12-06.1)"
    - "A reading that finds controls by window class leaves out the toolkit's own instances by their parent's class, never by the missing property under test"
    - "A test chooses in a closed list with Up and Down sent to it, so the list itself tells wxWidgets; a hand-made CBN_SELCHANGE moved the selection and reached no handler"

key-files:
  created:
    - tests/every_spin_control_names_the_field_a_person_types_in.rs
    - .planning/phases/12-the-editors-and-what-the-alpha-still-owes/12-06-SUMMARY.md
  modified:
    - Cargo.toml
    - src/presentation/accessibility/names.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/wx_settings.rs
    - src/presentation/wx_item_form.rs
    - src/presentation/wx_compose.rs
    - src/presentation/wx_send_later.rs
    - src/application/reading_habits.rs
    - tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md

key-decisions:
  - "Mark as read after built as the planner's default (README decision 10): Immediately, After a number of seconds, Never, with a seconds spin control 1 to 600 reachable only while a wait is chosen. No message from the orchestrator overruled it."
  - "The ports stay typed: a port is a number somebody copies from a provider, not one they step to."
  - "A description travels to the typing field as well as the name, because focus never reaches the arrows; a helper pair mirrors set_accessible_name and set_accessible_name_and_description."
  - "The check interval's range is checking_on_a_schedule's own SHORTEST and LONGEST_INTERVAL_MINUTES, so the control and the schedule hold one pair of numbers."

requirements-completed: []

duration: 62 min
completed: 2026-09-23
---

# Phase 12 Plan 06: Spin Controls and the Field a Person Types In Summary

**The check interval, Font size, Default reminder and Mark as read after's seconds are spin controls holding their own ranges. The typing field's name, which the plan was mostly about, does not work in the running program: it reads correctly in tests and the pull request's scan of the real app found the fields as before. The cause is unknown and is split out to 12-06.1.**

Started 2026-09-23T21:07Z; seven commits on branch `12-06-spin-controls` from `main` at `b00be9c0`, then this documents commit after Pratik chose on 2026-09-24 to split the naming out.

## What does not work: the typing field's name

On 2026-09-23 the Accessibility scan on pull request #97 launched the real program and read it from outside, twice (runs 35927025769 at `fc17f46c` and 35930014324 at `83f394d8`). It found the spin controls' typing fields as they were before this plan:

| Window | MSAA walk | Axe.Windows over UI Automation |
|---|---|---|
| Edit Event | 8 fields with no name | 9 violations |
| Send Later | 4 fields with no name | 6 violations |
| Account editor | fields named by the label before them, `Check Interval (min):` and `Then remove it after this many days (0 for never):` | 0 |
| Settings | `Font size:`, the label with its colon | 0 |

Every test here says the opposite, and so does every probe taken to reproduce the app's conditions: in the test process, from a second process the test starts (the way a screen reader reads), on this machine and in CI's Test Suite, in debug and release, with the program's manifest activated, hidden and shown, and five seconds later. The annotation is written, and it is visible everywhere except the running program. What the app has that a test process lacks is not found. Pratik chose on 2026-09-24 to split this out as 12-06.1 and to add no diagnostics on this branch. The code that writes the name stays, because it is right in every place anything can read it and 12-06.1 starts from it; its doc comment says it does not reach the app.

So the rest of this section's first draft, which said "every field answers its arrows' name on both channels", describes a test process. Read on the built windows in the test process before the change (the red run of `6fe17bff`), 19 spin controls in five windows:

| Before, in a test process | After, in a test process only |
|---|---|
| 10 fields answered no name on MSAA and none on UI Automation (event form: Starts, Ends and Last day Day and Year, both Minutes; Send Later: all four) | every field answers its arrows' name on both channels |
| 9 fields answered their static label's raw text on both channels (`Rows:`, `Columns:`, `Start time`, `End time`, the two Compose spinners' and POP days' labels with their colons) | the same |
| 3 descriptions on the arrows only (the hold, Alert minutes before, How many times) | each reaches the field |

## What works, measured

After task 2 there are 25 (2 in the account editor, 5 in Settings, 12 in the event form, 2 in Insert Table, 4 in Send Later), and the reading holds all 25. The whole object was read as well as the name, as 12-04's MSAA finding asks: every named field answers role text (0x2a) and the same child count as a bare spin control's field, and every set of arrows the same role and child count as a bare one's, so naming erased nothing. A companion builds a spin control named on its arrows alone and sees its field nameless, so the reading sees the defect the ledger describes.

The four numbers: check interval 1 to 60 (the schedule's own bounds), Font size 8 to 72, Default reminder 0 to 1440, Mark as read after's seconds 1 to 600, each read back with `UDM_GETRANGE32`. Typing 20, 45 and 45 into the three Settings fields and choosing After a number of seconds, OK writes `font_size` 20, `default_reminder_minutes` 45 and `mark_read_after` "45"; choosing Immediately and Never writes "immediately" and "never". Built over a stored "never", the seconds field is disabled; choosing After a number of seconds with the Down key enables it, Immediately disables it; built over "5" it shows 5, enabled.

**Measured and not claimed: Page Up and Page Down do nothing in a spin control on this toolkit.** A throwaway probe (not committed) sent each key to a spin control's field at 100 in 0 to 1440: Up 101, Down 100, Page Up 100, Page Down 100. The plan's first truth said "Page Up and Page Down by more" on the strength of premise 3's "Up, Down, Page Up and Page Down are the native control's"; the native control handles the arrows only. The pages and the changelog say Page Up and Page Down do nothing. A step other than one is 12-08's key handler by premise 3, and 12-08's executor should read this before assuming the native control helps.

What only a person can settle: whether NVDA speaks the value after Up and Down, and reads Mark as read after's three entries and the seconds (ledger 592). The field's name is 12-06.1's first.

## The red halves

| Commit | Named red | Green against the stub, by design |
|---|---|---|
| `6fe17bff` | 5 per-window naming readings, the UI Automation reading, the description reading, and `names::tests::test_a_spin_controls_description_travels_to_the_field_a_person_types_in` against a stub answering the name alone | the census (1, 2, 12, 2, 4), the whole-object pin, the companion that builds the defect |
| `f022934d` | the ranges, the census moving to 2 and 5, the Settings save, the words saved, when the seconds are offered, the five rewritten `reading_habits` cases against stubs, and the count check (the target gained four tests two records name) | none |

Both passed the gate's red mode: "a red that is exactly the one this commit named".

## Guard records

| Record | Break | Red, measured |
|---|---|---|
| every spin control's typing field is named through the annotation service | the annotation loop runs over nothing | the 7 naming readings, nothing else |
| the event form's minute spinner is named on its typing field too | both minute spinners named through `set_accessible_name` alone | the event form's reading and the UI Automation reading |
| the check interval spin control holds 1 to 60 minutes as its own range | the range widened to 0 to 3650 | the four numbers' range reading alone |
| a stored never is shown as never, not as a wait | `parts` shows Never as a wait of nought | 3 `reading_habits` cases (the library, which is its suite) |
| a message nobody read is never marked read, whatever is selected (existing) | unchanged; one case it names now iterates six answers of its own | its 2 named cases, exact |

`guards.sh --remeasure` runs: 43 s (two records), 306 s (five records, the count check's two plus three), 39 s (the interval record re-anchored by the refactor). The sweep header's count of records arrived since moved 257 to 261.

## Commits and what each hook run printed

| Commit | What | Hook |
|---|---|---|
| `6fe17bff` | test: failing readings of every spin control's typing field | red, 173 s |
| `b439452e` | feat: name every spin control's typing field on both channels | affected, 150 s |
| `f022934d` | test: failing readings of the four numbers as spin controls | red, 136 s |
| `42f4213f` | feat: the check interval and three Settings numbers as spin controls | affected, 175 s |
| `b331b1e9` | refactor: the check interval's range is the schedule's own bounds | affected, 134 s |
| `fc17f46c` | docs: the pages, the ledger, the summary and the four marks | docs_only, 89 s |
| `83f394d8` | test: read every spin control's typing field from another process | affected, 102 s |
| this commit | docs: the naming split out to 12-06.1, the ledger reopened | |

## The pull request's runs

Pull request #97, two runs, read 2026-09-23:

| Check | `fc17f46c` | `83f394d8` |
|---|---|---|
| Rustfmt, Clippy, Security Audit, Build (debug and release), Setup Executable | pass | pass |
| Test Suite | pass, 19m02s | pass, 18m52s, the cross-process reading included |
| NVDA screen reader tests | pass, 24m49s | fail, 10m31s: NVDA never started on the runner ("Timed out waiting for NVDA to be running"), not this change |
| Accessibility scan | job passed (it may not fail the build); findings above | the same |
| Mutants in the change | fail: `test_the_share_of_history_before_red_green_is_computed_and_printed` needs `.git` in cargo-mutants' copy, ledger 458 | not waited for |

The documents commit that splits the naming out was pushed without waiting for CI, on the coordinator's instruction.

## Deviations from Plan

**1. [Rule 2] Send Later named in its own file.** Premise 1 said Send Later's spinners are the item form's builders, "so naming them there names both windows". The builders build and name nothing; each window names its controls after the builder returns, and Send Later did so with `set_accessible_name` in `wx_send_later.rs`, a file the plan did not list. Its four calls go through the helper, and `build_the_asking_dialog` is `pub` so the reading builds Send Later too. Ledger 419 to 422 could not have closed otherwise.

**2. [Rule 2] Insert Table and Send Later read, and descriptions carried.** The plan's reading covered four windows; it reads five, since Send Later's entries are among the twelve. The hour spinners and the whole-number fields carried help sentences on their arrows only, so the helper has a describing twin and the field gets `PROPID_ACC_DESCRIPTION` too.

**3. [Rule 3] The three `windows` features arrived in the red commit.** The reading's UI Automation half uses `IUIAutomation` from `Win32_UI_Accessibility`. `Cargo.lock` is unchanged; the crate's header comment says what the features are for.

**4. The acceptance count of builder sites against helper calls does not hold, and could not.** It was a proxy that took construction sites and naming sites for one set. Today: `grep -rn 'SpinCtrl::builder' src` 13 sites, helper calls 15 (`name_the_spin_control(` and `name_and_describe_the_spin_control(`), because the item form names five builders' controls through one function and Send Later names four more in its own file. The reading's census over the built windows, 25 spin controls, each named, is the criterion that says every spin control goes through the helper.

**5. `names.rs`'s pure part is `what_the_typing_field_carries`, not `spin_field_name`.** A function returning its argument would test nothing; which properties the field carries is the real decision, and its case is the description travelling.

**6. Three test files outside the list followed the type changes.** `theme_reach` checks the interval field as a spin control, apart from the text fields' array; the Settings focus reading compares focus with the font size's typing field, because `set_focus` on a spin control lands in the buddy; and the record-named `reading_habits` case about a message nobody read iterates six answers now that `MarkRead::ALL` is gone, re-measured exact.

**7. Page Up and Page Down.** Measured doing nothing; not built and not claimed (above).

**8. The naming did not reach the running program, and the plan is split.** The plan's second truth, its first `[D]` line and its twelve ledger closures rest on the typing field being named. The pull request's scan found it not named in the app; the tests found it named. Pratik chose on 2026-09-24 to merge the spin controls and move the naming to 12-06.1. The reopened ledger entries and the unticked EDIT-01 say so. A seventh commit, `83f394d8`, added the cross-process reading while looking for the cause; it passes, did not reproduce the fault, and is kept as a reading 12-06.1's fix must still pass (ledger 593).

**9. `read_the_calendar_and_pim_page` lost its `base` parameter**, which only the removed parse fallback read.

**Total deviations:** 9. One is the plan's main claim failing in the running program; the rest are the reading reaching further than the plan asked, and the one plan truth the toolkit cannot meet, Page Up and Page Down, measured and said.

## Tests

| Target | Before | After |
|---|---|---|
| `tests/every_spin_control_names_the_field_a_person_types_in.rs` | new | 15, and 1 ignored child the cross-process reading starts |
| `presentation::accessibility::names::` | 11 | 12 |
| `application::reading_habits::` | 25 | 25 (five rewritten in place, one renamed) |
| `presentation::wx_item_form::` | 14 | 14 |
| `presentation::wx_account_manager::` | 14 | 14 |
| `presentation::wx_settings::` | 0 | 0 |
| `data::config::` | 68 | 68 |

Criteria greps, 2026-09-23 on the branch: `tf_with_description("Check &Interval` 0, `.clamp(1, 60)` 0 in the account manager, `.clamp(8, 72)` or `.min(1440)` 0 in Settings; `grep -c '^\s*"Win32_' Cargo.toml` 11, the three new ones 3, `[dependencies]` 1, `git diff main -- Cargo.lock` empty; `grep -c 'spin control' docs/USER_GUIDE.md` 5 and `docs/changelog.md` at least 1.

## Threat model

T-12-21: `parts` and `from_parts` round-trip immediately, never and waits of 1, 2, 45 and 600, and a stored 5000 shows as 600, never as Immediately; the two settings guards are green at 68. T-12-22: a zero buddy is said at debug and the arrows keep their name; the reading counts the spin controls it finds. T-12-23: the reading asserts each field's name equals its own arrows' on 25 spin controls, in a test process; in the running program no name reached the field at all, so nothing wrong reached it either. T-12-SC: no crate added, three features of the `windows` crate already in the tree.

## Known Stubs

None.

## Ledger

408, 409, 410, 412, 413, 414, 419, 420, 421, 422, 424 and 425 stay open. This branch's documents commit `fc17f46c` marked them fixed on the test's reading, and the scan contradicted it the same day, so they were reopened in both halves with a sentence each pointing at 12-06.1. 411, 415, 418 and 423 are the AM or PM lists' and the Send on Month list's value elements, not spin controls, and 416 and 417 are the Category combo box and the form's height: all six stay open, not this plan's. 592 opened for the ear; 593 opened because the reading cannot see the fault. The header reads `open_count` 551 and `fixed_count` 42 against 549 and 42 before the plan.

## Exception set

Nothing tracked was rewritten by script. `cargo fmt` formatted Rust files; `scripts/guards.sh` applied and restored its own breaks in four runs, as the project's tool does. Five throwaway probes, `tests/zz_probe_spin_keys.rs`, `zz_probe_spin_names.rs`, `zz_probe_com.rs`, `zz_probe_v6.rs` and `zz_probe_later.rs`, were written with the Write tool, run, and deleted with `rm`; none was ever tracked. Every other edit, the twenty-four ledger edits of the reopening included, was by hand.

## Self-Check: PASSED

Files: the target, the summary, and every modified file above exist. Commits `6fe17bff`, `b439452e`, `f022934d`, `42f4213f`, `b331b1e9`, `fc17f46c` and `83f394d8` are on the branch. Both ledger halves agree: 593 entries, 551 open and 42 fixed in each.
