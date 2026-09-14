---
phase: 08-every-number-the-project-quotes
plan: 01
status: complete
subsystem: testing
tags: [measurements, guards, mutants, timing, check-sh, house-style, provenance]

requires:
  - phase: 06-how-the-application-speaks
    provides: "`docs/wcag-coverage.md` as the precedent for a page that says at the top how each figure was taken, and the coverage-page guard record as the precedent for a record whose companion goes red with the reading"
  - phase: 07-installing-updating-and-what-is-stored
    provides: "`the_suites_that_guard_what_changed` reading a record whose break lands outside `src/`, without which this plan's record would have been invisible to the gate"
provides:
  - "`docs/development/measurements.md`: the one page a figure about this tree is written on, twenty rows today, columns what, value, command, date, commit, conditions"
  - "`tests/every_number_carries_its_command_and_its_date.rs`: the reading that refuses a row without a backticked command, a `20YY-MM-DD` date or a seven-hex commit, refuses an absent or empty table, refuses two rows with one `what` and one date; four companions splicing each omission into the real page; a self-check that the target is in both of `check.sh`'s lists"
  - "`scripts/check.sh`: the target in `guards_that_read_the_whole_tree` and in the documents-only line"
  - "`guards/guards.toml`: one record, `a figure on the measurements page cannot lose its date`, suite the new target, five red; census 592, 784 records"
  - "`scripts/guards.py`: `the_timing_line`, printed after every run as `timed: rebuild N s, run N s, N s in all`, the run split into `cargo test --no-run` and the run so the two terms are timed apart; `THE_COST_OF_ASKING_PROPERLY` at 92 against 47, dated"
  - "`.cargo/mutants.toml`: the timeout comment rewritten around 104 s for every target and 52 s for the library at eight threads, both taken twice today, the values untouched"
affects: [08-02, 08-03, 08-04, 08-05, 08-06, 08-07, 08-08, 08-09]

actuals:
  tokens: 12000
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A page of dated measurements held by a reading of its shape and never its value, with a companion per omission spliced into the page's own lines"
    - "A runner that prints its own rate per record, so a log is a rate series and the constant is re-taken by reading rather than by stopwatch"
    - "A guard record for a document, suite the integration target that reads it, red listing the reading and every companion that asserts the page is clean"

key-files:
  created:
    - docs/development/measurements.md
    - tests/every_number_carries_its_command_and_its_date.rs
  modified:
    - scripts/check.sh
    - guards/guards.toml
    - scripts/guards.py
    - .cargo/mutants.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

key-decisions:
  - "The page lives under docs/development/ because installer/Wixen-Mail-Setup.iss ships docs\\*.md and is not recursive; a page of build-machine figures is a note to ourselves"
  - "The record for the page names five tests, the reading and its four companions, because each companion asserts the real page is clean before splicing; that is the coverage-page precedent and not a record to narrow"
  - "The two timing terms are split by two cargo invocations, --no-run then the run, rather than by reading cargo's stderr as it streams; the split costs one fingerprint check and adds no second way for the capture to come back empty"
  - "timeout_multiplier and minimum_test_timeout keep their values: the multiplier is applied to a baseline the tool measures fresh at the start of every run, so the values were never wrong and moving them without a run to read the result off is the mistake the comment warns about"
  - "The product on the page is 784 x 92 s and not 783 x 86 s: the record count is today's including this plan's own record, and the rate is the one taken through the runner's own timing line"

patterns-established:
  - "A row on the measurements page: value, the command as run, the date, the commit, and what moves it; a later re-take is a new row, not an edit"
  - "A red commit for a script no cargo target reads names house_style's doctest runner in its trailer, with the doctest written before the function has a body"

requirements-completed: [PERF-06, PERF-07]

