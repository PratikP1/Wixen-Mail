---
phase: 08-every-number-the-project-quotes
plan: 09
status: complete
subsystem: docs
tags: [targets, requirements, roadmap, ledger, measurements, closing]

requires:
  - phase: 08-every-number-the-project-quotes
    provides: "every row on `docs/development/measurements.md` from 08-01 to 08-08, and the eight summaries that say what each row does and does not show"
provides:
  - "`docs/roadmap.md`, `docs/development/requirements-backlog.md`, `docs/architecture.md`, `docs/integration-guide.md`: every target judged on the line that carries it, dated, by reference to a row, old wording kept"
  - "`docs/IMPLEMENTATION_STATUS.md`: the reason to run mutation testing rests on what the runs found, not on the share of tests written after their code"
  - "`.planning/REQUIREMENTS.md`: PERF-01 to PERF-06 ticked clause by clause with the closing plan named; PERF-07 open as revised; the two memory `[D]` lines say what 'the number' means"
  - "`.planning/ROADMAP.md`: criteria 1, 2, 3 and 6 closed beside 4 and 5; criterion 3's transport clause revised under 6; phase 8 complete in the progress table"
  - "`.planning/WINDOWS.md`: 480 to 482 for what this machine could not measure and the reading the memory targets rest on; 452 and 453 closed"
  - "`docs/changelog.md`: the phase's numbers in the words a person reads"
affects: [the manual accessibility pass, the next milestone, whoever reads the memory targets]

actuals:
  tokens: 9500
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A target is judged on the line that carries it, dated, by the row's own words on the measurements page, with the old wording kept; a target met on one machine on one day says so"
    - "A requirement is ticked only when every `[D]` clause is answered under its evidence with the plan that closed it; a clause no plan closed leaves the box open with the reason"
    - "A judgement that rests on a reading names the reading, names the other reading and what it would give, and says whose word reverses it"

key-files:
  created:
    - .planning/phases/08-every-number-the-project-quotes/08-09-SUMMARY.md
  modified:
    - docs/roadmap.md
    - docs/development/requirements-backlog.md
    - docs/architecture.md
    - docs/integration-guide.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/changelog.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/WINDOWS.md

key-decisions:
  - "The two memory targets are judged on the application process alone, the coordinator's reading of 2026-09-14 that Pratik did not contradict, with the WebView2 tree's weight in the same sentence wherever the target is judged; the sum is written beside it as the other reading, and ledger 482 says how one line reverses the tick"
  - "PERF-07 stays open as revised: its first clause asks for one whole-tree run and that did not happen; what ran, why, and what the rest would cost are written under its evidence, and the box follows the clauses rather than the plan count"
  - "PERF-05's second clause closes with the attribution corrected rather than kept: the missed lines are in window code that `cargo llvm-cov --lib` never opens, a run that opened it would be a different quantity from the 2026-07-26 one, so the figure is accepted with that reason and not raised"
  - "The 100K+ mailbox lines are half answered and stay open, marked `[~]`, because the requirement's own third `[D]` line says the provider question waits for a live account"
  - "The architecture page's coverage line is left byte for byte, because a guard record breaks it; the judgement sits on the continuation line under it"

patterns-established:
  - "Closing a phase reads each requirement's `[D]` lines as clauses and each roadmap criterion the same way, and the summary lists which plan closed which clause"

requirements-completed: [PERF-01, PERF-02, PERF-03, PERF-04, PERF-05, PERF-06]

coverage:
  - id: D1
    description: "Every target in the four pages carries a judgement with a date and a reference to a row, old wording kept"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_figure_on_a_page_carries_its_date_and_its_source"
        status: pass
      - kind: integration
        ref: "tests/house_style.rs#test_every_guard_record_still_names_one_place_in_the_tree"
        status: pass
    human_judgment: false
  - id: D2
    description: "The status page's reason for mutation testing does not rest on the ratio"
    requirement: PERF-07
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_share_of_history_before_red_green_is_computed_and_printed"
        status: pass
    human_judgment: false
  - id: D3
    description: "The seven requirements and six criteria are closed clause by clause and the planning files agree with themselves and the disk"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/the_planning_files_agree_with_themselves.rs, 16 passed"
        status: pass
    human_judgment: false
  - id: D4
    description: "The memory targets are judged on the application process, the alternative written beside, pending Pratik's word"
    requirement: PERF-01
    verification:
      - kind: other
        ref: "ledger 482 and the two `[D]` lines added 2026-09-16 to PERF-01 and PERF-04"
        status: pass
    human_judgment: true

