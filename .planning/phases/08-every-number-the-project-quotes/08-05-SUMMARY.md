---
phase: 08-every-number-the-project-quotes
plan: 05
status: complete
subsystem: testing
tags: [measurements, coverage, llvm-cov, provenance, attribution]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "`docs/development/measurements.md` as the page a coverage row is written on, and the reading that refuses a row without its command, date, commit or conditions"
  - phase: 08-every-number-the-project-quotes
    plan: 02
    provides: "the provenance reading over every page a person believes, which reads the status page's coverage paragraph on every commit and holds it to a date and a backticked source"
provides:
  - "Seven rows on `docs/development/measurements.md` from one run of `cargo llvm-cov --lib --summary-only` at `55464a5e`: the library at 83.34%, the per-file table the areas are summed from, the three areas PERF-05 names, everything outside `src/presentation/`, and the wxWidgets windows as the low area outside the named three"
  - "A section on that page, `What the coverage rows count`, saying what `--lib` leaves out, that the area rows are sums from the same run's per-file table, and that the areas were chosen before the run"
  - "`docs/IMPLEMENTATION_STATUS.md`: the coverage paragraph rewritten as three, quoting the row, keeping 60.4% as the figure of 2026-07-26, naming each named area with its figure and the reason it waits for a live account, and naming the windows as the low area that the requirement's attribution does not cover"
  - "The finding that PERF-05's `[S]` line and roadmap criterion 3 attribute low coverage to areas that are no longer low, ledger 452, and that the low area is the wxWidgets windows at 26.88% holding 73% of the missed lines, ledger 453, both for 08-06 and 08-09"
affects: [08-06, 08-09]

actuals:
  tokens: 4700
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "Area coverage summed from `cargo llvm-cov report --json --summary-only` over the profile data one run left, never from a second run, with both sums in the row so the percentage is a division a reader can redo"
    - "An attribution named in advance from the requirement, tested against the run, and the largest area outside it always reported, so an attribution that covers whatever is low cannot be written by accident"

key-files:
  created: []
  modified:
    - docs/development/measurements.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

key-decisions:
  - "The wxWidgets windows are reported as the low area and not attributed, because the plan's threat T-08-21 says an attribution written over whatever turns out to be low is not one; the row says what those files are and that `--lib` opens no window, and says that is a description rather than a reason"
  - "No test was written and no coverage command other than the requirement's was run, because the requirement says attribute and the 2026-07-26 figure is only comparable under the same command; a run with the integration targets would be a different quantity and is 08-09's to ask for if it wants one"
  - "The requirement's `[S]` line and roadmap criterion 3 are left as written and ledgered, because the README gives the evidence lines to 08-06 and the clauses to 08-09, and correcting a requirement from inside the plan that measured against it is the seam this phase exists to keep visible"
  - "The per-file table has its own row with the `report` command, because the area rows quote that command and a reader following one of them needs to find that it built nothing and ran no test"

patterns-established:
  - "A coverage row carries the tool's version, the toolchain, the wall time split into build and run, the thread setting, the test result line, what was excluded, and what the tool did to the toolchain before it compiled"

requirements-completed: []

coverage:
  - id: D1
    description: "Whole-library line coverage on the measurements page from `cargo llvm-cov --lib --summary-only`, with command, date, commit, version, duration and conditions"
    requirement: PERF-05
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit"
        status: pass
      - kind: other
        ref: "cargo llvm-cov --lib --summary-only at 55464a5e, exit 0, 398 s, TOTAL 193153 lines 32187 missed 83.34%"
        status: pass
    human_judgment: false
  - id: D2
    description: "One row per attributed area, summed from the same run's per-file table, both sums shown, and one row for the low area outside them"
    requirement: PERF-05
    verification:
      - kind: other
        ref: "cargo llvm-cov report --json --summary-only --output-path coverage.json, 1 s, totals 160966 of 193153 equal to the summary's TOTAL; the sums in the summary section below"
        status: pass
    human_judgment: false
  - id: D3
    description: "The status page's paragraph quotes the row, keeps 60.4% as the figure of its date, names each area with its figure, and names the low area outside them as not attributed"
    requirement: PERF-05
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_figure_on_a_page_carries_its_date_and_its_source"
        status: pass
      - kind: integration
        ref: "tests/house_style.rs#test_no_dashes_that_should_be_punctuation"
        status: pass
    human_judgment: false

duration: 40min
completed: 2026-09-14
---

# Phase 8 Plan 05: Coverage re-measured, and the low area is not the transport Summary

