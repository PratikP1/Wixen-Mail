---
phase: 08-every-number-the-project-quotes
plan: 08
status: partial
subsystem: testing
tags: [mutants, shards, measurements, rate, tooling, checkpoint]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "the mutant count row and the two suite rows on `docs/development/measurements.md`, and the mutation timeout comment in `.cargo/mutants.toml` that says the multiplier is read against a baseline the tool takes per run"
  - phase: 08-every-number-the-project-quotes
    plan: 07
    provides: "the pattern for a launcher that waits for a quiet machine and survives a closed window, the `Start-Process` shape that worked, and the sweep worktree this plan moved"
provides:
  - "`scripts/mutants.sh --shard k/n [--out DIR] [--in-place] [-- ...]` and `--shards n`: each shard to its own directory with `conditions.txt` written before the run and `timing.txt` after; the launcher skips complete shards, waits for a quiet machine, skips the baseline after the first complete shard with the config's own timeout, refuses a modified tree in place, and stops on a shard that did not complete"
  - "`scripts/mutants_report.py --shards DIR N`: every shard merged into one `Run` through the same refusals as a single run, with a missing, partial, moved or differently committed shard refused by name; `--complete`, `--timing`, `--timeout-after`, `--tree-is-clean` and `--wait-until-quiet` for the launcher; 83 worked examples where there were 47"
  - "Four rate rows, a fixed-cost row and two product rows on `docs/development/measurements.md`, all of 2026-09-15 at `2847391c`"
  - "Both worktrees, `../wixen-mail-sweep` and `../wixen-mail-mutants`, at `main` as it stands after this task, `1401e4d3`, built and clean"
  - "The checkpoint below, usable without the plan"
affects: [08-08 tasks 3 and 4, 08-07 task 3, 08-09]

actuals:
  tokens: 10875
  tasks: 1
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A long job is a set of units on one commit: each unit writes what it ran under before it starts and its cost after, in its own directory, and the merge refuses a unit that cannot be shown to belong with the others"
    - "A cost with two terms that scale with different counts is quoted as two terms, and the product shows both multiplications"
    - "A launcher for a long external job is rehearsed on a stand-in project that finishes in seconds before its first real run, which is where the tool's real flag matrix and record shape are read"

key-files:
  created: []
  modified:
    - scripts/mutants.sh
    - scripts/mutants_report.py
    - docs/development/measurements.md
    - docs/changelog.md
    - guards/guards.toml
    - .planning/phases/08-every-number-the-project-quotes/08-07-SUMMARY.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
    - .planning/WINDOWS.md

key-decisions:
  - "The every-target shape is measured and offered as `-- --all-targets`, the suite as the gate runs it, and in place, because a scratch copy has no `.git` and the share-of-history test fails its baseline there; the plan's every-target scratch run cannot exist at this tree"
  - "The commit and argument comparison lives in the merger with worked examples, and the shell writes the record before the run starts, so a shard killed partway still says what it was"
  - "A shard after the first complete one skips the baseline and is passed `--timeout` from the config's rule over that shard's baseline, because the tool's fallback without a baseline is 300 s whatever the suite takes"
  - "`--in-place` drops `-j`, because the tool refuses the pair even at one, and refuses `MUTANTS_JOBS` above one before the conditions are written"
  - "An in-place shard refuses a tree with a tracked file modified, added after the merge on its own branch, because the checkpoint tells Pratik to kill the run whenever he needs the machine and a kill mid-mutant leaves the mutant behind with the baseline skipped"
  - "The fixed term is wall time minus the mutants' recorded phase durations, so it holds the copy, the build, the baseline and the tool's bookkeeping between mutants; the split is exact for the shard and slightly overstates the fixed term, which is the safe direction"

patterns-established:
  - "A row's product shows both multiplications, `count x rate + shards x fixed`, so the term that scales with the shard count is visible and the copy setting is a decision with a number beside it"

requirements-completed: []