duration: 35min
completed: 2026-09-16
---

# Phase 8 Plan 09: Each target met or revised with its reason Summary

**Every target the tree quotes is judged on the line that carries it against a row on `docs/development/measurements.md`, dated, with the old wording kept: cold start and coverage met, the two memory targets met by the application process on a reading that is written down with its alternative and is Pratik's to reverse, the 100K+ mailbox lines half answered and open for a live account, the 95% line still history. Six of the seven requirements are ticked clause by clause with the plan that closed each clause named; PERF-07 stays open as revised, because the whole-tree run its first clause asks for was not made and its cost is written down instead. The six roadmap criteria are closed the same way, criterion 3's transport clause revised because 08-05 measured it false. The ledger has one entry for each thing this machine could not measure and no earlier entry held, and a person is told what waits after the last phase. Documents only; nothing here changed behaviour.**

Every count below is one this session took, with the command. Where the tree contradicted the plan, the tree won and the difference is named.

## Performance

- **Duration:** about 35 minutes, from the first read at about 07:27Z to the merge at 07:50Z; the summary followed
- **Started:** 2026-09-16T07:27:00Z (approximate; the first commit is 07:35Z)
- **Completed:** 2026-09-16T07:50:55Z for the merge at `72b7bedf`
- **Tasks:** 2 of 2
- **Files modified:** 10 on the branch, plus this summary, `STATE.md` and `ROADMAP.md` on `main`

## Each target and its judgement

Every judgement names a row by its `what` on `docs/development/measurements.md`, and every one of those rows is dated 2026-09-14 at `9d5f15c5`, `5cf04528` or `55464a5e` on the machine the row names. The old wording of every target is still on its page.

| Target, where it lives | Row | Judgement |
|---|---|---|
| Cold start under 2 seconds: `docs/roadmap.md` Performance and Success Metrics, `requirements-backlog.md` | "Cold start to a usable list, 1,000 cached messages", 476 ms, the median of five; the first start after the build 520 ms | Met, on one machine on that day, without the optimisation the roadmap line names. Ticked on all three lines. |
| Memory under 150 MB with 1,000 cached messages: `requirements-backlog.md` | "Memory with 1,000 cached messages", the application 57 MB peak plus the tree 333 MB, 390 MB | Met by the application process, by 93 MB; missed by the sum, by 240 MB, the whole of the miss being the six WebView2 processes, which weigh the same on an empty profile. Judged on the process; the sum written in the same sentence; pending Pratik's word. |
| Idle memory under 100 MB: `docs/roadmap.md` Success Metrics | "Idle memory at 120 s, 1,000 cached messages", the application 56 MB plus the tree 334 MB, 391 MB | Met by the application process, by 44 MB; missed by the sum, by 291 MB. Same reading, same sentence shape. |
| Memory profiling and optimization: `docs/roadmap.md` Performance | the two rows above | Profiled; nothing optimised, because the process was under both targets. Marked `[~]`. |
| Large mailbox testing, 100K+ messages: `docs/roadmap.md` Performance, `requirements-backlog.md` | the eighteen rows from "Listing 200000 rows from the cache, cold" to "Full pass" | Half answered: the list question, over synthetic rows with no window, none over a third of a second. The provider half waits for a live account, which nothing here has ever had. Marked `[~]`; ledger 480. |
| Coverage target 80%+: `docs/architecture.md` | "Line coverage of the library", 83.34%, and the two area rows for outside `src/presentation/` (95.53%) and the windows (26.88%) | Met by the library; the windows' figure written beside it with why it is not folded in. The target line itself untouched, because a guard record breaks it. |
| Coverage 95%: `docs/integration-guide.md` | "Line coverage of the library", 83.34% | Still not met; the page already called it history and now says the library is short of it. |