coverage:
  - id: D1
    description: "docs/development/measurements.md exists with twenty rows, each with what, value, command, date, commit and conditions"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_page_holds_the_rows_the_phase_was_scheduled_from"
        status: pass
    human_judgment: false
  - id: D2
    description: "The reading refuses a row lacking its command, date or commit, and each companion plants exactly that omission in the real page and is answered with exactly that row"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_reading_can_see_a_row_whose_date_is_missing"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_reading_can_see_a_row_whose_command_is_missing"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_reading_can_see_a_row_whose_commit_is_missing"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_reading_can_see_a_row_that_appears_twice"
        status: pass
    human_judgment: false
  - id: D3
    description: "The target runs on the commits that could break it: in both of check.sh's lists"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_this_target_runs_on_the_commits_that_could_break_it"
        status: pass
    human_judgment: false
  - id: D4
    description: "The guard runner prints a timing line per run in a fixed shape, and the cost constant is today's"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/house_style.rs#test_the_guard_runner_still_obeys_its_own_examples"
        status: pass
      - kind: other
        ref: "scripts/guards.sh --remeasure \"the editor stores off when the last alert is taken away\", twice, timing lines 93 s and 91 s"
        status: pass
    human_judgment: false
  - id: D5
    description: "The mutant count is on the page from cargo mutants --list and the mutation timeout comment is calibrated against today's suite"
    requirement: PERF-07
    verification:
      - kind: other
        ref: "cargo mutants --list | wc -l, 12,335 in 3 s; cargo test --all-targets timed twice, 104 s and 103 s; cargo test --lib at eight threads twice, 52 s and 52 s"
        status: pass
    human_judgment: false

duration: 75min
completed: 2026-09-14
---

# Phase 8 Plan 01: Every number beside its command Summary

**One page, twenty rows, every figure the rest of the phase is scheduled from taken today by the command in its row; a reading that refuses a bare row on every commit; the guard-sweep rate re-taken through a timing line the runner now prints itself; the mutation timeouts read against today's suite.**

Every figure below is one this session took, with the command. Where the plan's premises said otherwise, the tree won and the difference is named.

## Performance

- **Duration:** about 75 minutes, from the first read at about 19:40Z to the summary commit
- **Started:** 2026-09-14T19:40:00Z (approximate; the first commit is 20:04Z)
- **Completed:** 2026-09-14T20:53:08Z for the merge; the summary followed
- **Tasks:** 3 of 3
- **Files modified:** 7 on the branch, plus the three planning files and the ledger on `main`

## Accomplishments

- `docs/development/measurements.md` exists, twenty rows, each with what, value, command, date, commit and conditions. Sixteen taken in task 1, two in task 2, two in task 3.
- `tests/every_number_carries_its_command_and_its_date.rs`, ten tests: the reading, a row-count floor, four companions splicing an omission into the real page, two over literal text (no table, empty table, a command holding a pipe), and the self-check with its own companion over `scripts/check.sh`. Seven were red against the absent page and the unlisted target; all ten green after.
- The target is in both of `check.sh`'s lists with the reason at each.
- One guard record for the page, measured by hand and confirmed by the runner: five red, nothing else. Census 592, 784 records by the TOML reader.
- `scripts/guards.py` prints `timed: rebuild N s, run N s, N s in all` after every run, held by three doctests; 57 worked examples where there were 54.
- `THE_COST_OF_ASKING_PROPERLY` is `about 92 seconds a record against 47.`, dated, with the commit, the thread setting and the library's test count beside it.
- `.cargo/mutants.toml`'s timeout comment names 104 s and 52 s, taken twice each, says which applies, what a mutant is called hung after under each, and that 46 s is what it used to say. Both settings unchanged.
- `scripts/check.sh all` green on the branch at `d52e9cdd`, 7,687 passed and none failed, 310 s; green again on the merge, 7,687 passed, 310 s. No keyring race either time.

## The figures, each with its command

Every row on the page was produced by running the command in the row during this plan. The two headline rows the plan asked the summary to quote, as run:

```
python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"
-> 783            (at 7b2482b1, before this plan's record; 784 after it)

cargo mutants --list | wc -l
-> 12335          (3 seconds, cargo-mutants 27.1.0)
cargo mutants --list-files | wc -l
-> 247
```