coverage:
  - id: D1
    description: "Shards that can be run one at a time, restarted, and merged as one run with every refusal a worked example"
    requirement: PERF-07
    verification:
      - kind: integration
        ref: "tests/house_style.rs#test_the_mutation_report_still_obeys_its_own_examples, 83 examples in 63 items"
        status: pass
      - kind: other
        ref: "a three-shard run, a kill-and-restart, and every merger refusal on a two-function crate in the scratchpad; the four rate shards in the worktree; the lone-shard refusal quoted below"
        status: pass
    human_judgment: false
  - id: D2
    description: "The rate one mutant costs on this machine under both suite shapes, the fixed cost per shard, and the products over the day's count"
    requirement: PERF-07
    verification:
      - kind: other
        ref: "the four `timed:` lines quoted below and the rows on docs/development/measurements.md, read by tests/every_number_carries_its_command_and_its_date.rs"
        status: pass
    human_judgment: false
  - id: D3
    description: "Which run to make, with the products in front of the person whose machine it is"
    requirement: PERF-07
    verification:
      - kind: manual_procedural
        ref: "the checkpoint below; Pratik chooses and starts it"
        status: unknown
    human_judgment: true
    rationale: "Weeks of his machine; the plan makes it his decision"

duration: 4h10min
completed: 2026-09-15
---

# Phase 8 Plan 08: A mutation run in shards a person can start, stop and read as one Summary

**A whole-tree mutation run is now a set of shards on one commit: `scripts/mutants.sh --shards n` runs them in turn, skips the complete ones, waits for a quiet machine, and a kill loses one shard; `scripts/mutants_report.py --shards DIR n` reads every shard as one run and refuses a missing, partial, moved or differently committed shard by name. The rate was measured on one shard of 25 mutants under both suite shapes, four runs in a worktree, and the products are 17.5 days for the library at eight threads and 19.8 days for every target, both in place, both on `docs/development/measurements.md` as two terms. Which run to make is Pratik's, from the checkpoint below. Nothing was started.**

Task 1 of 4. Task 2 is the checkpoint. Tasks 3 and 4 are not attempted.

## Performance

- **Duration:** about 4 hours 10 minutes, from the first read at about 04:35Z to the second merge at 08:38Z, of which about 3 hours 20 minutes were the four rate shards running with nothing else building
- **Started:** 2026-09-15T04:35:00Z
- **Completed:** task 1 merged at `99682439`, its second half at `1401e4d3`; the plan is partial
- **Tasks:** 1 of 4
- **Files modified:** 5 on the two branches, plus this summary, 08-07's summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md` on `main`

## What the scripts can now do

| Flag | What it does |
|---|---|
| `--shard k/n` | One shard of every mutant the configuration allows, to `DIR/shard-k-of-n/`. Before the run, `conditions.txt` with the commit, the shard, everything after `--`, `copy = scratch` or `in-place`, and `baseline = run` or `skip`. After it, the `timed:` line printed and written to `timing.txt`, then the report as before. Exits with the report's status. |
| `--shards n` | Shards 0 to n-1 in turn. A shard whose `outcomes.json` holds as many outcomes as `mutants.json` declares is skipped with a line saying so. Before each: `--wait-until-quiet`, the same poll as the guard runner's, and for an in-place run `--tree-is-clean`. After the first complete shard, every shard is passed `--baseline skip --timeout N`, N from the config's multiplier and floor over that shard's baseline test time. A shard that is not complete after its run stops the launcher with the reason above it and the sentence that a restart picks up at that shard. The last line when nothing remains is `Every shard is complete: n of n.` and the merge command. |
| `--out DIR` | Where the shard directories go; default `target/mutants`. Two measurements of one shard under different arguments do not overwrite each other. |
| `--in-place` | The tool mutates the tree the script runs in rather than a copy, so the build is incremental. No `-j`, because the tool refuses the pair. Refused over a tree with a tracked file modified, naming the file and the `git checkout`. |
| `-- ...` | Everything after goes to `cargo test`; after a second `--`, to the test binary. |

`scripts/mutants_report.py` gained `--shards DIR N` and, for the launcher, `--complete OUT_DIR`, `--timing OUT_DIR STARTED FINISHED`, `--timeout-after OUT_DIR`, `--tree-is-clean` and `--wait-until-quiet`, the last reusing `guards.wait_until_quiet`. The pure parts, each with worked examples: `conditions_from`, `why_these_shards_are_not_one_run`, `one_run_from`, `a_shard_is_complete`, `timing_of`, `the_timing_line`, `timeout_for_a_skipped_baseline`, `why_an_in_place_shard_is_refused`, and two small ones, `listed` and `arguments_in_words`.