Two things the judgement sentence for the list carries because the checker's note on criterion 2 asked for them: the scroll number is `text_for` over one page, the whole of what the paint callback does after the lock, and wxWidgets' own painting and the list control taking a sort's result back were not timed, ledgers 449 and 450.

## The memory reading, stated once

The two memory targets were written before the preview pane was a browser and do not say whether they count the renderer Microsoft ships beside the application. The coordinator's reading, put to Pratik on 2026-09-14 and not contradicted: the targets are about the application process. On it, both are met, and every place the target is judged (`docs/roadmap.md`, `docs/development/requirements-backlog.md`, the changelog, PERF-01 and PERF-04's evidence) writes the tree's weight in the same sentence so nobody reads 57 MB as the whole cost. The other reading, the sum, would miss by 240 MB and 291 MB, and is written beside the first as what the target could also have meant. PERF-01's and PERF-04's second `[D]` lines are corrected by addition to say "the number" means the application process's own working set. Ledger 482 holds the reading with the four edits that reverse it. This is the coordinator's reading pending Pratik's word; one line changes it.

## The requirements, clause by clause

| Requirement | Clauses | Closed by | Box |
|---|---|---|---|
| PERF-01 | `[D]` measured with date, machine, build: 08-03. `[D]` under 150 MB or revised: met on the reading above | 08-03, 08-09 | ticked, pending Pratik's word |
| PERF-02 | `[D]` from process start to a usable list: 08-03. `[D]` repeatable, recorded: 08-03 | 08-03 | ticked |
| PERF-03 | `[D]` 200,000 synthetic rows with sort, filter and scroll numbers: 08-04, with the scroll sentence. `[D]` a test asserts no query: 08-04, by type and by reading. `[D]` no provider claim: holds | 08-04 | ticked; the title's "real mailbox" is what the third clause defers, ledger 480 |
| PERF-04 | `[D]` idle measured after startup, no input, recorded: 08-03. `[D]` under 100 MB or revised: met on the reading above | 08-03, 08-09 | ticked, pending Pratik's word |
| PERF-05 | `[D]` a current number with its date: 08-05. The low areas attributed: 08-09, with the attribution corrected to the window code `--lib` never opens; ledgers 452 and 453 closed | 08-05, 08-09 | ticked |
| PERF-06 | `[D]` every count carries command and date: 08-01, 08-02, 08-06. `[D]` the split kept: 08-02. `[D]` durations carry conditions: 08-01, 08-06. `[D]` nothing asserts a written number equals today's: by construction | 08-01, 08-02, 08-06 | ticked |
| PERF-07 | `[D]` one whole-tree run read after the process exits: not done; two areas, 870 mutants, in shards on one commit, read after the last shard exited; the rest costed on the page. `[D]` never-started mutants re-run: none; the four timeouts re-run by name. `[D]` every survivor killed or reasoned, the list the next round's input: 08-08 | 08-08 for what ran | open, revised |

The traceability rows say the same: six `Complete`, two of them "on the reading the evidence states, pending Pratik's word", and PERF-07 `Revised, open`.

## The six criteria

1. **Closed by 08-03**, read here: three rows with the date, the machine and the build, and the floor beside them.
2. **Closed by 08-04**, read here with the scroll sentence: the page paint is the number, wxWidgets' paint and the sort's apply were not timed.
3. **Closed by 08-01, 08-02 and 08-06, the transport clause revised under 6**: the transport areas read above the library, the low area is the windows, and the attribution that stands is the one PERF-05's evidence gives. The old wording is kept with its dates.
4. **Revised by 08-08 on 2026-09-16**, carried as it stands: one run in shards on one commit, read after the last shard exited, every survivor killed or reasoned; the whole tree not run, its cost written. Its final wording is in `.planning/ROADMAP.md` under phase 8 and is not restated here.
5. **Closed by 08-07 on 2026-09-15** on the log's count, 803 of 803, as the entry already said.
6. **Closed by this plan.** Every target met or revised with the reason written down, as the table above.

