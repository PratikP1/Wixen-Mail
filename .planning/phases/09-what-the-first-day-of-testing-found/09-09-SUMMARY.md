---
phase: 09-what-the-first-day-of-testing-found
plan: 09
subsystem: the Settings dialog, the spell-checking service, the measurements page, a measurement harness, guards
tags: [settings, performance, measurement, harness, wxchoice, freeze, lazy-pages, spellcheck, nvda, guards, changelog]

requires:
  - phase: 08-every-number-the-project-quotes
    provides: "08-03: common::started and the harness pattern in tests/the_numbers_the_targets_ask_for.rs, the release binary against a throwaway profile under WIXEN_MAIL_DATA, one log line with milliseconds, medians over five; docs/development/measurements.md and the reading that refuses a row without its command, date, commit and conditions"
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-06: wx_settings::answer_the_arrows_on moving the tab selection through SetSelection, which is what raises the page-changed event a page is now built on; 09-08: the pattern of a red commit with stubs answering the tree as it is and the count check's remedy run after the green"
provides:
  - "common::started::settings_built_line, and the line the dialog writes at info just before show_modal, settings built in N ms, from an Instant taken at the top of the ID_SETTINGS arm"
  - "tests/the_settings_dialog_opens_in.rs: an ignored harness that times the three candidates alone, builds the real dialog five times, shows each later tab once, starts the release binary against a throwaway profile, opens Settings five times through its own menu command and reads the line back; six tests on every commit holding the two line shapes, the frozen build, the later pages and the spelling sentence"
  - "wx_settings: the dialog frozen from before its first page to after its layout; LaterPages, the six pages after General built the first time their tab is shown by the tab row's page-changed handler on a frozen panel; SettingsWidgets::compose, reading, permissions, calendar_and_pim, feedback and advanced; read_settings reading a page nobody showed from the settings it would have shown; PermissionsTabControls, CalendarPimTabControls and AdvancedTabControls in place of tuples"
  - "spellcheck::source_for_language, the source for_language would build decided the same way with nothing built; WindowsSpeller::can_check; hunspell_name_for and hunspell_files_for, one file search the loader and the naming share"
  - "Eleven rows on docs/development/measurements.md, five before and six after, with the section defining what they time; the changelog entry for #34; ledger 504 to 508"
  - "guards/guards.toml: nine records measured, two moved with the code and measured again, 39 named by the count check run and read, one corrected; 867 records, census 802 + 65"
affects: [09-10, the phase's closing read of FOUND-12; the tester, who says whether it feels immediate and whether the first visit of Reading is felt; ledger 505, the dictionary leak, and 506, what NVDA's hooks cost per control]

actuals:
  tokens: 36500
  tasks: 2
  commits: 9

tech-stack:
  added: []
  patterns:
    - "A performance plan's suspects are measured against the whole first; when they do not sum to it, the remainder is decomposed with scratch instrumentation that is reverted, and the fix shape is chosen from what that finds rather than from the plan"
    - "A harness for a UI cost a person feels reproduces the observers that person has attached: the release binary's own line with the screen reader running is the figure that counts, and the in-process figure is kept as the one a runner can take, with the row saying which is which"
    - "A dialog with many controls is frozen while it is built and thawed after its layout; a page that is not the first is a panel in the notebook from the start and its controls arrive on the page-changed event, frozen, laid out and thawed, so a setting is never off the screen and never seen empty"
    - "A sentence that describes a resource does not build the resource to describe it: the decision that would build it is asked on its own"

key-files:
  created:
    - tests/the_settings_dialog_opens_in.rs
  modified:
    - src/common/started.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_settings.rs
    - src/service/spellcheck/mod.rs
    - src/service/spellcheck/windows_speller.rs
    - tests/checkbox_labels.rs
    - tests/every_event_has_a_control.rs
    - tests/the_sort_controls_sit_together.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/development/measurements.md
    - docs/changelog.md