## Red first

The functions were written as docstrings with worked examples and no bodies. `python -m doctest scripts/mutants_report.py`: 22 of the new examples failed against the bodiless functions (the passing ones being definitions, assignments and an `is None` case). `cargo test --test house_style test_the_mutation_report_still_obeys_its_own_examples` was red on it: `test result: FAILED. 0 passed; 1 failed`. The red commit's trailer names the bare name, `Fails-until-green: test_the_mutation_report_still_obeys_its_own_examples`, as 08-07 found the gate requires; the gate confirmed "a red that is exactly the one this commit named." The second branch's red was the same shape, one example failing, the same test red.

`python -m doctest -v scripts/mutants_report.py`: **47 tests in 25 items** before this plan, **83 tests in 63 items** after, 36 more where the criterion asked for five. The green commit `2847391c` quotes the item count as 57, which was the count before the launcher's command-line helpers were added to `main`; the test count it quotes, 81, was right.

## The four timing lines

Every run was `scripts/mutants.sh --shard 0/496` in `../wixen-mail-mutants` at `2847391c`, with `tasklist` finding no `cargo.exe` or `rustc.exe` before each start, started detached from PowerShell with `Start-Process` for the last three. `cargo mutants --list | wc -l` in that worktree read **12,391** that day, so 496 shards of 24 or 25. Shard 0 is the first 25 mutants of the tool's list, all in `src/presentation/accessibility.rs`.

**Run 1, every target in a scratch copy, `--out target/mutants-rate-all`, nothing after `--`:** refused at the baseline. The tool copies the tree without `.git`, and `test_the_share_of_history_before_red_green_is_computed_and_printed` runs `git merge-base`, which answered `fatal: not a git repository`. The report said `The build failed before anything was changed, so none of the 25 mutants was tested.` and `It exited 4`. Its timing line still measured the copy's fixed term for that shape:

```
timed: 439 s before and between the mutants (the copy, the build and the baseline, 353 s build + 76 s test), then no mutant was tried, so there is no rate
```

**Run 2, the library at eight threads in a scratch copy, `--out target/mutants-rate-lib -- --lib -- --test-threads=8`:**

```
timed: 402 s before and between the mutants (the copy, the build and the baseline, 337 s build + 52 s test), then 2889 s over 25 mutants, 24 answered by the suite, 116 s a mutant
```

**Run 3, the same in place, `--out target/mutants-rate-lib-in-place --in-place`:**

```
timed: 97 s before and between the mutants (the copy, the build and the baseline, 39 s build + 54 s test), then 2958 s over 25 mutants, 24 answered by the suite, 118 s a mutant
```

**Run 4, every target in place, `--out target/mutants-rate-all-in-place --in-place -- --all-targets`:**

```
timed: 194 s before and between the mutants (the copy, the build and the baseline, 71 s build + 122 s test), then 3241 s over 25 mutants, 24 answered by the suite, 130 s a mutant
```

In every run that reached its mutants: 23 caught, 1 nothing noticed (`src/presentation/accessibility.rs:339:9: replace Accessibility::announce_what_was_typed -> Result<()> with Ok(())`, the same one each time), 1 the compiler rejected with exit 101, 0 timed out, **0 never started** in any shard. The tool's own `Auto-set test timeout` lines read 261 s, 272 s and 611 s, and `timeout_for_a_skipped_baseline(52.2, 5.0, 60)` gives 261, so the launcher and the tool apply one rule. Each in-place run left the worktree clean by `git status --porcelain`.

Two things the record showed that the plan's inference did not. Each mutant in `accessibility.rs` rebuilt 58 to 75 s, where the guard rate row's one-file rebuild in `managers.rs` reads 44 to 46 s, because most of the presentation layer depends on it; the rate is a rate for a file at the heavy end. And under every target a caught mutant tested for 61 to 65 s, not 122, because `cargo test` stops at the first failing target and the library catches it first; only the missed one paid 119 s. So the every-target shape is 13 percent over the library shape, not the weeks apart the plan inferred from the two suite rows.