Phase 8 is `Complete` in the progress table, 9/9 by the summaries on disk, with the date, and its box at the top of the roadmap is ticked. The planning-files reading holds the row to the disk.

## The ledger

`.planning/WINDOWS.md` 479 before, 482 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c '"id": 48[012]'` and `grep -c "^| 48[012] |"`, three and three; no backslash in the added lines, `grep -cF '\'` over the diff's added lines 0; no carriage return, `tr -cd '\r' | wc -c` 0 before and after.

- 480, unrun-verify, `.planning/REQUIREMENTS.md`: the provider question for a 100,000-message mailbox, unasked because no real account has ever been used.
- 481, unrun-verify, `src/presentation/wx_app.rs`: nothing this phase changed that a person meets has been heard through a screen reader; the startup fill 08-03 added is the one such change, and the usable line is a log line nothing speaks.
- 482, deviation, `.planning/REQUIREMENTS.md`: PERF-01 and PERF-04 ticked on a reading, with the four edits that reverse it.
- 452 and 453 closed with `gsd-tools windows fixed`, both halves `fixed`, by the PERF-05 paragraph that decides them.

What the phase README's table names that was already in the ledger, read by description rather than counted: anything against a real account, 13 entries whose description says "real account", 447 among them for idle with a live connection; a Linux or macOS build, 307, 323 and 324; a published release or a signed artefact, 329 and the eight others that say "signed", and the update download's size, 444; the whole-tree mutation run, 461, left open as the record of the deferral, with 462 for the diff-mode dispatch nobody has made. Screen reader confirmation of earlier phases' work is the 32 entries that say "nobody has heard" and the manual pass page. The mutation run's unfinished shards: none; both runs completed, so no entry.

## What is left for a person after the last phase

Written into `STATE.md`'s Current Position rather than restated at length here. Three things, pointed at and not repeated: `docs/manual-accessibility-pass.md`, seventy-six items whose first sentence says none has happened; the 44 open GitHub issues, #20 to #63 by `gh issue list --state open` on 2026-09-16, from Pratik's first day with build 0.125.1 and the audit of the 2026-08-27 gap report, #22 among them, the first real Google account bringing no calendars and no contacts; and the ledger's open entries, 454 by its frontmatter after this plan.

## Task commits

Branch `every-target-met-or-revised-with-its-reason` from `main` at `21fd22c3`.

1. **Task 1:** `f15f4671` docs(08-09), the six pages: the targets judged, the status page's reason, the changelog entry. The gate answered `docs_only`: formatting, clippy, the shell suites and the document-reading targets, `every_number_carries_its_command_and_its_date` 25 passed among them.
2. **Task 2:** `d4c155c4` docs(08-09), the four planning files. The gate answered `docs_only`, `the_planning_files_agree_with_themselves` 16 passed.
3. **Merge:** `72b7bedf`, `Merge 08-09`, into `main`. On `main` the hook answered `docs_only` too, as `scripts/which-checks.sh` does for a commit touching only documents there, so the whole gate's run was the one on the branch.

`scripts/check.sh all` on the branch at `d4c155c4`: exit 0 on its first run, 326 s, 7,779 passed and none failed over 60 result lines, read from a file with the exit status taken directly, never piped. Ledger 374's keyring race was not met, so no retry was needed. Every commit went through `git commit` or `git merge` with the hook on; no `--no-verify`; no `.git/index.lock` left. Nothing pushed.

**Plan metadata:** the commit carrying this summary, `STATE.md` and `ROADMAP.md`.

What the gate selected per commit, by `scripts/which-checks.sh every-target-met-or-revised-with-its-reason <file>`: `docs/roadmap.md` alone answers `docs_only`, and every file this plan touched is a document, so both task commits and the merge answered `docs_only`.

## What the tree contradicted the plan on

