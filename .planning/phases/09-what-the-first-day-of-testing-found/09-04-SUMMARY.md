---
phase: 09-what-the-first-day-of-testing-found
plan: 04
subsystem: settings dialog, view menu, guards
tags: [settings, tab-order, reading-tab, compose-tab, sort-menu, radio-group, guards]

requires:
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-01: the tree at 1.0.0-alpha.1 and the rule that a fix under the alpha gets a changelog entry and no bump"
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-03: a source-reading target with companions, coupled to the files it reads by a guard record's suite"
provides:
  - "Then by built into the Message List section straight after the sort row, so it is the tab stop after Default sort order; held by tests/the_sort_controls_sit_together.rs, which builds the real dialog and walks the panel's children"
  - "Cc and Bcc lines built by build_compose_tab into a Writing section, first on the Compose tab; ComposeTabControls, a named struct in place of the five-tuple"
  - "SettingsWidgets::sort_order and sort_then public for the reading; nothing else"
  - "The Sort submenu as one radio group, the three separators gone; held by tests/one_sort_is_checked.rs reading the chain and sync_sort_menu, and by tests/one_sort_is_checked_on_a_live_menu.rs asking a real menu bar"
  - "Two guard records, one per moved thing, each with suite naming its target"
affects: [09-05 onward, which write changelog entries under Unreleased; anything that walks a panel's children for tab order; the phase's last plan, which reads FOUND-06 and FOUND-07 clause by clause]

actuals:
  tokens: 9554
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "Tab order is read from the built dialog, not the source: wxWidgets moves Tab through a panel's children in creation order, so get_next_sibling from one control to the next is the order somebody hears, and a reading of which section a control is added to would be the source's opinion about that"
    - "A reading that rests on a platform claim gets a companion on the real control that builds the old shape and reads it back before and after the action, and what it answers is recorded as a measurement rather than taken from the report"
    - "A guard break for code the same plan moves is a plant at the new site, not a reversal of the move: the reversal is two edits when the old section is declared elsewhere"

key-files:
  created:
    - tests/the_sort_controls_sit_together.rs
    - tests/one_sort_is_checked.rs
    - tests/one_sort_is_checked_on_a_live_menu.rs
  modified:
    - src/presentation/wx_settings.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/changelog.md

key-decisions:
  - "Cc and Bcc lines goes first on the Compose tab in a section called Writing, before Sending, Drafts and Signatures, because it is about the window a message is written in and somebody meets the sections in order"
  - "build_compose_tab hands back a named struct rather than a six-tuple, on the Reading builder's own reason: two check boxes and two spin boxes told apart by counting"
  - "The look at a live menu the plan asked for by hand is a test on a real menu bar instead, tests/one_sort_is_checked_on_a_live_menu.rs, which runs on every whole gate; what it cannot reach, the application's own menu after a real header click, is ledger 487"
  - "Task 1's guard break plants a Write the month as static before Then by rather than building Then by into date_sec again, because date_sec is declared 250 lines below the new site and that edit is not one replacement"
  - "The live-menu target has no guard record, because it reads none of this tree's files; a record needs a file to break"

patterns-established:
  - "When a plan asks for a look by hand at behaviour a test process can drive, write the test and ledger the part of the look the test cannot make"

requirements-completed: []