## The products

On `docs/development/measurements.md`, dated 2026-09-15 at `2847391c`, each as two terms:

| Run | Product |
|---|---|
| The library at eight threads, in place | 12,391 x 118 s + 496 x 97 s = 1,462,138 s + 48,112 s = 1,510,250 s, about 420 hours, **about 17.5 days** |
| Every target, in place | 12,391 x 130 s + 496 x 194 s = 1,610,830 s + 96,224 s = 1,707,054 s, about 474 hours, **about 19.8 days** |
| The library in a scratch copy, for comparison | 12,391 x 116 s + 496 x 402 s = 1,437,356 s + 199,392 s = 1,636,748 s, about 18.9 days |

`--in-place` saves about a day and a half over 496 shards, all of it in the fixed term. The rows say the rate is from one file at the heavy end and no leaf module was measured.

## The merger's refusal on the lone shard

`python scripts/mutants_report.py --shards target/mutants-rate-lib 496` in the worktree, with only shard 0 there:

```
Shard 1/496 is missing: there is no target\mutants-rate-lib\shard-1-of-496\conditions.txt.
Every shard of the run has to be there before the run is read.
```

exit 1. On the scratch crate, with three real shards: a merge over them reported 10 mutants as one run; the same with `4` refused shard 0/4 as missing; a shard whose `outcomes.json` was cut to one outcome was refused as `Shard 2/3 stopped after 1 of its 2 mutants`; and a shard whose `conditions.txt` was edited to another commit was refused as `Shard 1/3 ran at deadbeef... and shard 0/3 at af9cfd7a..., so they are two lists and not one run.`

## Guard record

One record, "shards from two commits are not read as one run", `file = scripts/mutants_report.py`, `suite = house_style`, the break turning `if said.commit != first.commit:` into a comparison of a value with itself. Measured on the target through `scripts/guards.sh --remeasure`: `the one test named went red, and nothing else did`, `test_the_mutation_report_still_obeys_its_own_examples`. Census at `guards/guards.toml:82-83` 192 + 607, 799 by the TOML reader, 798 before. `tests/house_style.rs` holds 70 test functions before and after, so the count check printed no remedy.

## What the gate selected for each file

By `scripts/which-checks.sh one-shard-at-a-time-and-the-rate-before-the-run <file>`: `scripts/mutants.sh` alone `affected`, `scripts/mutants_report.py` alone `affected`, `guards/guards.toml` alone `affected`, `docs/changelog.md` alone `docs_only`, `docs/development/measurements.md` alone `docs_only`; all five on `main`, `all`. Neither script maps to a target of its own; `scripts/check.sh --suites-for scripts/mutants.sh scripts/mutants_report.py` prints nothing, for the reason ledger 442 records. What reaches `mutants_report.py` is `house_style`'s doctest runner and what reads `mutants.sh` as text is `test_no_mutation_result_is_read_from_the_lists_written_for_people` and `test_no_mutation_run_has_its_failure_swallowed`, all in `guards_that_read_the_whole_tree`, which the hook ran on every commit. The page is read by `tests/every_number_carries_its_command_and_its_date.rs`, which refused the two product rows once for a command cell with no backticked token, corrected before the commit.

## Task commits

Branch `one-shard-at-a-time-and-the-rate-before-the-run` from `main` at `d53893b7`:

1. **Red:** `6dd3e85e` test(08-08), the examples on bodiless functions, trailer naming the doctest-running test.
2. **Green:** `2847391c` feat(08-08), the bodies, the shell modes, the changelog entry.
3. **Documents:** `6497d610` docs(08-08), the rows and the record.
4. **Merge:** `99682439`, `Merge 08-08 task 1`, alone into `main`.

Branch `an-in-place-shard-refuses-a-tree-somebody-left-broken` from `main` at `99682439`:

5. **Red:** `d6ef92e9` test(08-08), one example for the refusal.
6. **Green:** `9bcad4af` feat(08-08), the body, `--tree-is-clean`, the check before every in-place shard, the changelog sentence.
7. **Merge:** `1401e4d3`, `Merge 08-08 task 1, second half`.