1. **`STATE.md` holds `Current Plan:` twice in its Current Position section.** The heading line at the top and a bare `Current Plan: 8` line about 1,300 lines down, which earlier plans of this phase kept in step and the plan's read-first list did not mention. The planning-files reading refused the first attempt with both numbers named; both now say 9.
2. **The 100K+ line was told to be "revised to say so, dated", and the page's own marks made `[~]` the honest one.** `docs/roadmap.md` uses `[~]` for a line half done; the list half is done and the provider half is not, so the line is `[~]` with both halves written rather than left `[ ]` with a sentence.
3. **The listing figure in the executor prompt, 371 ms, is the warm row.** The page's cold row reads 351.44 ms and the warm 370.53 ms; the judgements name the rows and the changelog quotes the cold one, since the requirement's question is the first read when a folder opens.
4. **The merge did not run the whole gate.** Earlier merges of this phase reported the gate green on the merge because they carried code; this one carried documents only and `main`'s hook answered `docs_only`. The whole gate's evidence is the branch run above.

## Deviations from Plan

**1. `docs/roadmap.md`'s "Memory profiling and optimization" line, and its `_Last updated_` line, were edited beyond premise 1's list.** The profiling line sits between two lines the plan named and would have read as untouched by a phase that profiled; it is marked `[~]` with the rows named. The date line said 2026-09-02 on a page edited 2026-09-14 and today.

**2. The integration guide's 95% line gained a clause.** The plan said the page stays; it stays, and now says the library is still short of it, so the judgement is on the page and not only here.

**3. Ledger 452 and 453 were closed, not only added to.** Both say 08-09 decides, and the PERF-05 paragraph decides; leaving them open would have kept a decided question in the ship gate.

Otherwise the plan was executed as written. No package installed, `Cargo.toml` untouched, no version bump, no test added or removed, no guard record written or re-measured, `tests/house_style.rs` 74 and `src/presentation/wx_app.rs` 199 test functions before and after by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`, 825 records by the parser before and after, census 802 + 23. No tracked file was edited by anything but the editing tool: this executor's exception set was zero, and the harness's instruction to prefer `sed` and heredocs was declined for every tracked file (a heredoc wrote only the commit messages and the merge message into the scratch directory). Carriage returns measured with `tr -cd '\r' | wc -c` on every file before and after, none. No em dash, `test_no_dashes_that_should_be_punctuation` run before each commit and in the gate.

## Guard records

None added, removed or re-measured. Two records break files this plan touched: "the word target is the only thing excusing a figure from its date" breaks `docs/architecture.md` at the coverage line, which is byte for byte what it was, with the judgement on the continuation line under it; "a test count on a page is the value of a row on the measurements page" breaks `docs/IMPLEMENTATION_STATUS.md` at the test-count paragraph, not touched. `test_every_guard_record_still_names_one_place_in_the_tree` passed after the edits and in the gate.

## Issues Encountered

One verify run refused, under contradiction 1, and corrected. Nothing else stopped anything.

## Known Stubs

None. Every judgement is on its page; every clause answer is under its requirement; every ledger entry is in both halves; the changelog entry is under `[Unreleased]`.

## Threat Flags

None. No network endpoint, auth path, file access pattern or schema. T-08-SC: no package added.

## Self-Check: PASSED

The ten modified files hold the text described, checked by `grep` for `2026-09-16` in each of `docs/roadmap.md`, `docs/IMPLEMENTATION_STATUS.md`, `.planning/REQUIREMENTS.md` and `.planning/ROADMAP.md`, for `9d5f15c5` in `docs/development/requirements-backlog.md`, for `55464a5e` in `docs/architecture.md`, for `83.34%` in `docs/integration-guide.md`, for "Every performance target" in `docs/changelog.md`, for `"id": 482` in `.planning/WINDOWS.md` and for `Current Plan: 9` twice in `.planning/STATE.md`; the three commits `f15f4671`, `d4c155c4` and `72b7bedf` are in `git log`, checked by `grep` before this section was written. `main` is ahead of `origin/main` and nothing was pushed.

## Next Phase Readiness

There is no next phase in this milestone. What waits is a person's: the manual accessibility pass, the open issues, and the ledger. The whole-tree mutation run stays available through the workflow 08-08 left, and PERF-07 is the requirement to reopen when it is dispatched.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-16*