key-decisions:
  - "Neither of the plan's two fix shapes was built, on the plan's own rule that the rows decide: the three lists cost a millisecond each, so nothing is kept once per process; the pages cost the rest, and inside them a spell checker built for one sentence and an unfrozen typeface list, so the change is a frozen build, a source named without a checker, and pages built when their tab is first shown, the last being the shape the plan and FOUND-12 offered for a page cost"
  - "The pages after General are built when their tab is first shown rather than filled after the dialog shows, because a page built on the page-changed event is never seen empty and needs no 'loading' sentence on either channel; what it costs is a pause on the first visit of that tab, ledger 507"
  - "The harness drives the release binary itself, posting the Settings menu command to the main window of a process it started and closing the dialog with WM_CLOSE, rather than a person pressing Ctrl+, on a real profile of this machine: the number is repeatable, and no run touches the tester's profile, which the executor was told not to"
  - "The in-process build is timed on a hidden frame and the page says it understates what a person with a screen reader pays by about three times; the harness was not changed to show its frame, because the release binary's line is the figure that counts and the hidden figure is the one a runner with no screen reader can take"
  - "read_settings reads a page nobody showed from the configuration the dialog opened with, and builds nothing on OK: pressing OK is not the moment to pay for six pages nobody looked at"
  - "The spelling source is decided by the same three questions for_language asks, with WindowsSpeller::can_check asking the factory without creating a checker; the one divergence, a dictionary whose files exist and will not parse, is written on the function"

patterns-established:
  - "A guard record whose break the count check cannot see: the has_hunspell record's built-in break reddened the new spellcheck test as well, found only because the remedy ran over every record naming the file"

requirements-completed: []

coverage:
  - id: D1
    description: "The time from Ctrl+, to the dialog built is measured on this machine on the release binary, and each candidate cost on its own, with the rows on docs/development/measurements.md carrying the machine, the build, the commit and the boundary, before anything changed"
    requirement: FOUND-12
    verification:
      - kind: unit
        ref: "src/common/started.rs#test_the_settings_built_line_is_held_byte_for_byte"
        status: pass
      - kind: integration
        ref: "tests/the_settings_dialog_opens_in.rs#test_the_settings_built_line_the_library_writes_parses_back"
        status: pass
      - kind: integration
        ref: "tests/the_settings_dialog_opens_in.rs#test_a_figure_line_carries_its_unit_and_parses_back"
        status: pass
      - kind: other
        ref: "cargo test --release --test the_settings_dialog_opens_in -- --ignored --nocapture at d169df71, the five rows dated 2026-09-16"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit"
        status: pass
    human_judgment: false
  - id: D2
    description: "The cost the rows named is paid differently: the dialog is built frozen, the spelling sentence names its source without building a checker, and the six pages after General are built when their tab is first shown; the after rows sit beside the before rows; every setting is still on the settings screen"
    requirement: FOUND-12
    verification:
      - kind: unit
        ref: "src/service/spellcheck/mod.rs#test_the_source_named_without_a_checker_is_the_one_a_checker_built_would_report"
        status: pass
      - kind: integration
        ref: "tests/the_settings_dialog_opens_in.rs#test_pages_after_the_first_are_built_when_their_tab_is_first_shown_and_read_from_the_settings_when_never_shown"
        status: pass
      - kind: integration
        ref: "tests/the_settings_dialog_opens_in.rs#test_the_dialog_and_each_later_page_are_built_frozen"
        status: pass
      - kind: integration
        ref: "tests/the_settings_dialog_opens_in.rs#test_the_spelling_sentence_is_worded_without_building_a_checker"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_offered_by_a_screen"
        status: pass
      - kind: integration
        ref: "tests/every_event_has_a_control.rs#test_every_event_is_reachable_and_keeps_what_it_was_given"
        status: pass
      - kind: other
        ref: "cargo test --release --test the_settings_dialog_opens_in -- --ignored --nocapture at 2b697408, the six rows dated 2026-09-17"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether Settings feels immediate on the tester's machine, and whether the first visit of the Reading tab is felt"
    requirement: FOUND-12
    verification: []
    human_judgment: true
    rationale: "FOUND-12's [S] line and ledger 504 and 507. The line stops before the dialog is shown; the show, the focus landing and the screen reader's first announcement are after it, and nobody has heard the dialog open since the change"

