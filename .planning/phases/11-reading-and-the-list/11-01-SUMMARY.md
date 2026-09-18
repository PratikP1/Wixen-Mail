---
phase: 11-reading-and-the-list
plan: 01
subsystem: settings, spellcheck, guards, CI
tags: [spelling-language, settings-screen, regression, ci, guards, changelog]

requires:
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-02: spellcheck::language_to_use, one resolver for a stored tag, and the settings screen's language_rows_and_selection asking it, with tests/the_language_the_screen_shows_is_the_one_used coupled to wx_settings.rs by a guard record"
provides:
  - "presentation::which_language_row: RowToShow, Existing(usize) or Added(String), and which_row_shows(stored, this_machine, rows), pure over the stored tag, the machine's tag and the offered rows; a tag with a region kept as chosen, a bare tag resolved as the checker resolves it with its own listed row as the fallback"
  - "wx_settings::language_rows_and_selection asking that rule, the three-line expression that asked the resolver first gone, the screen's import of language_to_use gone"
  - "guards/guards.toml: one record on the new module measured on the library, the screen's coupling record rewritten onto the new call and measured on its target; 913 records, census 802 + 111"
  - "docs/changelog.md: the entry under Unreleased, Fixed, naming the build, the regression's origin in 09-02's fix for #21, and that CI found it"
  - "ledger 530: the runner's en-AU case, unrun here"