`scripts/check.sh all` on each branch, exit 0 on its first run both times: 329 s and 323 s, 7,746 passed and none failed over 58 result lines; on each merge, 308 s and 327 s, the same. Ledger 374's keyring race was not met. Every commit went through `git commit` or `git merge` with the hook on; nothing was piped; the gate's output went to a file and its exit status was read directly; no `.git/index.lock` was left. Nothing is pushed.

**Plan metadata:** the commit carrying this summary, 08-07's corrected summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md`.

## The checkpoint: which run to make, then start it on an idle machine

This is written to be acted on without the plan.

**What it is.** One whole-tree mutation run over the 12,391 mutants the configuration allows, or a part of it, in shards of 25, each shard a unit that can be run, stopped and read alone, all at one commit. Every figure below was taken on 2026-09-15 on this machine, on one shard of `src/presentation/accessibility.rs`, a file at the heavy end of what a rebuild costs; a leaf module would come in under these and none was measured. The mutation worktree, `../wixen-mail-mutants`, is at `main` as it stands after this task, `1401e4d3`, built and clean, and every command below runs in it.

**The four options, with what each costs.**

| Option | What runs | The cost, both terms | What it gives up |
|---|---|---|---|
| 1. The whole tree, every target | every mutant, `cargo test --all-targets` as the gate runs it, in place | 12,391 x 130 s + 496 x 194 s = 1,707,054 s, **about 19.8 days** of the machine | nothing in the judgement. This shape only runs in place: a scratch copy has no `.git` and the baseline fails in it |
| 2. The whole tree, the library at eight threads | every mutant, `-- --lib -- --test-threads=8`, in place | 12,391 x 118 s + 496 x 97 s = 1,510,250 s, **about 17.5 days** | the targets under `tests/` never run against a mutant, so a mutant only they would catch is filed as a survivor, and task 3 has to reason about it as one |
| 3. A scoped run, and criterion 4 revised | one area with `--lib`, through the older `scripts/mutants.sh <dir>` mode, one process | at 118 s a mutant plus 97 s a shard of 25: `src/common` 400 mutants, about 13.5 hours; `src/service/protocols` 450, about 15.2 hours; `src/data` 1,386, about 2.0 days; `src/presentation` outside the windows 1,886, about 2.7 days; the rest of `src/service` 3,306, about 4.7 days; `src/application` 4,963, about 7.0 days. Counts from `cargo mutants --list -f` on 2026-09-15 at `1401e4d3`; they sum to 12,391 | the shard modes pass nothing to `-f`, so a scoped run today has no resume; if this is chosen, the executor adds `--file` to `--shard` and `--shards` first, red first, about an hour. Criterion 4 is revised under criterion 6 with this table as the reason and PERF-07 stays open |
| 4. A combination | option 3 now, one of 1 or 2 across the weeks after | the scoped answer's hours now and the whole product later | the same as option 3 until the whole run starts |

The plan said the two suite shapes would differ by weeks; they differ by 13 percent, because `cargo test` stops at the first target that fails and the library catches most mutants before any target under `tests/` runs. Every mutant that survives the library then pays the full every-target suite, so a tree with more survivors than this shard's one in 25 costs more than the product says.

**Recommendation: option 1, every target in place, started after the guard sweep of 08-07 has finished.** The plan recommended option 2 on the premise that every target cost weeks more; measured, it costs about 2.3 days more and buys the complete judgement. The expensive part of this run is not the machine, it is the reading afterwards: at one survivor in 25, the whole tree yields about 500 lines for task 3 to reason about, and option 2 adds to those every mutant that the integration targets would have caught, which somebody then has to recognise as tested rather than untested. Option 1's timeout is also 611 s against 272 s, so a busy minute is less likely to file a hang. The sweep goes first because it is a day and this is weeks, and neither may run beside the other: both launchers poll for a quiet machine between units, and two of them polling would take turns rather than overlap, which is slower than either alone.

**Where.** The worktree already exists and is built:

```
git worktree add --detach ../wixen-mail-mutants 1401e4d3
```