The rest of task 1's rows, all at `7b2482b1`: 568 records naming one file, 4 spelled as inline tables, 200 distinct files named, 21 naming `tests/house_style.rs`, 48 naming `src/presentation/wx_app.rs`, 2,025 commits by `git rev-list --count HEAD`, 7,679 test attribute lines, 7,245 tests the library builds by `cargo test --lib -- --list | tail -1`, 70 test functions in `house_style.rs` and 199 in `wx_app.rs`, 360,792 lines over 286 files under `src/`, 441 ledger entries by `grep '^total_count:'`, and the full-gate band of 275 s to 654 s from the commit-body harvest.

**The timing lines the runner printed**, one record, `the editor stores off when the last alert is taken away`, one file, library suite, `WIXEN_TEST_THREADS` unset so 8, nothing else building (no `cargo.exe` or `rustc.exe` in `tasklist` before each take), build warm, working tree at `bb61e88e` plus the uncommitted timing code:

```
take 1   timed: rebuild 46 s, run 47 s, 93 s in all
take 2   timed: rebuild 44 s, run 47 s, 91 s in all
--named-only   timed: rebuild 44 s, run 3 s, 47 s in all
```

The two takes differ by 2 s, within a fifth, so no third was taken. The pre-read the runner does before any break printed 68 s, 91 s and 91 s and is not in the rate. The product on the page: `784 x 92 s = 72,128 s, about 20 hours`.

**The two suite figures**, at `9399a1e2`, every target built first with `--no-run`, `date +%s` around the command in bash, `tasklist` quiet before each:

```
cargo test --all-targets --no-fail-fast        104 s, 103 s     7,687 passed over 55 result lines
cargo test --lib -- --test-threads=8            52 s,  52 s     7,244 passed, 1 ignored; harness said 51.06 s and 51.60 s
```

`python -m doctest -v scripts/guards.py`: 54 passed in 37 items before task 2, 57 passed in 39 items after.

## What the tree contradicted in the plan

1. **`check.sh --suites-for guards/guards.toml docs/development/measurements.md` prints nothing, and the plan's criterion said it would print the target.** The coupling function drops any candidate already in `guards_that_read_the_whole_tree`, with a comment saying why: every scoped run ends with that list, so the answer would be a second run of the same target. The plan required the target to be in that list. The two requirements were each right and jointly made the criterion's expected output impossible. The coupling was shown on a scratch copy of the script with the target taken out of the list, which answers `every_number_carries_its_command_and_its_date`. Ledger 442.
2. **The rate moved, and its terms moved in opposite directions.** The plan's premise 3 and the roadmap's criterion 5 said about 86 s a record, 35 s rebuild plus 51 s run, measured through the runner's total time. Through the timing line: 44 to 46 s rebuild plus 47 s run, 92 s. Against 2026-09-10's 29 s plus 66 s the sum barely moved while both terms did, which is the reason the line prints them apart.
3. **The library holds 7,245 tests by `--list`, not 7,244.** The plan's criterion 5 text says 7,244 over the run's `passed` count, which leaves out the one ignored test. Both are right about different things; the page says which it counts.
4. **The commit count is 2,025, not 2,023.** Two planning commits landed after the README's reading.
5. **`cargo test --all-targets` is 104 s, not the 239 s or 197 s `CLAUDE.md`'s gate paragraphs quote.** Taken twice with every target built and nothing else running. Those paragraphs are 08-06's to point at the page; the row exists for it.

## Task commits

Branch `every-number-beside-its-command` from `main` at `7b2482b1`.

1. **Task 1, red:** `9fa49dba` test(08-01), seven tests named in trailers, red because the page did not exist and the target was in neither list, said so in the message.
2. **Task 1, green:** `1022b9d2` feat(08-01), the page, both lists, the changelog entry.
3. **Task 1, record:** `0588644e` test(08-01), one record, census 592.
4. **Task 2, red:** `bb61e88e` test(08-01), the three doctests on `the_timing_line` before it had a body; `python -m doctest` reported 3 of 3 failing and `house_style::test_the_guard_runner_still_obeys_its_own_examples` went red on it.
5. **Task 2, green:** `9399a1e2` feat(08-01), the body, the split run, the constant, the rate and product rows.
6. **Task 3:** `d52e9cdd` docs(08-01), the timeout comment and two rows. Configuration and a page row, which `CLAUDE.md` lists among the exceptions to test-first; nothing here changes behaviour.
7. **Merge:** `b63527ab`, gate green on the merge.