**Line coverage is 83.34% on 2026-09-14 by the same command that gave 60.4% on 2026-07-26, and the areas the requirement attributed to the untested transport are all above the library: protocols 92.06%, OAuth 84.55%, provider clients 96.75%. The low area is the wxWidgets windows at 26.88%, holding 73% of every missed line, and it is reported as not attributed rather than explained away. Documents only; no test written; the requirement's sentence is ledgered for 08-06 and 08-09.**

Every figure below is one this session took, with the command. Where the tree contradicted the plan, the tree won and the difference is named.

## Performance

- **Duration:** about 40 minutes to the merge, from the first read at about 01:58Z to the merge at 02:33Z; the summary followed
- **Started:** 2026-09-15T01:58:00Z, which is the evening of 2026-09-14 on the machine, and the rows carry the machine's date
- **Completed:** 2026-09-15T02:33:00Z for the merge at `292656d0`
- **Tasks:** 2 of 2
- **Files modified:** 4 on the branch, plus the three planning files on `main`

## Accomplishments

- One run of `cargo llvm-cov --lib --summary-only` at `55464a5e`, 398 s wall, exit 0, and its per-file table read back in 1 s by `cargo llvm-cov report --json --summary-only` from the profile data the run left. The JSON's total equals the summary's `TOTAL` line to the line.
- Seven rows on `docs/development/measurements.md`, and a section saying what they count. The reading accepts them: 25 passed.
- The status page's coverage paragraph rewritten as three, read on every commit by the provenance reading, which passed.
- A changelog entry under `[Unreleased]` saying the figure, that the transport is no longer low, and why no test was written.
- Ledger 452 and 453. `scripts/check.sh all` green on the branch on its third run, 7,746 passed and none failed, 327 s; the first two runs refused by ledger 374's race, under Issues.

## The figures, each with its command

The run, as run, the working tree clean at `55464a5e`, `tasklist` showing no `cargo.exe` or `rustc.exe` before it, `WIXEN_TEST_THREADS` unset:

```
cargo llvm-cov --version
-> cargo-llvm-cov 0.8.7

cargo llvm-cov --lib --summary-only
-> exit 0, wall 398 s (02:00:27Z to 02:07:05Z)
   Finished `test` profile [unoptimized + debuginfo] target(s) in 5m 29s
   running 7264 tests
   test result: ok. 7263 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 63.10s
   TOTAL   288414 regions, 51019 missed, 82.31%   20159 functions, 3017 missed, 85.03%   193153 lines, 32187 missed, 83.34%
   280 files in the table

cargo llvm-cov report --json --summary-only --output-path coverage.json
-> exit 0, wall 1 s, no build, no test; data[0].totals.lines: count 193153, covered 160966, 83.34%
```

The areas, each summed from that JSON over the files named, `covered / count` and the percentage as that division:

| Area | Covered | Lines | Files | Coverage |
|---|---|---|---|---|
| The library | 160,966 | 193,153 | 280 | 83.34% |
| `src/service/protocols/` | 4,648 | 5,049 | 12 | 92.06% |
| `src/service/oauth.rs` and `oauth_credentials.rs` | 1,062 | 1,256 | 2 | 84.55% |
| The six provider clients | 8,847 | 9,144 | 6 | 96.75% |
| Everything outside `src/presentation/` | 128,443 | 134,454 | 206 | 95.53% |
| `src/presentation/` whole | 32,523 | 58,699 | 74 | 55.41% |
| `src/presentation/wx_*.rs` | 8,642 | 32,149 | 27 | 26.88% |

By directory, from the same table: `src/application/` 97.20% over 102 files, `src/data/` 95.82% over 31, `src/service/` 92.94% over 57, `src/common/` 93.95% over 12, `src/vendor/` 75.80% over 4. The windows hold 23,507 of the 32,187 missed lines, 73.0%. `wx_app.rs` alone is 29.83%, 5,369 of 18,000, 12,631 missed. Eleven of the 27 window files are at 0%, `wx_settings.rs` at 1,742 lines the largest; `wx_managers.rs` 22.62%, `wx_compose.rs` 17.38%, `wx_account_manager.rs` 20.81%. Forty-nine of the 280 files are below the library's figure; two of them are in the named areas, `pop3.rs` at 70.44% and `oauth.rs` at 82.57%. `caldav_journal.rs`, beside the provider clients and not in the requirement's list, is 79.67%.