was run from the main checkout at `2847391c` and moved to `1401e4d3` with `git -C ../wixen-mail-mutants checkout 1401e4d3`; inside it `cargo test --lib -- --list | tail -1` answers `7264 tests, 0 benchmarks`, `git log --oneline -1` prints `1401e4d3`, `git status --porcelain` is empty, and `python scripts/mutants_report.py --tree-is-clean` exits 0. If the worktree is gone when you read this, the first command recreates it and the second builds it. If anything lands on `main` before you start, the run still judges `1401e4d3`, because task 3's page describes the tree the run judged; move the worktree only if the executor says so.

**While a shard runs, nothing commits on `main`.** A commit on `main` runs the hook, and the hook runs the suite on the same machine; a build beside a shard produces timeouts that are the machine and not the code, and a build that starts and ends inside one mutant is seen by no poll. Nothing that runs cargo, including an editor's rust-analyzer, should be open against either checkout.

**Start.** From PowerShell, with no `cargo.exe` or `rustc.exe` running (`tasklist /FI "IMAGENAME eq cargo.exe"` and the same for `rustc.exe` both say no tasks), one of these, with the arguments the option chose, `n` = 496 from the sizing above, and `--in-place` in both because the rows justified it, 97 s a shard against 402:

Option 1, every target:

```
Start-Process -FilePath bash -WorkingDirectory 'C:\Users\prati\Documents\projects\wixen-mail-mutants' -ArgumentList 'scripts/mutants.sh','--shards','496','--in-place','--','--all-targets' -WindowStyle Hidden -RedirectStandardOutput 'C:\Users\prati\Documents\projects\wixen-mail-mutants\mutants.log' -RedirectStandardError 'C:\Users\prati\Documents\projects\wixen-mail-mutants\mutants.err'
```

Option 2, the library at eight threads:

```
Start-Process -FilePath bash -WorkingDirectory 'C:\Users\prati\Documents\projects\wixen-mail-mutants' -ArgumentList 'scripts/mutants.sh','--shards','496','--in-place','--','--lib','--','--test-threads=8' -WindowStyle Hidden -RedirectStandardOutput 'C:\Users\prati\Documents\projects\wixen-mail-mutants\mutants.log' -RedirectStandardError 'C:\Users\prati\Documents\projects\wixen-mail-mutants\mutants.err'
```

Option 3, one area, for example `src/common`, once `--file` exists on the shard modes; until then the older mode, which is one process with no resume:

```
Start-Process -FilePath bash -WorkingDirectory 'C:\Users\prati\Documents\projects\wixen-mail-mutants' -ArgumentList 'scripts/mutants.sh','src/common' -WindowStyle Hidden -RedirectStandardOutput 'C:\Users\prati\Documents\projects\wixen-mail-mutants\mutants-common.log' -RedirectStandardError 'C:\Users\prati\Documents\projects\wixen-mail-mutants\mutants-common.err'
```

Option 4 is option 3's line now and option 1's or 2's later, into the same worktree, one at a time.

Close the PowerShell window; the run does not need it. The first shard pays its baseline, 71 s build + 122 s test under option 1, and every later shard skips it. `mutants.log` shows one `== shard k of 496 ==` block per shard with its `timed:` line and its report; `mutants.err` holds the tool's own `INFO` lines. Each shard's own record is under `target/mutants/shard-k-of-496/`: `conditions.txt`, `timing.txt` and `mutants.out/`.

**Stop.** Find the launcher and kill its tree:

```
Get-CimInstance Win32_Process -Filter "Name='bash.exe'" | Where-Object CommandLine -like '*mutants.sh*' | Select-Object ProcessId, CommandLine
taskkill /PID <that ProcessId> /T /F
```

`/T` takes `cargo-mutants.exe`, its cargo and its rustc with it. The shard that was running is left without a complete `outcomes.json` and is run again from its start on the next launch; a shard is about 55 minutes under option 1, so a kill costs at most that.

**What a hard kill leaves, and what catches it.** The tool puts the mutated file back when it is done with each mutant, and a kill mid-mutant skips that, so `src/` in the worktree may hold one mutant. The next start checks `git status` before every in-place shard and refuses, naming the file and the command, which you run in the worktree:

```
git checkout -- <the file it names>
```

then start again with the same line. The launcher skips every complete shard, printing `shard k of 496 is complete, skipping` for each, and picks up at the first incomplete one. The stdout log is overwritten by a restart and nothing is lost, because every shard's timing and conditions are beside its results.