affects: [11-02 onward, which merge onto a main whose next push should show CI green; 11-12, which reads FOUND-17's CI clause when the push has run]

actuals:
  tokens: 4552
  tasks: 2
  commits: 4

tech-stack:
  added: []
  patterns:
    - "Two readers of one stored value answer two questions, and when the questions differ the answers may: the checker chooses a dictionary and may settle for the nearest, the screen shows what was chosen and must not; one resolver serving both was the regression"
    - "A CI-only failure is the confirmation and not the red: the red is a unit case over hand-built rows that fails on both machines, the environment-dependent target stays unchanged as the runner's evidence, and the commit says which machine shows which"

key-files:
  created:
    - src/presentation/which_language_row.rs
  modified:
    - src/presentation/mod.rs
    - src/presentation/wx_settings.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "A bare tag the checker cannot place keeps its own listed row as the fallback, as 09-02 established and the code held, rather than the plan's behaviour block's Added: taken literally the block would have added a duplicate row for a bare tag the machine lists without a dictionary; a sixth case holds the fallback"
  - "The rule lives in presentation::which_language_row at zero records, as the plan said, so its six tests flag nothing; wx_settings.rs gains a call and no test, and its nineteen records stay at 0"
  - "The screen record's break takes row 0 for whatever is stored, the screen as it stood before 2026-09-16, rather than the plan's Existing(0) for the answer alone in prose; the same one test reddens and the old break's meaning is kept"
  - "One ledger entry, 530, for the runner's case: the plan's files list did not name the ledger, but the phase README asks each plan to record what it could not settle, and a verify this machine cannot run is that"

patterns-established:
  - "A plan that corrects an earlier plan restates that plan's rule, and the restatement drops a clause; the earlier summary and the code are the source to diff the behaviour block against before the first test is written (observation 668)"

requirements-completed: [FOUND-17]

coverage:
  - id: D1
    description: "which_row_shows answers a tag with a region as its own row, available or not, or a row added at the end; a bare tag as the checker resolves it, its own listed row as the fallback; a tag nothing offers as added; case ignored"
    requirement: FOUND-17
    verification:
      - kind: unit
        ref: "src/presentation/which_language_row.rs#test_a_tag_with_a_region_is_shown_as_its_own_row_even_without_a_dictionary"
        status: pass
      - kind: unit
        ref: "src/presentation/which_language_row.rs#test_a_tag_with_a_region_the_machine_does_not_list_is_added_as_stored"
        status: pass
      - kind: unit
        ref: "src/presentation/which_language_row.rs#test_a_tag_with_a_region_finds_its_row_whatever_its_case"
        status: pass
      - kind: unit
        ref: "src/presentation/which_language_row.rs#test_a_bare_tag_is_resolved_to_the_row_the_checker_would_use"
        status: pass
      - kind: unit
        ref: "src/presentation/which_language_row.rs#test_a_bare_tag_the_checker_cannot_place_stands_as_its_own_row_when_listed"
        status: pass
      - kind: unit
        ref: "src/presentation/which_language_row.rs#test_a_tag_nothing_offers_is_added_as_stored"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a spelling language chosen for a region is shown as chosen, not as the checker's answer', measured 2026-09-18 on the library: all 2 tests named went red, and nothing else did"
        status: pass
    human_judgment: false
  - id: D2
    description: "language_rows_and_selection asks the rule and read_settings rebuilds the same list, so the row shown and the tag written are one row; the integration reading is unchanged and passes here"
    requirement: FOUND-17
    verification:
      - kind: integration
        ref: "tests/the_language_the_screen_shows_is_the_one_used.rs#test_the_language_the_screen_shows_is_the_one_the_checker_uses"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the settings screen shows a stored language nothing offers as itself, not as row 0', rewritten onto the new call and measured 2026-09-18 on its target: the one test named went red, and nothing else did"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether GitHub's runner, which offers no en-AU, now keeps en-AU in the integration reading, and so whether CI's Test Suite job is green on main"
    requirement: FOUND-17
    verification: []
    human_judgment: true
    rationale: "The next push of main is Pratik's; the case cannot be run red or green here because this machine offers the language; ledger 530"

duration: 30min
completed: 2026-09-18
status: complete
---

# Phase 11 Plan 01: A chosen spelling language is kept as chosen where this machine cannot check it Summary

**The Settings screen shows a stored en-AU as en-AU on a machine that offers only en-US, and
OK writes it back, where before it showed and wrote the checker's nearest dictionary and the
choice was gone the first time Settings was saved. The rule is `presentation::which_language_row`,
in two halves: a tag with a region is kept as chosen, its own row if the machine lists one and a
row added at the end if not; a bare tag is resolved as the checker resolves it, which is #21's
fix, with its own listed row as the fallback 09-02 established. Six cases, four red on this
machine before the rule; the integration target that failed on GitHub's runner is unchanged and
passes here; whether it passes there is the next push of `main`, which is Pratik's.** Nothing
pushed.

## Performance

- **Duration:** 30 min from the branch at 12:36:26Z to the merge at 13:04:06Z, of which about
  1 min 30 s was guard measurement across two runs (75 s and 13 s by the runners' `timed:`
  lines), about 5 min 20 s the three hook runs (121 s, 79 s, 121 s), and about 9 min 30 s the
  two whole gates (295 s and 273 s); the summary and the planning files after
- **Started:** 2026-09-18T12:36:26Z (the reading; the branch at 12:39Z, the first commit
  12:41:54Z)
- **Merged:** 2026-09-18T13:04:06Z at `316ea755`
- **Tasks:** 2
- **Files modified:** 6 (1 created), the ledger among them

## What landed

**Task 1, the rule.** `src/presentation/which_language_row.rs`. `RowToShow` is `Existing(usize)`
or `Added(String)`. `which_row_shows(stored, this_machine, rows)` finds the stored tag's own row
by `eq_ignore_ascii_case`; for a tag that names a region (`names_a_region`: a `-` with something
after it, so `en-AU` does and `en` and `en-` do not) that row is the answer; for a bare tag the
answer is `language_to_use`'s row, else the tag's own row; `None` either way is `Added(stored)`.
The module comment says why the halves differ: the checker asks which dictionary to open and may
settle for the nearest, the screen asks which row to show and what it shows is what OK writes
back, so a screen showing the checker's answer rewrote the choice. Registered in
`presentation/mod.rs` beside `what_the_scans_can_judge`. `cargo test --lib
presentation::which_language_row::` passes with 6.

The six cases, over hand-built rows: en-AU over [en-US available, en-AU not available] on an
en-US machine is `Existing(1)`, which is the runner's shape when Windows lists the language
without a dictionary; en-AU over [en-US] alone is `Added("en-AU")`, the runner's shape when it
does not list it; `EN-au` finds `en-AU`; a bare `en` over [en-029, en-US] on an en-US machine is
`Existing(1)`, #21; a bare `fr` over [en-US, fr not available] is `Existing(1)`, the fallback;
`zz-ZZ` and `zz` over [en-US] are `Added`.

**Task 2, the screen.** `language_rows_and_selection` at `wx_settings.rs:888-901` builds the rows
from `available_languages()`, asks `which_row_shows(stored, system_language().as_deref(), &rows)`,
and for `Added(tag)` pushes the row with `available: false` and selects it, as it did before; its
doc comment names the rule's module, the runner's case and #21. The three-line `selected`
expression that asked the resolver and fell back to `row_of(stored)` is gone, and with it the
screen's import of `language_to_use` (`grep -c 'language_to_use' src/presentation/wx_settings.rs`
answers 0; `grep -c 'which_row_shows(' src/presentation/wx_settings.rs` answers 1).
`read_settings` at `:3272-3279` rebuilds the same list through the call at `:3275` and writes
`languages[idx].tag`, unchanged. `tests/the_language_the_screen_shows_is_the_one_used.rs` is unchanged and passes here
(`test result: ok. 1 passed`), as do the eight other targets coupled to `wx_settings.rs`:
`every_event_has_a_control`, `checkbox_labels`, `the_settings_dialog_opens_in`,
`the_sort_controls_sit_together`, `the_settings_tab_row_says_each_tab_once`,
`every_settings_checkbox_reads_as_a_checkbox_after_its_page_is_built`,
`a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control` and
`how_much_message_text_stays_is_read_back_from_the_permissions_page`, which is the list `bash
scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_settings.rs` prints, nine.