coverage:
  - id: D1
    description: "Then by is built into the Message List section directly after Default sort order, so the two are adjacent tab stops; a test builds the real tab and reads the order back"
    requirement: FOUND-06
    verification:
      - kind: integration
        ref: "tests/the_sort_controls_sit_together.rs#test_then_by_is_the_tab_stop_after_default_sort_order"
        status: pass
      - kind: other
        ref: "grep -n '\"Then &by:\"' src/presentation/wx_settings.rs on main at c928cae4 -> 1264, inside build_reading_tab (1208) after list_sec (1212) and before folders_sec (1301)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Cc and Bcc lines is on the Compose tab, and test_every_setting_somebody_can_change_is_offered_by_a_screen stays green"
    requirement: FOUND-06
    verification:
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_offered_by_a_screen"
        status: pass
      - kind: other
        ref: "grep -n '\"Cc and Bcc &lines:\"' src/presentation/wx_settings.rs on main at c928cae4 -> 1008, inside build_compose_tab (991); cfg.copy_lines still written at read_settings from w.copy_lines"
        status: pass
    human_judgment: false
  - id: D3
    description: "Exactly one item in the Sort submenu is checked whichever way the sort was chosen"
    requirement: FOUND-07
    verification:
      - kind: integration
        ref: "tests/one_sort_is_checked.rs#test_the_seven_sort_items_are_one_radio_group"
        status: pass
      - kind: integration
        ref: "tests/one_sort_is_checked.rs#test_sync_sort_menu_follows_every_sort_there_is"
        status: pass
      - kind: integration
        ref: "tests/one_sort_is_checked_on_a_live_menu.rs#test_checking_one_sort_in_one_group_leaves_one_checked_and_a_separator_leaves_two"
        status: pass
      - kind: other
        ref: "grep -c 'append_separator' over the sort_menu chain on main at c928cae4 -> 0"
        status: pass
    human_judgment: false
  - id: D4
    description: "Whether the Reading tab now reads as one group by ear, and whether the application's own Sort submenu shows one tick after a real header click"
    requirement: FOUND-06
    verification: []
    human_judgment: true
    rationale: "FOUND-06's listening line and the look at the running program; ledger 487 and 488. Structure is held by tests; experience is the tester's."

duration: 40m
completed: 2026-09-16
status: complete
---

# Phase 9 Plan 04: The two sort controls together, and one sort checked Summary