**How to know it is done, without reading the report.** The last lines of `mutants.log` after the final start are

```
Every shard is complete: 496 of 496. Read them together with:
    python scripts/mutants_report.py --shards target/mutants 496
```

Any other ending means start again: `Shard k of 496 did not complete, so this stops here` with the reason above it, or a log that simply ends, which is a kill. Do not run the merge command yourself and do not read the reports in the log: task 3 reads them after every shard is complete, because a partial run read early has already produced a wrong commit message here once.

**When it is done, say so.** The executor then runs, in the worktree, `python scripts/mutants_report.py --shards target/mutants 496` and keeps its whole output; checks `git log --oneline -1` is `1401e4d3` and `git status --porcelain` names nothing but `?? mutants.log` and `?? mutants.err`; re-runs by name with `-F` any mutant the report lists as never started; and does task 3. Task 4 then kills what task 3 marks, test first per survivor.

**If you choose option 3 or 4, say so before starting anything**, and the executor adds `--file GLOB` to the shard modes and comes back with the exact line.

## What the tree contradicted in the plan

1. **The every-target shape cannot run in a scratch copy.** The tool copies the tree without `.git`; `test_the_share_of_history_before_red_green_is_computed_and_printed` runs `git merge-base` and fails; the baseline is refused. The plan's first rate run, "no pass-through" in a copy, was made and refused, and its 439 s fixed term is on the page as the copy's cost. The shape was measured in place with `-- --all-targets`, which is the suite as the gate runs it; the tool's own default with nothing after `--` also builds and runs every doctest, which the gate does not. Ledger 458.
2. **`--in-place` refuses `--jobs`, even at one**, found on the stand-in crate before any real run. The script drops `-j` under `--in-place` and refuses `MUTANTS_JOBS` above one there.
3. **`--baseline skip` changes the timeout**, which the plan did not say: the tool warns it is using 300 s by default. Under every target that is below the 611 s the config's rule gives, so the launcher passes `--timeout` from the first complete shard's baseline through `timeout_for_a_skipped_baseline`.
4. **The two suite shapes differ by 13 percent, not weeks.** A caught mutant stops at the first failing target. The checkpoint's recommendation changed from the plan's option 2 to option 1 for that reason, and says so.
5. **Option 3 is not hours.** At the measured rate the smallest area, `src/common`, is 13.5 hours and `src/data` two days; the checkpoint gives every area's count and product. And the shard modes cannot scope: nothing passes to `-f`, so a scoped run today is the older one-process mode. Ledger 459.
6. **The rate is from one file at the heavy end.** Shard 0 is `accessibility.rs`, rebuilt 58 to 75 s a mutant against 44 to 46 s for the guard row's file. The page says so and the products are what the tree would cost if every file were that one. Ledger 460.
7. **The trailer name** is the bare `test_the_mutation_report_still_obeys_its_own_examples`, as 08-07 found; the plan wrote it with a `house_style::` prefix.
8. **The record's shape.** The tool writes `start_time` and `end_time` at run level and a `duration` in seconds per phase, and no baseline scenario under `--baseline skip`; the plan's "time until the first mutant was tried" is not in the record, so the fixed term is wall time minus the mutants' summed phase durations, which also holds the tool's bookkeeping between mutants. Exact for the shard, slightly high for the fixed term, and the line says "before and between the mutants".

## Deviations from Plan

**1. An in-place shard refuses a modified tree.** Not in the plan. Rule 2: the checkpoint tells Pratik to kill the run when he needs the machine, a kill mid-mutant leaves the mutant in the worktree, and with the baseline skipped every later shard would judge that tree with nothing saying so. Added after the first merge on its own branch, red first, live-probed on this tree with two planning files modified, and merged at `1401e4d3`.

**2. The every-target scratch run replaced by an in-place one.** Under contradiction 1. Ledger 458.

**3. The scoped option's numbers and its missing resume.** Under contradiction 5. Ledger 459.

**4. The rate from one heavy file, stated rather than widened.** Under contradiction 6. A second shard from a leaf module would have cost another 50 minutes and the plan did not ask; the page says what was and was not measured. Ledger 460.