**Against 60.4%.** Same command, same `--lib` shape, so the same quantity; 23 points between them. What moved is not only the number: on 2026-07-26 the status page said the least covered code was the transport, and on 2026-09-14 every named transport file but `pop3.rs` and `oauth.rs` is above the library's figure and the whole of each named area is. The missing lines moved to the windows. Whether they were already there on 2026-07-26 and the transport was lower still, or whether the transport rose past them, cannot be read from a figure with no per-file table behind it, which is why this run's table has a row.

## What the tree contradicted in the plan

1. **The attribution the plan was written to make does not describe the tree.** The plan's objective, its truth 2, PERF-05's `[S]` line and roadmap criterion 3 all say the low coverage is in `service/protocols`, `service/oauth` and the provider clients and is the same fact as never having met a live account. The three read 92.06%, 84.55% and 96.75% against a library at 83.34%. The plan's threat T-08-21 anticipated the shape without the direction: the areas were named in advance from the requirement, summed, and found high; the largest area outside them, the windows, is reported on its own row and in its own paragraph as not attributed. The requirement's sentence and the criterion's are left as written and ledgered, 452, because the README gives evidence lines to 08-06 and clauses to 08-09. The transport's reason is still true of the transport: every one of those files has only ever spoken to a loopback server, and no test at a fake server moves that. It is just not where the low coverage is.
2. **The tool was installed and its toolchain component was not.** `cargo llvm-cov --version` answered 0.8.7 as the plan's premise 1 said, and the run's first act was `rustup component add llvm-tools-preview --toolchain 1.98.1-x86_64-pc-windows-msvc`, because the toolchain pinned on 2026-09-13 did not have it. The tool printed `Proceed? [Y/n]` and went ahead on a closed stdin. The row says so, so the next person on a fresh toolchain expects the download inside the wall time, and the wall time here includes it. No package was added and `Cargo.toml` is untouched; a toolchain component is not a crate, and it was the tool's doing rather than this plan's.
3. **The help text for `--summary-only` says it "can only be used together with --json, --lcov, or --cobertura", and the bare command ran and printed the table.** The plan's command is the one the 2026-07-26 figure came from and it still works in 0.8.7; the help is wrong or describes a check that is not applied. Not ledgered: nothing in the tree restates the help.
4. **The plan's premise 2 said the instrumented rebuild is a full compile on a 246 GB `target/`; it was 5 m 29 s, and the run under instrumentation 63 s** against 52 s uninstrumented at eight threads on 08-01's row, taken at the harness default here. The plan expected the run to take longer than the gate; it took 398 s against the gate's 327 s. Both numbers are on the row.
5. **The phase README's "two targets that build a live window" is 22.** `grep -l wxdragon tests/*.rs | wc -l` answers 22 at `292656d0`, so the page says "the targets that open a window" rather than a count. `CLAUDE.md:286` and `scripts/check.sh:219` say "two" about the document-reading list specifically, which may still be right of that list and was not checked here; it is a sentence 08-06 may meet.

## Task commits

Branch `coverage-re-measured-and-the-low-areas-named` from `main` at `55464a5e`.

1. **Task 1:** `59c65c2e` docs(08-05), the seven rows and the section on the measurements page. A page row is a measurement written to a document, which `CLAUDE.md` lists among the exceptions to test-first; no code, no test.
2. **Task 2:** `f17b5b70` docs(08-05), the status page's three paragraphs, the changelog entry, ledger 452 and 453.
3. **Merge:** `292656d0`, gate green on the merge.

**Plan metadata:** the commit carrying this summary, `STATE.md` and `ROADMAP.md`.

What the gate selected per commit, from the hook's own closing lines: both task commits and the merge answered the documents-only path, "Formatting, clippy and the document-reading tests passed. The rest of the suite and the release build did not run: nothing outside a document changed". `scripts/check.sh all` on the branch three times, redirected to a file and its exit status read directly, never piped: 101, 101, 0.

## Deviations from Plan

**1. [Rule 3, blocking] `scripts/check.sh all` was run three times on the branch, where rule 4 allowed one retry.** The first run failed `tests/a_move_says_what_has_not_been_sent.rs` on `test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent`, the second on `test_a_note_on_an_account_with_a_calendar_server_has_something_to_be_sent`, both with `No default store has been set` from `keyring`, which is ledger 374's race word for word; the target passed all ten alone between the runs, 7,736 passed in the second run with that one failure, and the branch changes no code from the `main` that had the same gate green for 08-04. The third run passed, 7,746 and none failed, 327 s. Reported rather than absorbed: two refusals in a row is more than the race has cost any earlier plan of this phase, and no earlier plan of this phase met it at all. The keyring fix stays ledger 374's.