duration: 3h17m
completed: 2026-09-17
status: complete
---

# Phase 9 Plan 09: Settings measured, then built frozen with its later pages built when shown Summary

**Settings took 2,206 ms to build in the release binary, the median of five opens with NVDA
running in this session, measured before anything changed and written on the page with its
boundary; it takes 397 ms now on the same boundary, 341 on another run. The three things the
issue suspected, the spelling languages, the installed typefaces and the sound schemes, cost a
millisecond each. The time was in the pages of controls, and inside them in two things nobody had
named: a Windows spell checker built to word one sentence and released, 210 ms, and the typeface
list resizing itself after each of its 272 names under a shown window, about a second. The dialog
is frozen while it is built, the sentence is worded without a checker, and only General is built
before the show, each other page arriving the first time its tab is reached, on a frozen panel,
with a page nobody reached written back as it was stored. Every tab is in the row from the start
and every setting is where it was. The dialog writes `settings built in N ms` to the log on every
open, and a harness reads it off the release binary against a throwaway profile, sending nothing
to a window it did not start.** #34 closed on the tree with the merge commit; whether it feels
immediate is the tester's. Nothing pushed.

## Performance

- **Duration:** 3 h 17 min from the branch to the merge, of which 52 min were the count check's
  remedy over 39 records, about 20 min the six scratch runs that decomposed the build, 13 min the
  two whole gates (378 s on the branch, and `main`'s hook at the merge), and about 12 min the
  five harness runs that took numbers
- **Started:** 2026-09-17T02:58Z (first commit 03:13:47Z)
- **Merged:** 2026-09-17T06:12:04Z at `a8b26596`
- **Tasks:** 2
- **Files modified:** 13 (1 created)

## The numbers

Every line the harness printed, at the two commits the rows name. Before, at `d169df71`
(`cargo test --release --test the_settings_dialog_opens_in -- --ignored --nocapture`, 2026-09-16):

```
== the Settings dialog: 1.0.0-alpha.1 at d169df71 on 2026-09-16
available_languages: first 1 ms, median 0 ms, the 5: 1, 0, 0, 0, 0 ms
installed_families: first 1 ms, median 1 ms, the 5: 1, 1, 1, 1, 1 ms
SoundScheme::discover: first 0 ms, median 0 ms, the 5: 0, 0, 0, 0, 0 ms
the machine offers 19 spelling languages and 272 typeface families; the schemes folder is empty
build_settings_dialog: first 679 ms, median 663 ms, the 5: 679, 643, 663, 684, 658 ms
settings built in, the release binary: first 2206 ms, median 2206 ms, the 5: 2206, 2384, 2182, 2149, 2332 ms
```

After, at `2b697408`, the same command, 2026-09-17, the quiet run of two at that commit:

```
== the Settings dialog: 1.0.0-alpha.1 at 2b697408 on 2026-09-17
available_languages: first 1 ms, median 0 ms, the 5: 1, 0, 0, 0, 0 ms
installed_families: first 1 ms, median 1 ms, the 5: 1, 1, 1, 1, 1 ms
SoundScheme::discover: first 0 ms, median 0 ms, the 5: 0, 0, 0, 0, 0 ms
the machine offers 19 spelling languages and 272 typeface families; the schemes folder is empty
build_settings_dialog: first 169 ms, median 160 ms, the 5: 169, 160, 161, 149, 143 ms
first visit of each later tab: Compose 32 ms, Reading 156 ms, Permissions 8 ms, Calendar & PIM 82 ms, Feedback 49 ms, Advanced 19 ms
settings built in, the release binary: first 397 ms, median 397 ms, the 5: 397, 375, 411, 439, 387 ms
```

The other run at `2b697408`, right after the test binary compiled: the build 201, 181, 179, 154
and 161 ms, median 179; the first visits 23, 132, 8, 80, 43 and 28 ms; the release binary 426,
431, 405, 388 and 350 ms, median 405. A run at `8162ef57`, the commit that made the change: the
build 140, 138, 133, 135 and 134 ms, median 135; the release binary 310, 341, 384, 341 and 366
ms, median 341. The spread between runs of one build is about a fifth, wider than the before rows
showed, and all three are in the after rows' conditions so the page does not show the flattering
one. The machine, the build, `tasklist` quiet before each run and NVDA running are in every row.

The line from the release binary's own log, the first open of the after run, as
`parse_the_settings_built_line` read it: `settings built in 397 ms`. Before: `settings built in
2206 ms`.

**Which cost it was.** The rows said the three candidates were nothing and the build was
everything, so each page was timed with `eprintln!` lines in `build_settings_dialog` and the
General tab, in the release test process, six runs, the file put back with `git checkout` after
each. Hidden frame, one representative run: dialog, notebook and General 401 ms; Compose 29;
Reading 138; Permissions 8; Calendar & PIM 76; Feedback 41; Advanced 22; buttons and sizer 1;
whole 716. Inside General: the typeface `Choice` with 272 names 84 ms, the theme choice 17, the
language `Choice` with 19 names 35, `for_language` 7, and **dropping the speller 210 ms**, five
runs between 204 and 215. The shown-frame run (`frame.show(true)` in the harness, reverted): whole
2,093 ms median, within three percent of the release binary's 2,153 that run; the typeface
`Choice` 1,090 ms, every other control about twice its hidden cost. With `dlg.freeze()` and the
frame shown: the typeface `Choice` 143 ms, whole 1,215, the release binary 1,250. The logs are in
this session's scratchpad and nowhere in the tree; the page carries only what the harness prints.

## What landed

| Where | Before | Now |
|---|---|---|
| `wx_app.rs`, the `ID_SETTINGS` arm | the accounts, then `handle_settings` | an `Instant` first, handed through `handle_settings` to `show_settings_dialog`; the scan target's call hands a fresh one |
| `wx_settings::show_settings_dialog` | build, `show_modal` | build, `tracing::info!` of `started::settings_built_line(asked_at.elapsed())`, `show_modal`; the working-day refusal asked only of a Calendar & PIM page that was built |
| `build_settings_dialog` | seven pages built in full before the dialog is shown | `dlg.freeze()` first; General built; six empty panels added to the notebook in order; `LaterPages` behind an `Rc`, shared with the page-changed handler; `dlg.thaw()` after `set_sizer` |
| `LaterPages::build_the_page_for` | nothing | the panel frozen, the page built into it once through its `OnceCell`, laid out, thawed; `compose()` and the five others build on demand for `read_settings`'s callers and the tests |
| `read_settings` | every field read from `w` in one function | General read in place; each later page through `if let Some(page) = w.later.<page>.if_built()` into `read_the_<page>_page`, so a page nobody showed leaves `base`'s values alone; `read_the_permissions_page` keeps the `reading: w.` spelling the config guard reads |
| `add_language_and_spelling` | `for_language(&config.language).source().describe()` | `source_for_language(&config.language).describe()` |
| `spellcheck::source_for_language` | none | Windows if `can_check` says so for the resolved or the stored tag, Hunspell if `hunspell_files_for` finds the pair, else the built-in list |
| `WindowsSpeller::can_check`, `a_factory_that_checks` | `for_language` created the factory, asked `IsSupported` and created the checker | the first two on their own, shared by both |
| `try_load_spellbook` | its own path search | the loop over `hunspell_files_for`, which `source_for_language` asks as well |
| `SettingsWidgets` | fifty-odd public and private fields across seven pages | General's controls and `later`; six accessors; `ReadingTabControls`, `CalendarPimTabControls` and `AdvancedTabControls` public with the fields the tests read |
| `tests/the_settings_dialog_opens_in.rs` | none | the harness, 855 lines: definitions, the profile, the release binary driven through `EnumWindows`, `GetMenu` and `PostMessageW`, six tests on every commit |
| `docs/development/measurements.md` | no Settings row | a section defining the four figures, five before rows, six after rows |
| `docs/changelog.md` | no entry | under `[Unreleased]`, Fixed, with the before and after figures dated and Known limitations |

## Task commits

| Commit | What |
|---|---|
| `b288e9f0` | test(09-09): the red half of task 1, three named and the count check; the library function a stub, the figure line without its unit |
| `d169df71` | feat(09-09): the wording, the `Instant` through the arm, the line before `show_modal`, the harness, three records, three re-measured |
| `38448539` | docs(09-09): the before rows and the section, from the run at `d169df71` |
| `8dfde86d` | test(09-09): the red half of task 2, four named and the count check; `source_for_language` a stub answering the built-in list |
| `8162ef57` | feat(09-09): the frozen build, the later pages, the source without a checker, the five targets moved to the accessors, five records, two moved, the remedy over 33 |
| `292be290` | test(09-09): the red half of the first-visits line, one named and the count check |
| `2b697408` | feat(09-09): the units, one record, six re-measured |
| `46f1f03c` | docs(09-09): the after rows and the changelog entry |
| `a8b26596` | Merge 09-09 into `main` |

Branch `settings-is-measured-then-opens-at-once` from `main` at `a2723385`. Not pushed; 84
commits unpushed before the commit that lands this summary.

## Honest RED and GREEN

Task 1's red named three and the count check. Against a `settings_built_line` answering an empty
string and a figure line with no unit, all three were red for their own reasons: the wording
test found `""`, the parse-back test found no line in a log holding `""`, and the unit test found
`first 50, median 30`. The count check fired on `started.rs`, 6 to 7, naming its three records.
The gate in `red` mode ran exactly the four.

Task 2's red named four and the count check. Against a `source_for_language` answering the
built-in list, the spellcheck test found Builtin where a built checker reported Windows for
`en-US`; the Reading page had controls before its tab was shown; `build_settings_dialog` had no
`dlg.freeze()`; and `add_language_and_spelling` still named `spellcheck::for_language(`. The count
check fired on `spellcheck/mod.rs`, 63 to 64, and on the harness target, 3 to 6. The red test for
the pages used the API of the day, `widgets.sort_order`, and the green moved that one line to
`widgets.reading().sort_order` when the field became an accessor; its assertions did not change.
The first-visits line had a red of its own, `292be290`, against the line without units.

Two things happened in green that the red did not name. The config guard
`test_whether_message_text_may_be_fetched_is_offered_by_a_screen` reads the literal `reading: w.`
in the settings source, and the first green spelt the page variable `page`; the permissions read
became `read_the_permissions_page(w: &PermissionsTabControls, ...)`, which is what the other four
page readers are called anyway. And the `has_hunspell` record came out short in the remedy, said
under Guard records.

## What the gate selected

| File | On the branch |
|---|---|
| `src/common/started.rs` | `--lib common::started`, and the two coupled targets `the_numbers_the_targets_ask_for` and `the_settings_dialog_opens_in` |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` on task 1's green, with the eleven coupled targets |
| `src/presentation/wx_settings.rs` | `--lib presentation::wx_settings`, the filter that matches nothing, and the six coupled targets: `every_event_has_a_control`, `checkbox_labels`, `the_settings_dialog_opens_in`, `the_language_the_screen_shows_is_the_one_used`, `the_sort_controls_sit_together`, `the_settings_tab_row_says_each_tab_once` |
| `src/service/spellcheck/mod.rs`, `windows_speller.rs` | `--lib service::spellcheck` and `--lib service::spellcheck::windows_speller` |
| `tests/the_settings_dialog_opens_in.rs` | `--test the_settings_dialog_opens_in` on every commit that touched it |
| `tests/checkbox_labels.rs`, `every_event_has_a_control.rs`, `the_sort_controls_sit_together.rs`, `theme_reach.rs` | each its own target on task 2's green |
| `guards/guards.toml`, `docs/development/measurements.md`, `docs/changelog.md` | no scoped target; the two docs commits answered `docs_only` and ran the document-reading targets, 75 in `house_style` and the measurements reading among them |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `46f1f03c`, output to a file and the exit status read
directly, never piped: exited 0 in 378 s on its first run, 7,870 passed and none failed over 67
result lines, the release build included; 8 more than 09-08's 7,862, which is this plan's 1, 3, 1
and 3. `main`'s hook ran `all` again on the merge: 7,870 and none failed. Ledger 374's keyring
race did not appear on either run.

## Guard records

858 records by the TOML reader before, 867 after; census 802 + 56 before, 802 + 65 after, the
line at `guards/guards.toml:84` moved in each green commit. Nine new, two moved with the code, one
older corrected, all measured through `scripts/guards.sh --remeasure`, `WIXEN_TEST_THREADS`
untouched.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| the settings built line keeps the shape the harness parses | `started.rs`, library | "built in" to "built after" | 1 | rebuild 29 s, run 48 s |
| the harness can still parse the settings built line the application writes | `started.rs`, `the_settings_dialog_opens_in` | the same | 1 | 11 s, 1 s |
| a figure line the settings harness prints carries its unit | the target | the units dropped | 1 | 12 s, 1 s |
| the spelling source named without a checker is the one a checker would report | `spellcheck/mod.rs`, library | Builtin where Windows checks | 1 | in the 39 |
| the spelling sentence on the settings screen is worded without building a checker | `wx_settings.rs`, the target | the checker put back | 1 | in the 39 |
| the settings dialog is frozen while its pages are built | `wx_settings.rs`, the target | `dlg.freeze()` taken out | 1 | in the 39 |
| a settings page after the first is built when its tab is first shown and not before | `wx_settings.rs`, the target | every page built in the open | 1 | in the 39 |
| pressing OK keeps the stored answer for a settings page nobody showed | `wx_settings.rs`, the target | `read_settings` starting from the defaults | 1 | in the 39 |
| the first-visits line the settings harness prints carries its units | the target | the units dropped | 1 | 12 s, 1 s |

Moved with the code and measured again: "pressing OK writes the answers the panel holds rather
than rebuilding from the stored value", its break now inside the Feedback page's `if let`; and
"try_load_spellbook really finds a dictionary sitting on its path, not a stub answering None", its
break now the loop over `hunspell_files_for`. Both exact.

**What the remedy found.** The count check fired three times: task 1's red on `started.rs` (3
records, 5 min, all exact), task 2's red on `spellcheck/mod.rs` and the harness target (33
records), and the first-visits red on the target (6 records, 2 min, all exact). The 33 were run
with the five new records and the moved Feedback record, 39 in all, 52 min, rebuild 1,374 s and
run 1,760 s over the 39 by the `timed:` lines. Thirty-eight were exactly what they said. One was
not: "has_hunspell says no for the plain built-in checker, not always yes", whose break makes a
built-in checker claim Hunspell data, which the new spellcheck test sees as the two answers
disagreeing. Corrected by hand with the second name and a comment, measured again on its own: both
named went red and nothing else. Counts written: `started.rs` 7, `spellcheck/mod.rs` 64, the
harness target 7, `wx_settings.rs` 0.

## Premises the tree contradicted

1. **The three candidates cost nothing, and the plan's two fix shapes were for costs that were not
   there.** `available_languages()` 1 ms and 19 languages, `installed_families()` 1 ms and 272
   families, `SoundScheme::discover` 0 ms; so no `OnceLock` behind any of them, and the changelog
   has no line about a font installed while the program runs, because nothing is remembered. The
   pages cost the rest, which is the branch the plan and FOUND-12 wrote as "built when its tab is
   first shown"; and inside the pages, two costs the issue did not list, the released spell
   checker and the unfrozen typeface list, which the scratch decomposition found and the summary
   above gives. Observation 619.
2. **The test process understates the running program by three times, and it is the window being
   shown.** The plan's harness built its frame and never showed it. A scratch run that showed it
   matched the release binary within three percent; NVDA's in-process hooks attach to a process
   with a window on screen and watch every control's creation, and this session had NVDA running,
   as the tester's machine does. The row says so and says the mechanism is inferred, not
   diagnosed, because stopping his NVDA was not asked for. Observation 620.
3. **The line cannot be written in `handle_settings`.** `show_modal` is called inside
   `show_settings_dialog`, so the line is written there from an `Instant` `handle_settings` is
   handed; the boundary is the plan's.
4. **`for_language` was reached by the settings screen for a sentence.** `add_language_and_spelling`
   built a checker to word "Spelling is checked by Windows" and dropped it, and the drop cost 210
   ms of every open; the issue listed the language *list* and not this.
5. **`wxChoice` resizes itself after every item unless frozen**, wx 3.3.2 `src/msw/choice.cpp:272`,
   and a child added under a frozen window is frozen with it, `src/common/wincmn.cpp:1305`; read
   from the source `wxdragon-sys` unpacks under `target/release/wxWidgets`. The freeze is why the
   typeface list went from 1,090 ms to 143 with the window shown.
6. **A source-reading guard reads a variable name.** `config.rs`'s
   `test_whether_message_text_may_be_fetched_is_offered_by_a_screen` looks for `reading: w.` in
   the settings source, so the permissions page reader is a function whose parameter is `w`, as
   the other page readers are.
7. **`wx_settings.rs` had 8 records at the plan's count and 8 name it still**, none re-measured
   by the count check because the file holds no tests; two of its records moved with the code
   and were measured by hand. The harness target went from 0 records to 8 naming it.

Premises 1 (seven pages, the three sites), 2 (the 08-03 instrument's shape), 3 (what can be
timed without a window) and 5 (the record counts, `started.rs` 3 and 6, `wx_app.rs` 50) of the
plan held; `wx_app.rs` is named by 56 records now, as 09-08 left it.

## Deviations from plan

**1. [Decision] Neither of the plan's two fix shapes; three changes the decomposition justified.**
Premise 1 above, and the key decision. Ledger 508.

**2. [Decision] The release binary is driven by the harness against a throwaway profile, not by a
person on a real profile.** The executor was told not to touch the tester's profile and not to
send input to anything it did not start; the harness starts the binary, finds its main window by
process id, walks its menu bar for the item beginning `&Settings`, posts `WM_COMMAND`, and closes
the dialog with `WM_CLOSE`. The plan's "press Ctrl+, once" is the same arm. Ledger 508.

**3. [Extension] The dialog timed five times, and a sixth build timing each tab's first visit.**
The plan said once; five builds in one window give a median, and the first-visit line is where
the open's cost went, which the page would otherwise not show. The extra line had its own red.
Ledger 508.

**4. [Rule 1] The `has_hunspell` record came out short in the remedy** and was corrected by hand
and measured again; the case `CLAUDE.md` gives for a test added near a rule.

**5. [Process] No tracked file was edited by a script.** Every edit to a tracked file went through
Read then Edit or Write, the harness's `replace_all` on four field names in one test file
included; `cargo fmt` ran before each commit and never while a guard or the harness ran; the six
scratch instrumentations were Edit calls put back with `git checkout` on the one file each; carriage
returns measured with `tr -cd '\r' | wc -c` on every changed file before each commit and on the
ledger after it, zero on each; no em-dash in any file this plan wrote, measured with `grep -c`
for the byte sequence. The exception set for scripted edits on a tracked file is zero, as 09-08
left it. Commit messages were written to the scratchpad and passed with `-F`; the two whole-gate
runs, the merge, every remedy and every harness run wrote to a file and the exit status was read
directly. `git commit` and `git merge`, not `gsd-tools query commit`. Nothing was sent to any
window this plan did not start; the tester's profile was not read; `Cargo.toml` untouched, no
package or feature added; `WIXEN_TEST_THREADS` at its default.

Everything else executed as written.

## Threat register

T-09-30: task 2's shape was chosen after task 1's rows and the decomposition, and the summary
names the row and the scratch figure for each of the three candidates and each of the three
changes. T-09-31: the settings guard in `config.rs` ran on every commit that touched the screen
and stays green; the test that stores a sort order and never shows Reading holds `read_settings`
to the stored value, and its record breaks `read_settings` into starting from the defaults.
T-09-32: accepted and moot, nothing is kept between opens, so no list can go stale. T-09-SC: no
package added. New surface outside the register: the harness posts two window messages to a
process it started, by process id, and its module header says why never by title.

## Ledger

`.planning/WINDOWS.md` 504 to 508 written through `gsd-tools windows append`, both halves, no
backslash in any description (the nine in the file predate this plan), and
`the_planning_files_agree_with_themselves` green after, 16 passed. 503 before, 508 after; 475
open before, 480 after.

| id | kind | what |
|---|---|---|
| 504 | unrun-verify | whether Settings feels immediate on the tester's machine; the line stops before the show; FOUND-12's `[S]` line |
| 505 | todo | `try_load_spellbook` leaks both halves of every dictionary it loads, on every checker built; pre-existing, found while reading |
| 506 | todo | under a shown window with NVDA every control costs about twice and a Choice 10 to 60 ms; the mechanism inferred from the scratch run and not diagnosed further |
| 507 | todo | the first visit of Reading pays its build, 156 ms hidden and about twice under NVDA; whether it is felt, and whether filling after the show would be better, is the tester's |
| 508 | deviation | the four departures above |

## Known stubs

None. `settings_built_line` is called by `show_settings_dialog`; `source_for_language` by
`add_language_and_spelling`; `can_check` by `windows_checks`; `hunspell_files_for` by
`try_load_spellbook` and `source_for_language`; `build_the_page_for` by the page-changed handler;
the six accessors by `read_settings`'s callers through `if_built`, by `show_settings_dialog`'s
working-day check, and by the five test targets. `THE_TABS_AFTER_GENERAL` in the harness names
the tabs in the dialog's order and would mislabel a row if that order changed; the harness's
`THE_READING_PAGE` is the same coupling, and both are said in their comments.

## Not done here, on purpose

Hearing the dialog open (504) and the first visit of Reading (507) are the tester's. The
dictionary leak (505) and what NVDA's hooks cost per control (506) are todos. No `FOUND`
requirement is ticked, on the phase's rule that the last plan reads each clause by clause; the
closing read should count FOUND-12's first `[D]` line as met by the five before rows and the
section, its second by the six after rows, the three page tests, the frozen-build test and the
settings guard, and its `[S]` line as the tester's. Nothing pushed.

## Self-Check: PASSED

`tests/the_settings_dialog_opens_in.rs` exists; `grep -c 'settings built in' src/common/started.rs`
answers 2; `grep -c 'dlg.freeze()' src/presentation/wx_settings.rs` answers 1;
`grep -c 'source_for_language' src/presentation/wx_settings.rs` answers 1 and
`grep -c 'spellcheck::for_language(' src/presentation/wx_settings.rs` answers 0;
`guards/guards.toml` holds 867 records by the TOML reader; `docs/development/measurements.md`
holds eleven rows beginning `| Settings`; `docs/changelog.md` holds the line beginning
"**Settings opens in under half a second instead of over two.**". Commits `b288e9f0`, `d169df71`,
`38448539`, `8dfde86d`, `8162ef57`, `292be290`, `2b697408`, `46f1f03c` and `a8b26596` are in
`git log --oneline` on `main`.