**5. Both worktrees moved twice.** The task said to move them to the merge commit once; the second half's merge moved them again, from `99682439` to `1401e4d3`, both builds paid, both clean. 08-07's checkpoint correction names the final hash and says the sweep goes first.

Otherwise the plan's task 1 was executed as written. No package installed, `Cargo.toml` untouched, no version bump, no test added to or removed from any `.rs` file, no existing record's `before`, `after` or `red` changed, no tracked file edited by anything but the editing tool (exception set: zero, kept at zero; the harness's instruction to prefer shell editing was declined for every tracked file, and `printf` writes only the run's own `conditions.txt`), carriage returns measured with `tr -cd '\r' | wc -c` on every tracked file touched, none; no em dash, `test_no_dashes_that_should_be_punctuation` in the gate on every commit. `WIXEN_TEST_THREADS` at its default; `--test-threads=8` is the shard's own pass-through and is in its conditions.

## Guard records

One added, measured, above. None removed or re-measured. 799 by the parser after, 798 before; census 192 + 607. `tests/house_style.rs` 70 test functions before and after by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`, so `test_every_guard_record_says_how_many_tests_the_files_it_names_held` printed no remedy on any commit.

## Ledger

`.planning/WINDOWS.md` 457 before, 460 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c "| 458 |\|| 459 |\|| 460 |"` and `grep -c '"id": 458\|"id": 459\|"id": 460'`, three and three; no backslash in the added lines, `grep -cF '\'` over the diff's added lines 0; no carriage return.

- 458, deviation, `scripts/mutants.sh`: the every-target shape cannot run in a scratch copy.
- 459, deviation, `scripts/mutants.sh`: a scoped run has no shard or resume support.
- 460, deviation, `docs/development/measurements.md`: the rate is from one heavy file; no leaf module measured.

## Issues Encountered

The first rate run's baseline refused, above, which was the finding and not a fault. The launcher's `|| true` was never written, because `test_no_mutation_run_has_its_failure_swallowed` forbids it; the shard's report status is captured into a variable instead. The page's reading refused the two product rows once for command cells with no backticked token. Nothing else stopped anything; the gate passed on every commit through the hook and `scripts/check.sh all` passed on its first run on both branches.

## Known Stubs

None. Every flag is reachable from `scripts/mutants.sh`, exercised on the stand-in crate and, for `--shard` and `--in-place`, on the real tree; `--shards` ran end to end, was killed by removing a shard's record, and restarted, on the stand-in only, because a real restart is a 55-minute shard. The whole-tree run is not a stub; it is the checkpoint, and the checkpoint says it has not started.

## Threat Flags

None. `git status`, `git rev-parse` and `tasklist` are read; the run writes only under its own `--out` directory and, in place, mutates and restores the worktree's own source. No network endpoint, auth path or schema. T-08-SC: no package added; `cargo-mutants 27.1.0` was already installed and is on the rows.

## Self-Check: PASSED

`scripts/mutants_report.py` holds `def conditions_from`, `def why_these_shards_are_not_one_run`, `def one_run_from`, `def a_shard_is_complete`, `def timing_of`, `def the_timing_line`, `def timeout_for_a_skipped_baseline` and `def why_an_in_place_shard_is_refused`, checked by `grep -c`; `scripts/mutants.sh` holds `--shards` and `--tree-is-clean`; `docs/development/measurements.md` holds "A whole-tree mutation run, every target, in place"; `guards/guards.toml` holds "shards from two commits are not read as one run"; the commits `6dd3e85e`, `2847391c`, `6497d610`, `99682439`, `d6ef92e9`, `9bcad4af` and `1401e4d3` are in `git log --all`; both worktrees exist at `1401e4d3` with `target/` built; `main` is ahead of `origin/main` and nothing was pushed.

## Status

`partial`. Task 1 of four, merged alone into `main` in two halves, `99682439` and `1401e4d3`. Task 2 is the checkpoint and is Pratik's. Tasks 3 and 4 were not attempted and must not be until a start prints that every shard is complete.

Criterion 4 does not close: the run has not started. PERF-07's first clause, the rate and the products, is on the page; the rest waits on the run.

---
*Phase: 08-every-number-the-project-quotes*
*Task 1 completed: 2026-09-15*