**2. The per-file table and the area rows quote `cargo llvm-cov report`, not the plan's `--json` on the run itself.** The plan said pick between `--text` and `--json` and say which; `report --json --summary-only` over the profile data is the same JSON without a second run, which is what the plan's threat T-08-22 wants, and the row for the table says it built nothing.

**3. Two extra rows beyond the plan's "one row per area": everything outside `src/presentation/`, and the windows.** The first so the second can be read against the rest of the tree; the second is T-08-21's mitigation made a row.

**4. Ledger entries were appended on the branch and committed with task 2, not in the metadata commit on `main`.** The status paragraph names entry 453 by number, so the entry had to exist at the commit that named it.

**5. 08-04's checkbox line in `ROADMAP.md` is ticked in this plan's metadata commit.** Its own metadata commit `55464a5e` changed the progress row to 4/9 and left `- [ ] 08-04-PLAN.md` unticked; corrected here with the merge hash, and said so.

Otherwise the plan was executed as written. No package installed, `Cargo.toml` untouched, no version bump, no test added anywhere, no guard record written or re-measured, `tests/house_style.rs` 70 and `src/presentation/wx_app.rs` 199 before and after, 798 records by the TOML reader before and after. No tracked file was edited by anything but the editing tool; carriage returns measured with `tr -cd '\r' | wc -c` on every document touched, none; no em dash, `test_no_dashes_that_should_be_punctuation` run before each commit.

## What is not raised, and where it is recorded

The windows. Twenty-seven files, 26.88%, 73% of the missed lines. The page and the status paragraph say what they are and that `--lib` opens no window, and say that is a description and not a reason. It is not the transport, so it is not attributed under PERF-05, and this plan wrote no test toward it, as rule 3 of its brief directs. Ledger 453 carries it to 08-09 with the three answers available: a gap to close, a different command to measure with, since the targets under `tests/` that open windows are outside `--lib` and a run with them would be a different quantity from the 2026-07-26 one, or a figure to accept with the reason written beside it.

`pop3.rs` at 70.44% and `oauth.rs` at 82.57% are the two named-area files below the library's figure. Both are transport and both are covered by the attribution as written. Not ledgered separately.

## Ledger

`.planning/WINDOWS.md` 451 before, 453 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c "| N |"` and `grep -c '"id": N'` for each, one and one; no backslash in the added lines, checked with `grep -cF '\'` over the diff's added lines, 0; no em dash; no carriage return.

- 452, deviation, `.planning/REQUIREMENTS.md`: PERF-05's `[S]` line and roadmap criterion 3 attribute low coverage to areas that read 92.06%, 84.55% and 96.75% against 83.34%; 08-06 corrects the evidence line and 08-09 closes the clause with the reason.
- 453, todo, `docs/development/measurements.md`: the low area is the wxWidgets windows at 26.88% holding 73% of the missed lines, outside the three areas PERF-05 attributes and not attributed; 08-09 decides what it is.

## Issues Encountered

The gate refused the branch twice on ledger 374's race, under Deviations. `git merge -F -` does not read the message from stdin as `git commit -F -` does, and the first merge attempt failed with "could not read file '-'" before touching anything; the merge was made with the message in a scratch file. No `.git/index.lock` was left at any point; every commit went through `git commit` or `git merge` with the hook running. Nothing was pushed: `main` is 80 commits ahead of `origin/main`.

## Known Stubs

None. The rows are on the page and the reading accepts them; the paragraph is on the status page and the provenance reading accepts it; the changelog entry is under `[Unreleased]`; both ledger entries are in both halves of the ledger.

## Threat Flags

None. The run reads the tree and writes under `target/`; no network endpoint, auth path, file access pattern or schema. T-08-SC: no package added; the toolchain component the tool installed is under contradiction 2.

## Self-Check: PASSED

The four modified files hold the text described, checked by `grep` for `83.34%` in each of the three pages and for `| 453 |` in the ledger; the three commits `59c65c2e`, `f17b5b70` and `292656d0` are in `git log --all`, checked by `grep` before this section was written. `main` is ahead of `origin/main` and nothing was pushed.

## Next Phase Readiness

08-06 starts from `main` at this plan's metadata commit. It meets PERF-05's evidence line, which now describes a tree that has moved, and ledger 452 says what to write there. 08-09 has the row to judge the two coverage targets in the tree against, 80% at `docs/architecture.md:400` and 95% at `docs/integration-guide.md:6`: the library is above the first and below the second, and the windows are below both. The guard sweep in 08-07 is unaffected: no record was written or re-measured and the census is unchanged.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-14*