**"Then by" is the tab stop after "Default sort order" on the Reading tab, both in the Message
List section, held by a test that builds the real dialog and walks the panel's children, which
is the order Tab moves in; "Cc and Bcc lines" opens the Compose tab in a Writing section, and
every setting is still offered by a screen (#36). The Sort submenu is one radio group of seven,
the three separators gone, held by a reading of the chain and by a real menu bar asked which
items it says are checked (#39). Both closed on the tree with the merge commit; whether the tab
reads as one group by ear is the tester's.**

## Performance

- **Duration:** about 40 minutes from the branch to the merge, of which about 1 minute was guard
  measurement (two runs, one record each, 14 s and 17 s) and about 13 minutes the two whole gates
- **Started:** 2026-09-16T18:55:18Z
- **Merged:** 2026-09-16T19:35Z at `c928cae4`
- **Tasks:** 2
- **Files modified:** 7 (three created)

## What landed

**The Reading tab.** `sort_then` is `labelled_choice(panel, &list_sec, ...)` now, built straight
after `list_sec.add_sizer(&sort_row, ...)` with a comment saying why it is there and what puts it
in the tab order. Its label, accessible name, choices and index are unchanged. The reading's own
answer, quoted from the red run against the tab as it was, is the tester's sentence measured:

```
Default sort order to Then by: the control is not within 3 tab stops; what was met first:
  ["Start in All Inboxes", "Folders and Message Lists", "Unread on a folder or account that holds others:"]
what is behind Then by: "Write the month as:" is 3 tab stops behind it, so the control still sits
  in the Dates and Times section rather than with the sort it is the second level of
```

After the move, walking `get_next_sibling` from `sort_order` meets `["Then by:"]` and then the
`sort_then` window itself, so what sits between the two is Then by's own label and nothing else,
and the `get_prev_sibling` chain from `sort_then` passes no "Write the month as:". That is the
sibling reading; the plan offered a position reading as the alternative and it was not needed.
A static box for each section is a child of the panel too, which is why "Folders and Message
Lists" appears in the walk above.

**The Compose tab.** `build_compose_tab` opens with a section called Writing holding one control,
"Cc and Bcc lines", before Sending, Drafts and Signatures: it is about the window a message is
written in, and somebody meets the sections in order. It hands back `ComposeTabControls`, a struct
with six named fields, rather than a six-tuple with two check boxes and two spin boxes told apart
by counting, which is the reason `ReadingTabControls` gives for itself. `copy_lines` moved from
`ReadingTabControls` to that struct and under the Compose comment in `SettingsWidgets`;
`read_settings` writes `cfg.copy_lines` from `w.copy_lines` as before.
`test_every_setting_somebody_can_change_is_offered_by_a_screen` passed on the green, quoted:

```
test data::config::every_setting_is_acted_on::test_every_setting_somebody_can_change_is_offered_by_a_screen ... ok
```

`house_style`'s `test_no_settings_control_is_built_and_then_forgotten` read the struct as handed
back without a change. `sort_order` and `sort_then` are `pub` on `SettingsWidgets` with a comment
saying which test and why; nothing else became public.

**The Sort submenu.** The three `.append_separator()` calls are gone from the chain, and the
comment above it says why there are none. `sync_sort_menu` is unchanged, as the plan said. On
`main` at `c928cae4`:

```
grep -c 'append_separator' <(sed -n '/let sort_menu = Menu::builder()/,/\.build();/p' src/presentation/wx_app.rs)
0
```

**What the live menu said, which is more than the issue did.** The plan asked for a look at the
running program, by hand, once. `tests/one_sort_is_checked_on_a_live_menu.rs` asks a real menu bar
the same question by id, on every whole gate: seven radio items in one group, `check_item` on the
third then on the first, as `sync_sort_menu` does on the column-header path, and all seven read
back through `MenuItem::is_checked`. One group answers `[0]` as built, `[0]` after sender then
date, `[5]` after subject descending. The companion builds the tester's shape, a separator after
the second, fourth and sixth, and it answers `[0, 2, 4, 6]` as built and `[0, 2, 4, 6]` after the
same two checks. The first draft expected `[0, 2]`, written from the issue's "after sorting by
sender, date newest still reads as checked", and the menu corrected it: wxWidgets checks the first
item of every radio group as it is made, so the menu the tester met showed four ticks from the
moment it was built, before anybody had sorted anything, and sorting moved one of the four. The
comment above the chain and the changelog entry say that rather than "up to four ticks for one
sort". Observation 611 in the skill observation log.

**The chain reading.** `whether_the_sort_items_are_one_group` finds `let sort_menu =
Menu::builder()` through `what_ships`, lists everything the chain appends in order, requires the
seven `ID_SORT_` ids in order (six is a complaint naming the count, T-09-14), and complains at the
first thing appended between the first radio item and the last, naming the item it follows: "a
separator is appended after ID_SORT_DATE_OLDEST". Companions splice a separator back after the
second item and take the seventh item out. `whether_the_menu_follows_every_sort` reads
`MailSortOption`'s variants from `ui_types.rs` and requires `sync_sort_menu` to name each with no
wildcard arm; its companion plants `_ =>` and then removes an arm.

## Task commits

| Commit | What |
|---|---|
| `80e32324` | test(09-04): the red half of task 1, one reading named, three companions green on arrival, two fields public |
| `35d868e3` | feat(09-04): Then by beside Default sort order, Cc and Bcc lines on Compose in Writing, `ComposeTabControls`, one record, the #36 changelog entry |
| `f67bbdaa` | test(09-04): the red half of task 2, one reading named, four tests green on arrival |
| `139aaccb` | feat(09-04): the separators gone, the comment, the live-menu target, one record, the #39 changelog entry |
| `c928cae4` | Merge 09-04 into `main` |

Branch `the-sort-controls-sit-together-and-one-sort-is-checked` from `main` at `9d85e666`. Not
pushed; 45 commits unpushed before the commit that lands this summary.

## Honest RED and GREEN

Task 1's red commit names one test, `test_then_by_is_the_tab_stop_after_default_sort_order`, red
for the tester's reason with the output quoted above; the three companions, each on a panel built
in the test to the shape it plants, were green on arrival because they hold the reading and not
the tab, and the commit says so. The two `pub` words went in the red commit so the target compiled.
Task 2's red commit names one test, `test_the_seven_sort_items_are_one_radio_group`, red with "a
separator is appended after ID_SORT_DATE_OLDEST"; green on arrival and said so: the separator
companion (the tree already had one there), the `sync_sort_menu` reading (the plan asked for it as
a hold) and its companion, and the missing-item companion. The live-menu target arrived in task 2's
green commit; its one test was red once by hand, against the expectation `[0, 2]`, and that red is
the finding above rather than a broken test, so it was corrected to what the menu measures and
committed green. Not a second red commit, on the rule of one per task. The count check did not
fire on either task: no test was added to a file a record names.

## What the gate selected

| File | On the branch |
|---|---|
| `tests/the_sort_controls_sit_together.rs` | `--test the_sort_controls_sit_together` (task 1 red, as a changed target; task 1 green, as coupled) |
| `src/presentation/wx_settings.rs` | `--lib presentation::wx_settings::` (matches nothing) plus the coupled targets: `every_event_has_a_control`, `checkbox_labels`, `the_language_the_screen_shows_is_the_one_used`, and from the green `the_sort_controls_sit_together` fourth (task 1 red and green) |
| `tests/one_sort_is_checked.rs` | `--test one_sort_is_checked` (task 2 red, as a changed target; task 2 green, as coupled) |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app::`, 199 tests, plus the coupled targets: ten before this plan and `one_sort_is_checked` eleventh (task 2 green) |
| `tests/one_sort_is_checked_on_a_live_menu.rs` | `--test one_sort_is_checked_on_a_live_menu` (task 2 green, as a changed target; coupled to nothing) |
| `guards/guards.toml`, `docs/changelog.md` | no scoped target; both rode code commits, so no `docs_only` run happened |

Every commit ran the whole-tree guards. `scripts/check.sh all` on the branch at `139aaccb`, run
once, output to a file and exit status read directly, never piped: exit 0 in 389 s, 7,819 passed
and none failed over 65 result lines, the release build included. Seven more than 09-03's 7,812:
one in `the_sort_controls_sit_together`, five in `one_sort_is_checked`, one in
`one_sort_is_checked_on_a_live_menu`. The keyring race (ledger 374) did not appear. `main`'s hook
ran `all` again on the merge: 7,819 and none failed.

## Guard records

839 records by the TOML reader before, 841 after; census 802 + 37 before, 802 + 39 after, the
line at `guards/guards.toml:84` moved in each green commit. Two new records, none corrected, two
measured across two runs on their own targets, `WIXEN_TEST_THREADS` untouched.

| Record | File and suite | Break | Red |
|---|---|---|---|
| Then by is the tab stop after Default sort order, not after Write the month as | `wx_settings.rs`, `the_sort_controls_sit_together` | a static labelled "Write the month as:" built straight before "Then by" | 1: the reading, on both of its halves |
| the seven sort items are one radio group, not four with a tick each | `wx_app.rs`, `one_sort_is_checked` | the separator after Date (Oldest First) put back | 1: the reading; the separator companion stays green because it plants a second and the complaint is the same |

Both measured by `scripts/guards.sh --remeasure`, 14 s and 17 s by the runner's `timed:` lines,
"the one test named went red, and nothing else did" each time. Counts written: `wx_settings.rs` 0,
`wx_app.rs` 199, the two targets 1 and 5. `bash scripts/check.sh --suites-for guards/guards.toml
src/presentation/wx_settings.rs` names the new target fourth; for `wx_app.rs`, eleventh.

The live-menu target has no record: it reads none of this tree's files, and a record is a break
in a file. What would break it is a wxdragon or wxWidgets change, and a `Cargo.toml` change runs
the whole gate.

## Premises the tree contradicted

1. **`:1212` is the restored layout at startup, not a column-header sort.** The plan's premise 3
   read `sync_sort_menu(` at `:1212` and `:3102` as "a column-header sort and the menu's own".
   `:1212` (now `:1212` still) is inside the startup path restoring a saved column layout, "A
   restored layout carries a restored sort"; `:3102` is the header-click handler; and the menu's
   own path, `sort_from_menu`, does not call `sync_sort_menu` at all, because choosing a radio
   item checks it. Nothing in the fix turned on this, since with one group all three paths are
   right, but the premise was wrong about which paths there are.
2. **The plan's break for task 1 does not compile as one replacement.** "`sort_then` built into
   `date_sec` again": `date_sec` is declared at `:1548`, 287 lines below the new site at `:1261`,
   so `&list_sec` to `&date_sec` is a compile error and the honest reversal is two edits. The
   record plants the tester's neighbour at the new site instead; its comment said "250 lines"
   in the green commit, a guess written before it was measured, and the number came out of it
   in the commit that lands this summary. Observation 610.
3. **The tester's menu showed four ticks as built, not up to four after sorting.** Above.
4. **Line numbers moved by 37 to 38** since the plan's premises were taken at `524ff24f`: on
   `main` at `9d85e666`, `list_sec` at `:1175`, "Then &by:" at `:1556`, "Cc and Bcc &lines:" at
   `:1564`, the sort chain at `:6183` with separators at `:6194`, `:6205`, `:6216`, and
   `sync_sort_menu` at `:14206`. The shapes held exactly.

Premises 1, 2 and 4 held otherwise: the fields and their reads, the sibling API on `Window`,
seven pages, `wx_settings.rs` at zero unit tests and three coupled targets (four now),
`wx_app.rs` at 199 by the runner, and libtest's ORed filters (the verify ran the module path
alone: 199).

## Deviations from plan

**1. [Decision] `ComposeTabControls` in place of the five-tuple.** The plan said "returned in its
tuple". The tuple would have become six with two `CheckBox` and two `SpinCtrl`, and the Reading
builder already gives the reason for a struct. Files: `wx_settings.rs`. Commit `35d868e3`.

**2. [Decision] The live look is a test.** The plan asked for a look at the running program by
hand; Bash and PowerShell in this harness see a stale profile and cannot read a menu's ticks, and
a look is made once where a test is made on every whole gate. The part of the look the test
cannot make, the application's own menu after a real header click, is ledger 487. Files:
`tests/one_sort_is_checked_on_a_live_menu.rs`. Commit `139aaccb`.

**3. [Decision] Task 1's break is a plant, not a reversal.** Above, and on the record itself.
Commit `35d868e3`.

**4. [Decision] Two ledger entries, not one.** 487 for the look and 488 for FOUND-06's listening
line, because they are two different people's answers: a look and a listening pass.

Everything else executed as written. No scripted edit touched a tracked file: exception set zero,
and it stayed there; commit messages were written to the scratchpad and passed with `-F`; `cargo
fmt` ran before each commit. Carriage returns measured with `tr -cd '\r' | wc -c` on every
changed file before each commit: zero on each. No em-dash in any file this plan wrote, measured
with `grep -c` for the byte sequence before each commit. `Cargo.toml` untouched; no package added.

## Threat register

T-09-12: `read_settings` still writes `cfg.copy_lines` from `w.copy_lines`, and
`test_every_setting_somebody_can_change_is_offered_by_a_screen` passed on the green, quoted above.
T-09-13: one group, held by the chain reading and proven on a real menu bar; the application's own
menu after a header click is ledger 487. T-09-14: the reading complains "appends 6 radio items"
when one is taken out, and its companion holds that. T-09-SC: no package added. No new surface
outside the register.

## Known stubs

None. `sort_then` is built by `build_reading_tab`, handed back in `ReadingTabControls`, and read
at `read_settings`; `copy_lines` is built by `build_compose_tab`, handed back in
`ComposeTabControls`, and read at `read_settings`; the Sort submenu is built into the View menu
and its seven ids are handled at `:5233` onward. The three new targets are reached by the gate,
two through the records' `suite` and the third on every whole gate.

## Not done here, on purpose

Whether the Reading tab reads as one group by ear is FOUND-06's `[S]` line, ledger 488, and the
tester's; the test reads the sibling chain and finds only Then by's own label between the two
sorts, which is structure present. Whether the application's own Sort submenu shows one tick after
a real header click is ledger 487, a look rather than a listening pass; a real menu bar built to
the same shape answers one tick on that path. "Then by" is not on the Sort submenu, as the plan
said. No `FOUND` requirement is ticked, on the phase's rule that the last plan reads each clause
by clause. Nothing pushed.

## Self-Check: PASSED

Files: `tests/the_sort_controls_sit_together.rs`, `tests/one_sort_is_checked.rs` and
`tests/one_sort_is_checked_on_a_live_menu.rs` exist; `grep -c 'append_separator'` over the
`sort_menu` chain is 0; `grep -n '"Then &by:"' src/presentation/wx_settings.rs` is `:1264`,
inside `build_reading_tab`; `grep -n '"Cc and Bcc &lines:"'` is `:1008`, inside
`build_compose_tab`; `guards/guards.toml` holds 841 records by the TOML reader. Commits
`80e32324`, `35d868e3`, `f67bbdaa`, `139aaccb` and `c928cae4` are in `git log --oneline` on
`main`.