**The changelog.** One entry at the top of `[Unreleased]`, Fixed, beginning "**A spelling language
you chose for a region this computer has no dictionary for was rewritten to this computer's own
region the first time Settings was saved.**": found by CI on 2026-09-18 in
`1.0.0-alpha.1+149.g744d05ef` and not by anybody, because the machine the testing happens on
offers the language and GitHub's does not; what a person would have seen; the cause as the fix
for #21 in 09-02 putting one resolver in front of both readers; what is kept now and that checking
still happens in the nearest dictionary; that a bare tag still resolves to this computer's region;
Known limitations: that the row then says "(no dictionary installed)" has been read from the code
(`wx_settings.rs:940`) and not heard. The version stays `1.0.0-alpha.1`: no build has been cut
since it was set.

## Why the runner's case is green now, reasoned through

The runner's machine is set to en-US and, by what the run showed, offers en-US and not an
available en-AU; `choices_from` marks every Windows-listed tag available, so either the runner's
Windows does not list en-AU at all or lists it without a dictionary. At `744d05ef`,
`language_to_use("en-AU", Some("en-US"), rows)` found no usable exact match, found the machine's
own region in the family and usable, and answered `en-US`; `row_of("en-US")` was a row; the
`or_else` that would have kept en-AU was never reached; the screen selected en-US and
`read_settings` wrote it back, which is the run's line: `stored "en-AU": the screen would keep
"en-US", the checker uses "en-AU"`.

At `316ea755`, `names_a_region("en-AU")` is true, so the resolver is never asked. If the runner
lists en-AU, `row_of` finds it and the screen selects that row, whose tag is `en-AU`
(`test_a_tag_with_a_region_is_shown_as_its_own_row_even_without_a_dictionary`). If it does not,
the answer is `Added("en-AU")`, a row with that tag is pushed at the end and selected
(`test_a_tag_with_a_region_the_machine_does_not_list_is_added_as_stored`). Either way
`read_settings` reads `languages[idx].tag`, which is `en-AU`, and the case's `kept != "en-AU"`
is false. The other two cases are as they were: the bare `en` goes through the resolver as before
and the test computes its expectation through the same resolver; `zz-ZZ` names a region, no row
spells it, and it is added, which is what the old `or_else` did too. On the runner both passed at
`744d05ef` and nothing in their path changed. So the target is written to pass on a machine that
offers en-AU (here, measured) and on one that does not (the runner, reasoned); the run is what
settles the second, ledger 530.

## Task commits

| Commit | What |
|---|---|
| `7c8ebda9` | test(11-01): the red half of task 1, four named by module path, two green on arrival and said |
| `29e4d850` | feat(11-01): the rule in two halves, its record measured, the census line at 111 |
| `016b1cfd` | feat(11-01): the screen asking the rule, the coupling record rewritten and measured, the changelog entry; no red precedes it and the message says why |
| `316ea755` | Merge 11-01 into `main` |

Branch `a-chosen-language-kept-as-chosen` from `main` at `d42b4aed`. Not pushed; 6 commits ahead
of `origin/main` before the commit that lands this summary, by `git rev-list --count
origin/main..main`.

## Honest RED and GREEN

Task 1's red named four by module path, from cargo's own lines: the region tag shown as its own
row without a dictionary, the region tag found whatever its case, the bare tag resolved to the
checker's row, and the bare tag standing as its own listed row. Two were green on arrival and the
commit says so: the region tag the machine does not list, and the tag nothing offers, because the
stub answered `Added(stored)` for everything and those two cases want `Added`. They stay to hold
the rule to that. The gate in `red` mode ran exactly those four red and nothing else, in 121 s,
having run the whole presentation layer once because `mod.rs` changed. The green ran the module's
six and the whole-tree guards in 79 s, no marker, no remedy printed: the only file that gained
tests is the new module, whose one record was measured before the commit. The count check was not
named in the red, because no file a record names gained a test at that commit.

**Task 2 has no red commit, and this is the place that says so.** The plan's red for the screen
is the runner's failure at `744d05ef`, quoted above; the target passes here at the parent
commit, because this machine offers en-AU, and `scripts/red-commit.sh` refuses a red commit
naming a test that passes. So task 2 is one commit, `016b1cfd`, whose message says the runner's
line is its red and `7c8ebda9` is the rule's. Under the TDD gate's own terms, the plan has a
`test(11-01)` commit and two `feat(11-01)` commits after it, and the screen change is glue over
the tested rule; the summary records it here rather than claiming a red that could not be made.
The hook ran `presentation::wx_settings` (matching nothing) and the nine coupled targets in 121 s.

## What the gate selected

| File | On the branch |
|---|---|
| `src/presentation/which_language_row.rs` | `--lib presentation::which_language_row` on both task 1 commits |
| `src/presentation/mod.rs` | `--lib presentation` whole, on task 1's red |
| `src/presentation/wx_settings.rs` | `--lib presentation::wx_settings` (matching nothing) and the nine coupled targets on task 2's commit |
| `guards/guards.toml`, `docs/changelog.md` | no scoped target; the whole-tree guards on every commit |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `016b1cfd`, its output to a file with the exit status
appended by the same shell, never piped: exit 0, 8,038 passed and none failed over 74 result
lines, 295 s from 12:54:15Z to 12:59:10Z, the release build included; its last block says five of
CI's seven jobs. Six more than 10-07's 8,032: the module's six. Inside the 275 s to 654 s band on
`docs/development/measurements.md`. `main`'s hook ran `all` again on the merge: 8,038 and none
failed, 273 s from 12:59:33Z to 13:04:06Z. The keyring race (ledger 374) did not appear on either.

## Guard records

912 records by the TOML reader before, 913 after; census 802 + 110 before, 802 + 111 after, the
line at `guards/guards.toml:84` moved in task 1's green commit. One new, one rewritten, both
measured through `scripts/guards.sh --remeasure` with `WIXEN_TEST_THREADS` untouched and the
counts written by the runner.

| Record | File | Break | Red | Run |
|---|---|---|---|---|
| a spelling language chosen for a region is shown as chosen, not as the checker's answer | `which_language_row.rs` | `names_a_region` inverted, so no tag names a region and every tag takes the resolver's answer, the tree before this plan | 2, "all 2 tests named went red, and nothing else did": the region tag without a dictionary and the region tag not listed; the case-insensitive one stays green because its row is an exact usable match the resolver answers too | rebuild 26 s, run 49 s, on the library |
| the settings screen shows a stored language nothing offers as itself, not as row 0 (rewritten) | `wx_settings.rs`, `suite` the language target | the rule's answer discarded and `RowToShow::Existing(0)` matched, the screen before 2026-09-16 | 1, "the one test named went red, and nothing else did" | rebuild 12 s, run 1 s, on the target |

The rewritten record's `before` had been `.or_else(|| row_of(stored));`, the very line task 2
deleted, as the checker said; its comment now says which of the target's three cases the break
reddens (all three, inside the one test) and that the rule has a record of its own on the
library. Counts written: `which_language_row.rs` 6 on one record; `wx_settings.rs` 0 and the
target 1 on the rewritten one. The count check printed no remedy at any commit: no file an
existing record names gained or lost a test. `wx_app.rs` untouched.

## What the tree contradicted

Every command in the plan's five premises was re-run against `main` at `d42b4aed` before
anything was built, and all held: `language_rows_and_selection` at `wx_settings.rs:884`, the
`or_else` at `:892`, `read_settings`'s rebuild at `:3278`, `language_to_use` at
`spellcheck/mod.rs:354-380`, the record at `guards.toml:23471` with the deleted line as its
`before`, 32 records naming the checker's module, 0 tests in `wx_settings.rs`, the target
passing here (`test result: ok. 1 passed`). One thing the plan's behaviour block lacked that
the tree and 09-02's summary both had:

1. **The fallback for a bare tag.** The block said a bare tag is the resolver's row, "when that
   answers nothing or names no row, `Added(stored)`". The code at `d42b4aed` and 09-02's summary
   said the resolver's row, else the stored tag's own row, else added. A bare `fr` on an en-US
   machine that lists `fr` without a dictionary would, under the block, be added a second time
   at the end, named `fr` rather than in its own words, beside the row Windows offered. The
   fallback is kept and a sixth case holds it
   (`test_a_bare_tag_the_checker_cannot_place_stands_as_its_own_row_when_listed`). Observation
   668 in the skill observation log.

## Deviations from plan

**1. [Decision] The bare tag's own listed row stands as the fallback**, above. The plan's
"at least five tests" is six.

**2. [Decision] The screen record's break matches the whole answer rather than the plan's
prose.** The plan said "the rule's answer replaced by `Existing(0)`"; the `after` computes the
rule's answer, drops it, and matches `RowToShow::Existing(0)`, so the import stays used and the
break compiles without a warning. Same meaning, the same one test red.

**3. [Decision] One ledger entry, 530**, above, for the runner's case; the plan's files list did
not name the ledger and the phase README asks for it.

Everything else executed as written. **No scripted edit touched a tracked file: the exception set
for this plan is zero, and it stayed there.** Every tracked file was changed by Read then Edit or
Write; `cargo fmt` ran before each commit, which is the project's formatter and not a rewrite;
the only `sed`, `awk`, `grep` and `tr` in the session read files and logs; `git checkout` was not
needed. Commit messages were written to the scratchpad and passed with `-F`. Carriage returns
measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on each,
`.planning/WINDOWS.md` included. No em-dash in any file this plan wrote, measured with `grep -c`
for the byte sequence: zero on each. `git commit` and `git merge`, never `gsd-tools query
commit`; never `--no-verify`; `check.sh` never piped, its exit status appended to its own log by
the shell that ran it. No AI attribution in any commit. `Cargo.toml` and `Cargo.lock` untouched;
no crate added (T-11-SC). The tester's profile was not read; no binary was started; nothing was
sent to any window this plan did not build.

## Threat register

T-11-05 mitigated: a chosen tag is never rewritten on save, held by the region half's three cases
and the measured record, and by the integration reading where the machine can run it. T-11-06
mitigated: the red is a unit case over hand-built rows, red here at `7c8ebda9` and, by
construction, on any machine; the runner's run after the push is the check, ledger 530. T-11-SC:
nothing added. No new surface outside the register: the screen reads the same stored value from
the same file, and the rule is pure.

## Ledger

`.planning/WINDOWS.md` 530 written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 529 before, 530 after; 499
open before, 500 after.

| id | kind | what |
|---|---|---|
| 530 | unrun-verify | the target's en-AU case, unrun red or green here because this machine offers the language; the next push of `main` and its Test Suite job are the reading |

## The issue

None to close or comment: no issue was filed for this, and the run is the record. Nothing was
run through `gh` after the merge.

## Known stubs

None. `which_row_shows` is reached by `language_rows_and_selection`, which `add_language_and_spelling`
calls when the General tab is built (`wx_settings.rs:930`) and `read_settings` calls when OK is
pressed (`:3275`); both callers are in the shipping path and the integration target builds the
real tab and reads back through the second.

## Not done here, on purpose

Whether GitHub's runner keeps en-AU is the next push of `main`, Pratik's, and FOUND-17's `[S]`
line and its "CI on `main` is green" clause wait on it with ledger 530; the two `[D]` lines are
ticked with every test named above. Roadmap criterion 11 is closed structurally for its rule and
reading clauses and its CI clause waits the same way; the row is `1/15`. Nothing else in the phase
was touched: `wx_app.rs` is as it was, and no page owes this plan a sentence beyond the changelog.

## Self-Check: PASSED

`src/presentation/which_language_row.rs` exists; `grep -c 'pub mod which_language_row'
src/presentation/mod.rs` is 1; `grep -c 'which_row_shows(' src/presentation/wx_settings.rs` is 1;
`grep -c 'language_to_use' src/presentation/wx_settings.rs` is 0; `guards/guards.toml` holds 913
records by the TOML reader and the census line says 111; `docs/changelog.md` holds the line
beginning "**A spelling language you chose for a region this computer has no dictionary for";
`.planning/WINDOWS.md` holds 530 in both halves. Commits `7c8ebda9`, `29e4d850`, `016b1cfd` and
`316ea755` are in `git log --oneline` on `main`.