**Plan metadata:** the commit carrying this summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md`.

What the gate selected per file, by `scripts/which-checks.sh` on the branch: `tests/every_number_carries_its_command_and_its_date.rs` alone answers `affected` (its own target plus the tree guards); `docs/development/measurements.md` alone answers `docs_only`; with `scripts/check.sh` beside it, `affected`; `guards/guards.toml` answers `affected` and maps to no target of its own, so the seven `house_style` tests that read it ran as tree guards; `scripts/guards.py` answers `affected` and maps to no target, reached by `house_style`'s doctest runner; `.cargo/mutants.toml` answers `affected`, read by nothing. `--suites-for` answers nothing for any of them, for the reason under contradiction 1.

## Deviations from Plan

**1. [Rule 2, missing critical] `conditions` may not be empty.** The plan's behaviour said `conditions` may say `none`; the reading also refuses an empty `conditions` cell, because an empty cell cannot be told from a forgotten one and `none` costs four letters. The page's "How to add a row" says so.

**2. The record names five tests, not one.** Anticipated by the plan's action: the companions read the real page and assert it is clean before splicing, so a page that already disagrees fails them all. The record names exactly what went red, as `guards.toml`'s own header requires, on the coverage-page precedent.

**3. The `--named-only` figure was taken rather than inferred.** The constant's second figure, "against 35", is the cost of a named-only run, which the plan did not ask for; one run of the same record in that shape gave 47 s, and the constant quotes it.

Otherwise the plan was executed as written. No package installed, `Cargo.toml` untouched, no version bump, no user-visible change beyond the changelog entry the plan asked for.

## Guard records

One added, measured by hand on its own target and confirmed under `--remeasure`. The timed record's counts were written back by `--remeasure` and matched what was there, so `guards/guards.toml` carries only the new record. No test was added to `tests/house_style.rs` or `src/presentation/wx_app.rs`: 70 and 199 before, 70 and 199 after, and the count check did not fire on any commit.

Records: 783 before by the TOML reader, 784 after. Census 192 + 592.

## Ledger

`.planning/WINDOWS.md` 441 before, 443 after, both halves of each entry written by `gsd-tools windows append`, no backslash in either:

- 442, deviation, `scripts/check.sh`: the `--suites-for` criterion answered silently by design.
- 443, todo, `scripts/guards.py`: the docstring of `run_the_whole_suite` still quotes 23 s and 89 s from before 2026-09-09, a fifth sweep figure for 08-06 to point at the page.

## Issues Encountered

None that stopped anything. The gate passed on every commit through the hook, never piped; `check.sh all` was redirected to a file and its exit status read directly, twice green.

## Known Stubs

None. Every figure on the page was produced by a command run in this session; the reading holds the page's shape; the timing line prints on every run; the constant and the comment name today's figures.

## Threat Flags

None. No network endpoint, auth path, file access pattern or schema change. The reading opens one page under `docs/` and one script.

## Self-Check: PASSED

The three created files exist on disk and the seven commits named above are in `git log --all`, checked by `[ -f ]` and `grep` before this section was written. `main` is ahead of `origin/main` and nothing was pushed.

## Next Phase Readiness

08-02 reads the page and adds rows. Every later plan has a row to point at: 08-07's checkpoint quotes the product row, 08-08 quotes the mutant count row and the two suite rows. The four sweep-cost figures elsewhere in the tree (`CLAUDE.md`, `tests/house_style.rs`, `guards/guards.toml`, `scripts/guards.sh`) and the two gate figures in `CLAUDE.md` are 08-06's to replace with references to the page; the roadmap's criterion 5 is corrected in this plan's metadata commit to quote the product row.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-14*
