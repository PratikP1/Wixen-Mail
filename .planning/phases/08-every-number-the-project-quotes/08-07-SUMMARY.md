---
phase: 08-every-number-the-project-quotes
plan: 07
status: partial
subsystem: testing
tags: [guards, sweep, resume, measurements, tooling]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "the `timed:` line the runner prints per record, and the rate and product rows on `docs/development/measurements.md` the checkpoint quotes"
  - phase: 08-every-number-the-project-quotes
    plan: 06
    provides: "the sweep's cost stated once as a rate times a count, so the checkpoint quotes a row rather than a figure"
provides:
  - "`scripts/guards.py` with `--log`, `--resume`, `--stop-after` and `--wait-until-quiet`; `verdicts_in`, `foreign_builds_in`, `is_quiet`, `why_a_resume_is_refused`, `the_resume_command` and `the_closing_line` as pure parts with worked examples the gate runs"
  - "A worktree at `../wixen-mail-sweep` on `main` at `1837f93b`, built once, waiting for Pratik to start the sweep"
  - "The checkpoint below, usable without the plan"
affects: [08-07 task 3, 08-08, 08-09]

actuals:
  tokens: 10312
  tasks: 1
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A long run's log is its own resume state: the runner prints a marker line before each unit and a verdict after, and the resume is a reading of exactly those lines, held by the examples of the function that prints them"
    - "A tool that breaks the tree and restores in a `finally` gets a refusal at the next start, read from `git status` over the files it breaks, because a process-tree kill does not reach a `finally`"

key-files:
  created: []
  modified:
    - scripts/guards.py
    - scripts/guards.sh
    - docs/changelog.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
    - .planning/WINDOWS.md

key-decisions:
  - "`--resume` is refused without `--log`, because a run that records its verdicts nowhere cannot itself be resumed; the plan only said a bare `--resume` takes the `--log` path"
  - "`--stop-after` cuts the selection before the pre-read, so a one-record chunk reads one suite rather than the 23 integration targets the file names; seen in the runaway run's output"
  - "`could not be measured` is a verdict and a resume skips it, because measuring a moved break again finds the same thing and the correction is by hand in task 3"
  - "The log is read before it is opened for appending, because the open creates it and the first live probe of a resume from a missing log started the whole sweep"
  - "`own_pids` is passed empty: the runner polls only between records, when `subprocess.run` has returned and no child of its own is alive, and the parameter stays so `is_quiet` states what quiet means"
  - "The worktree was created and built by the executor, because a checkpoint instruction nobody has run is the check nobody reads; the sweep itself was not started"

patterns-established:
  - "A refusal in an impure shell is run once on the input it refuses before the work is called done; the pure examples cannot see where in the sequence it sits"

requirements-completed: []

coverage:
  - id: D1
    description: "The runner appends its output to a log, resumes from that log measuring only what has no verdict, stops after a known number of records, and waits for a quiet machine"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/house_style.rs#test_the_guard_runner_still_obeys_its_own_examples, 95 examples in 54 items"
        status: pass
      - kind: other
        ref: "bash scripts/guards.sh --log target/one-record.log --stop-after 1 --wait-until-quiet, exit 0, one verdict and its timing line, then --resume --stop-after 1 skipping it and measuring the next"
        status: pass
    human_judgment: false
  - id: D2
    description: "A resume refuses over a modified guarded file and prints the checkout; a contended record is re-measured on resume; a detached start finishes with its window gone"
    requirement: PERF-06
    verification:
      - kind: other
        ref: "the three logs quoted under Proof by running below"
        status: pass
    human_judgment: false
  - id: D3
    description: "The one guard sweep of the milestone, every record on one commit"
    requirement: PERF-06
    verification:
      - kind: manual_procedural
        ref: "the checkpoint below; Pratik starts it"
        status: unknown
    human_judgment: true

duration: 1h20min
completed: 2026-09-15
---

# Phase 8 Plan 07: A sweep that can be stopped and picked up Summary

**The guard runner can be started detached, stopped at any record, and picked up from its own log measuring only what the log holds no verdict for; it refuses to resume over a tree a killed run left broken and prints the `git checkout` that cleans it; it waits before each record until no other cargo is building and marks a record contended when one was alive as its run returned. Every one of those was run, not read. The sweep itself is not started: it is hours on an idle machine and Pratik starts it, from the checkpoint below, in a worktree at `main` as it stands after this task's merge, `1837f93b`, which exists and is built.**

Task 1 of 3. Task 2 is the checkpoint. Task 3 is not attempted.

## Performance

- **Duration:** about 1 hour 20 minutes from the first read at about 03:15Z to the merge at 04:22Z, then the worktree's first build and this summary
- **Started:** 2026-09-15T03:15:00Z, the evening of 2026-09-14 on the machine
- **Completed:** task 1 merged at 2026-09-15T04:22:39Z; the plan is partial
- **Tasks:** 1 of 3
- **Files modified:** 3 on the branch, plus this summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md` on `main`

## What the runner can now do

`scripts/guards.sh` takes four new flags, documented in its usage block with a sentence each:

| Flag | What it does |
|---|---|
| `--log PATH` | Appends every line the runner prints to PATH as well as the screen, flushed per write, stderr included so a traceback lands in the log under the record it ended on. Appends and never truncates, because `Start-Process` has no append and a redirect would overwrite the log a resume reads. |
| `--resume [PATH]` | Reads PATH, or the `--log` path when PATH is left out, with `verdicts_in`; skips every record with a verdict; prints `Resuming from PATH: N of M already measured, K to go`; measures the rest. A name with no verdict is measured again. Refused without `--log`. Before measuring anything it runs `git status --porcelain` over every guarded file and refuses, naming each modified file and the `git checkout` that puts it back. The pre-read is taken again on every invocation, because the tree may have moved between them. |
| `--stop-after N` | Cuts the selection to N records before the pre-read, so a chunk reads only the suites its records name; ends with a line saying how many remain and the resume command; exits 0. `--stop-after 0` measures nothing and reports what remains. |
| `--wait-until-quiet` | Before the pre-read and before each record, polls `tasklist` for `cargo.exe` and `rustc.exe` and waits, printing once a minute what it is waiting for, until none is alive; after each record's run returns, polls once more and, if one is alive, prints `   contended: another cargo ran during this record` under the verdict with the process named, and counts the record as unmeasured so a resume takes it again. |

The last line of every run is `the_closing_line`: `Every record selected has a verdict: M of M, N from the log and K from this run.` when nothing remains, or `K measured this run; R of M remain. Resume with:` and the command. That line is how somebody told not to read the verdicts knows whether the sweep is done.

`verdicts_in` reads the exact lines the loop and `say_what_it_found` print: `-- <name>` opens a block; the first line in the block indented with the loop's three spaces, the timing line excepted, is the verdict; `agreed` for the two clean shapes, `short` for a named test that stayed green or an unnamed one that went red, `could not be measured` for the five `Wrong` shapes and the loop's catch-all line; a `contended:` line anywhere in the block voids it; the last block for a name is the one that counts; a line it cannot place is refused with the line quoted and the record named. `could not be measured` is a verdict, so a resume skips it: a moved break measured again finds the same thing, and task 3 corrects it by hand.

The module docstring now says the script reads git in two places rather than one, and says which and since when.

## Red first

The six pure functions were written as docstrings with worked examples and no body. `python -m doctest scripts/guards.py` reported 33 of the 38 new examples failing, the five that passed being the helper definitions and an `is None` case a bodiless function satisfies. `cargo test --test house_style test_the_guard_runner_still_obeys_its_own_examples` was red on it: `test result: FAILED. 0 passed; 1 failed`. The red commit names that test in its trailer as `test_the_guard_runner_still_obeys_its_own_examples`, the bare name cargo prints for an integration target, which is what `red-commit.sh` reads and what 08-01's red commit `bb61e88e` used; the plan and the prompt wrote it with a `house_style::` prefix, which the gate would have reported as a test that never ran. The gate confirmed the red: "a red that is exactly the one this commit named."

`python -m doctest -v scripts/guards.py`: **57 tests in 39 items** before this task, **95 tests in 54 items** after, 38 more where the criterion asked for six.

## Proof by running

Each behaviour was run on this machine, with `tasklist` checked for `cargo.exe` and `rustc.exe` before each start; it was quiet every time except where a stand-in was started on purpose. Every log below was written by `--log`; every carriage-return count is `tr -cd '\r' < file | wc -c`.

**The plan's verify, one record.** `bash scripts/guards.sh --log target/one-record.log --stop-after 1 --wait-until-quiet`, exit 0, 10 lines and 10 carriage returns, one per line, as premise 7 said a text-mode write on Windows would leave:

```
== 1 guard, one build and one run ==

Reading what already fails with --lib, before anything is broken.
   timed: rebuild 29 s, run 49 s, 78 s in all
-- the Google merge asks what the whole contact is owed
   timed: rebuild 38 s, run 48 s, 86 s in all
   all 4 tests named went red, and nothing else did

1 measured this run; 797 of 798 remain. Resume with:
    scripts/guards.sh --log target/one-record.log --resume --wait-until-quiet
```

**The refusal.** With `src/application/allowed.rs` given a trailing blank line, `bash scripts/guards.sh --log target/one-record.log --resume --stop-after 1` exited 1 with, appended to the same log:

```
A guarded file is modified in the working tree, so nothing was measured:
    src/application/allowed.rs

A run killed hard enough to skip its restore leaves its break behind, and a
sweep resumed over it would measure every record against that edit. If the
change is not yours, put the file back and start again:
    git checkout -- src/application/allowed.rs
```

`git checkout -- src/application/allowed.rs` cleaned it; `git status --short src/` was then empty.

**The resume.** The same command on the clean tree skipped the measured record and measured the next; the log then held both verdicts, 30 lines and 30 carriage returns:

```
Resuming from target/one-record.log: 1 of 798 already measured, 797 to go
== 1 guard, one build and one run ==

Reading what already fails with --lib, before anything is broken.
   timed: rebuild 38 s, run 47 s, 85 s in all
-- the Microsoft merge asks what the whole contact is owed
   timed: rebuild 38 s, run 47 s, 85 s in all
   all 2 tests named went red, and nothing else did

1 measured this run; 796 of 798 remain. Resume with:
    scripts/guards.sh --log target/one-record.log --resume
```

**The detached start.** From PowerShell, `Start-Process -FilePath bash -ArgumentList 'scripts/guards.sh','--log','target/detach-test.log','--stop-after','1','--wait-until-quiet' -WindowStyle Hidden; exit`, so the PowerShell process was gone at once. Started 03:53:49Z; the closing line was in the log by 03:56:52Z; no `python.exe`, `cargo.exe` or `rustc.exe` left. The log, 10 lines and 10 carriage returns:

```
== 1 guard, one build and one run ==

Reading what already fails with --lib, before anything is broken.
   timed: rebuild 37 s, run 48 s, 85 s in all
-- the Google merge asks what the whole contact is owed
   timed: rebuild 37 s, run 47 s, 84 s in all
   all 4 tests named went red, and nothing else did

1 measured this run; 797 of 798 remain. Resume with:
    scripts/guards.sh --log target/detach-test.log --resume --wait-until-quiet
```

**The wait, the contended mark, and the re-measure.** A copy of `sleep.exe` named `target/rustc.exe` stood in for a foreign build, so no real build contended. In the first attempt it was started on a fixed delay and landed before the record instead of during it, so the runner waited for it twice (`waiting for a quiet machine: rustc.exe 8708` before the pre-read, `rustc.exe 32304` before the record, printed once a minute) and measured a clean record; that proved the wait and not the mark, and is observation 587 in the skill log. The second attempt started the stand-in five seconds after the `-- ` line appeared in the log, for 150 seconds:

```
-- the Google merge asks what the whole contact is owed
   timed: rebuild 37 s, run 47 s, 84 s in all
   all 4 tests named went red, and nothing else did
   contended: another cargo ran during this record
       rustc.exe 2012

1 record had another cargo alive as its run returned, so it is unmeasured and a resume takes it again:
    the Google merge asks what the whole contact is owed

0 measured this run; 798 of 798 remain. Resume with:
    scripts/guards.sh --log target/contend2.log --resume --wait-until-quiet
Resuming from target/contend2.log: 0 of 798 already measured, 798 to go
== 1 guard, one build and one run ==

waiting for a quiet machine: rustc.exe 2012
waiting for a quiet machine: rustc.exe 2012
Reading what already fails with --lib, before anything is broken.
   timed: rebuild 37 s, run 46 s, 83 s in all
-- the Google merge asks what the whole contact is owed
   timed: rebuild 36 s, run 47 s, 83 s in all
   all 4 tests named went red, and nothing else did

1 measured this run; 797 of 798 remain. Resume with:
    scripts/guards.sh --log target/contend2.log --resume --wait-until-quiet
```

The resume read the contended block as no verdict, waited for the stand-in, and measured the same record again. The stand-in was removed afterwards.

**The other refusals**, each run once: `--resume` without `--log` prints why and exits 1; `--log target/nope.log --resume` with no such file prints `Nothing to resume from: target/nope.log is not there` and creates nothing; a log holding `-- odd` followed by `   something else` is refused as `under "-- odd", a line this cannot read as a verdict:` with the line; a made-up complete log resumes with `798 of 798 already measured, 0 to go` and prints the closing line without a build; a log with one contended and one short block reports the short one as `That verdict came from the log.` and 1 remaining.

**The rate today.** Seven timing lines over the session: rebuild 36 to 38 s, run 46 to 49 s, 83 to 86 s a record, below the 92 s the page's rate row gives for 2026-09-14 at `bb61e88e`. The page is not changed by this plan; task 3 writes the sweep's own figures beside the prediction.

## What the first live run found

Opening the log for appending before reading it for a resume created the file, so `--log target/nope.log --resume` did not refuse: it read an empty log, found nothing measured, and started the whole sweep, pre-reading all 23 integration suites before the library. It was killed during the pre-read with no break applied; `git status` showed only `scripts/guards.py`. Two things changed: the log is read before it is opened, and `--stop-after` cuts the selection before the pre-read so a chunk reads only the suites its records name. The pure examples could not have seen either, because both are orderings in `main`; each refusal was then run once on the input it refuses. Observation 586 in the skill log.

## What the gate selected for each file

By `scripts/which-checks.sh a-sweep-that-can-be-stopped-and-picked-up <file>`: `scripts/guards.py` alone answers `affected`, `scripts/guards.sh` alone `affected`, `docs/changelog.md` alone `docs_only`, all three together `affected`; on `main` the three answer `all`. Neither script maps to a target of its own; what reaches `scripts/guards.py` is `house_style`'s doctest runner, in `guards_that_read_the_whole_tree`, and `scripts/guards.sh` is read by nothing. The hook on both commits ran formatting, clippy, the four shell suites and the tree-reading guards. `scripts/check.sh --suites-for scripts/guards.py scripts/guards.sh` prints nothing, for the reason ledger 442 records.

## Task commits

Branch `a-sweep-that-can-be-stopped-and-picked-up` from `main` at `4bdaa47f`.

1. **Red:** `812241ea` test(08-07), six bodiless functions with 38 examples, trailer `Fails-until-green: test_the_guard_runner_still_obeys_its_own_examples`.
2. **Green:** `bbd1f28d` feat(08-07), the bodies, the flags in `main`, `guards.sh`'s usage block, the changelog entry.
3. **Merge:** `1837f93b`, `Merge 08-07 task 1`, alone into `main`, the whole gate on the merge: 7,746 passed and none failed over 58 result lines, 307 s.

`scripts/check.sh all` on the branch at `bbd1f28d`: exit 0 on its first run, 345 s, 7,746 passed and none failed over 58 result lines; ledger 374's keyring race was not met. Every commit went through `git commit` or `git merge` with the hook on; nothing was piped; the gate's output went to a file and its exit status was read directly; no `.git/index.lock` was left. Nothing is pushed.

**Plan metadata:** the commit carrying this summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md`.

## The checkpoint: start the sweep on an idle machine, and resume it as often as it takes

This is written to be acted on without the plan.

**What it is.** The one guard sweep of the milestone: `scripts/guards.sh` unfiltered over every record in `guards/guards.toml`, 798 today, one build and one whole-suite run per record, with nothing else building beside it. It is hours, it has to be started by you on a machine you are not about to need, and it can be stopped and picked up as often as you like.

**How long.** The row "The whole guard sweep, every record once" on `docs/development/measurements.md` reads **784 x 92 s = 72,128 s, about 20 hours**, dated 2026-09-14 at `bb61e88e`, from the rate row above it, 92 s a record. The file holds 798 records today, so the same rate gives about 20.4 hours; the seven records timed in this session ran 83 to 86 s each, which would be about 19 hours. Both are a rate times a count and the log will say what it really cost, one `timed:` line per record. Triage of what it finds is on top.

**Where.** A worktree at `main` as it stood after this task merged, `1837f93b`, which is the first commit that holds the flags below. It already exists and is built:

```
git worktree add ../wixen-mail-sweep 1837f93b
```

was run from the main checkout, and inside it `cargo test --lib -- --list | tail -1` answered `7264 tests, 0 benchmarks` after 5 m 48 s, 8.2 GB under `target/`. If the worktree is gone when you read this, those two commands recreate it. `git log --oneline -1` inside it must print `1837f93b`. Nothing else has landed on `main` since that merge as this is written; if something has by the time you start, the sweep still runs at `1837f93b`, because task 3's corrections are made against the tree the sweep judged and anything newer is covered only by the per-commit checks.

**Corrected 2026-09-15 by 08-08, before the sweep started: the worktree is at `1401e4d3`, not `1837f93b`.** 08-08's task 1 merged into `main` in two halves, `99682439` and `1401e4d3`, and the first added one record to `guards/guards.toml`, "shards from two commits are not read as one run", so a sweep at `1837f93b` would have measured 798 records and never that one. The sweep worktree was moved with `git -C ../wixen-mail-sweep checkout 1401e4d3`; `git log --oneline -1` inside it prints `1401e4d3`, `git status --porcelain` is empty, and `cargo test --lib -- --list | tail -1` there still answers `7264 tests, 0 benchmarks`, because nothing under `src/` changed between the two commits. The record count is 799 and the census 192 + 607. Wherever this checkpoint says `1837f93b`, read `1401e4d3`, including the check under "When it is done": `git log --oneline -1` must be `1401e4d3`, and the closing line reads `799 of 799`. The mutation worktree of 08-08, `../wixen-mail-mutants`, sits at the same commit, so both long jobs judge one tree; run the sweep first, because it is a day and the mutation run is weeks, and neither may run beside the other. Nothing else in this summary is changed.

**While it runs, nothing commits on `main`.** The worktree keeps `main` from changing the files under the sweep, but a commit on `main` runs the hook, and the hook runs the suite on the same machine and the same `target/` contention; that is the case the tree records as reporting three guards green that go red on their own. The runner now notices a foreign cargo alive at the end of a record and marks that record for re-measuring, but a build that starts and finishes inside a record's run is seen by neither poll, so the cheaper answer is not to start one. Nothing that runs cargo, including an editor's rust-analyzer, should be open against either checkout.

**Start.** From PowerShell, with no `cargo.exe` or `rustc.exe` running (`tasklist /FI "IMAGENAME eq cargo.exe"` and the same for `rustc.exe` should both say no tasks):

```
Start-Process -FilePath bash -WorkingDirectory 'C:\Users\prati\Documents\projects\wixen-mail-sweep' -ArgumentList 'scripts/guards.sh','--log','sweep.log','--wait-until-quiet' -WindowStyle Hidden
```

Close the PowerShell window; the run does not need it. `sweep.log` grows in the worktree root by one `-- name` block per record. The first two to three minutes are the pre-read of what already fails, one whole-suite run per distinct suite the records name, 23 integration targets and the library, and they print only a `timed:` line each; that silence is by design.

**Stop.** Every stop of a hidden run is a hard kill, so expect what the next paragraph describes. Find it and kill its tree:

```
Get-CimInstance Win32_Process -Filter "Name='python.exe'" | Where-Object CommandLine -like '*guards.py*' | Select-Object ProcessId, CommandLine
taskkill /PID <that ProcessId> /T /F
```

`/T` takes the cargo and rustc it spawned with it; without `/T` they run on as orphans and the next start waits for them to finish, which is also fine.

**What a hard kill leaves, and what catches it.** The runner puts the broken file back in a `finally`, which runs on Ctrl+C and not on `taskkill /F`, so the record that was running is likely to leave its break in the tree. The next start with `--resume` reads `git status` over every guarded file first and refuses, naming the file and printing the command, which you run in the worktree:

```
git checkout -- <the file it names>
```

then start again. The block for that record in the log has no verdict, so the resume measures it again. A copy of the file may also be left under `%TEMP%\wixen-guards-*`; it is harmless and can be deleted. Nothing else is touched: the runner changes nothing through git.

**Resume.** The same line with `--resume` added:

```
Start-Process -FilePath bash -WorkingDirectory 'C:\Users\prati\Documents\projects\wixen-mail-sweep' -ArgumentList 'scripts/guards.sh','--log','sweep.log','--resume','--wait-until-quiet' -WindowStyle Hidden
```

It appends to the same log, prints `Resuming from sweep.log: N of 798 already measured, K to go`, takes the pre-read again, and measures again any record the log marks contended or left without a verdict. Repeat until a start prints that nothing remains.

**How to know it is done, without reading the verdicts.** The last line of the log after the final start is

```
Every record selected has a verdict: 798 of 798, N from the log and K from this run.
```

Any other last line, `K measured this run; R of 798 remain. Resume with:` and the command, means resume again. Do not read the verdicts above it: task 3 reads them after the log is complete, because a partial log has already produced a wrong commit message here once.

**When it is done, say so and give the worktree's path.** The executor then runs, in the worktree, `bash scripts/guards.sh --log sweep.log --resume --stop-after 0`, which must print the line above; checks that `git status --porcelain` names nothing but `?? sweep.log`; checks that `git log --oneline -1` is `1837f93b`; measures the log's carriage returns with `tr -cd '\r' < sweep.log | wc -c`; and does task 3. The plan's verification sentence said the worktree's status would be empty; the log is untracked at the root, so the exact expectation is written here instead.

## What the tree contradicted in the plan

1. **The trailer name.** The plan and the prompt name the red as `house_style::test_the_guard_runner_still_obeys_its_own_examples`; `red-commit.sh` reads cargo's `test NAME ... FAILED` lines, which for an integration target carry the bare name, and 08-01's red commit `bb61e88e` used the bare name. The bare name was used and the gate accepted it.
2. **The census header is at `guards/guards.toml:82-83`, not `79-80`**; 08-06's comment edit above it moved it. It still reads 192 and 606, which sum to the 798 records the parser counts. Not touched.
3. **The product row's `what` is "The whole guard sweep, every record once"**, not "guard sweep, product"; the checkpoint quotes it by its own words.
4. **`--resume PATH` without `--log`** is a run whose verdicts go nowhere and whose next resume starts from the same place, so it is refused with the flag to add, where the plan only said a bare `--resume` takes the `--log` path. Ledger 456.
5. **The checkpoint's verification** says `git status --porcelain` in the worktree is empty; `sweep.log` at the root is untracked and not ignored, so it will show as `?? sweep.log`. Written into the checkpoint as the exact expectation rather than left to be found. The log stays at the root as the plan placed it rather than under `target/`, which a build tool owns.
6. **`Start-Process` from PowerShell's cwd** is what the plan's line relies on; `-WorkingDirectory` is added so the line is exact wherever it is pasted.

## Deviations from Plan

**1. `--resume` refused without `--log`.** Under contradiction 4. Ledger 456.

**2. A build that starts and finishes inside one record's run is marked by neither poll.** The plan's behaviour names this gap as accepted and the changelog's Known limitations says it; ledgered so it is visible at ship time rather than only in a plan. Ledger 457.

**3. The worktree was created and built by the executor.** The plan gives both commands to Pratik. They were run once here so the checkpoint's instructions are ones that have been executed: the worktree is at `1837f93b`, 8.2 GB, 5 m 48 s for the first build, and `--stop-after 0` in it printed `0 measured this run; 798 of 798 remain`. The log that run created was removed so the sweep's log starts with the sweep. The sweep itself was not started.

**4. The stop-after cut moved before the pre-read.** Under "What the first live run found". A design refinement inside the plan's behaviour, not a change to it.

Otherwise the plan's task 1 was executed as written. No package installed, `Cargo.toml` untouched, no version bump, no test added to or removed from any `.rs` file, no record's `before`, `after` or `red` changed, no tracked file edited by anything but the editing tool (exception set: zero, kept at zero), carriage returns measured with `tr -cd '\r' | wc -c` on every tracked file touched, none; no em dash, `test_no_dashes_that_should_be_punctuation` in the gate on every commit. `WIXEN_TEST_THREADS` at its default.

## Guard records

None added, removed or re-measured. 798 by the parser before and after; census 192 + 606 at `guards/guards.toml:82-83`. `tests/house_style.rs` holds 70 test functions and `src/presentation/wx_app.rs` 199 before and after by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`, so `test_every_guard_record_says_how_many_tests_the_files_it_names_held` printed no remedy on any commit. `scripts/guards.py` is named by no record. The two records measured during the proofs, `the Google merge asks what the whole contact is owed` and `the Microsoft merge asks what the whole contact is owed`, agreed every time, four and two named tests red and nothing else; no count was written, because none of those runs was `--remeasure`.

## Ledger

`.planning/WINDOWS.md` 455 before, 457 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c "| 456 |\|| 457 |"` and `grep -c '"id": 456\|"id": 457'`, two and two; no backslash in the added lines, `grep -cF '\'` over the diff's added lines 0; no carriage return.

- 456, deviation, `scripts/guards.py`: `--resume` refused without `--log`.
- 457, deviation, `scripts/guards.py`: a build inside one record's run is seen by neither poll.

## Issues Encountered

The runaway run, above, killed during its pre-read with nothing broken. `git merge -F -` does not read a message from stdin; the merge was made with a message file. Nothing else stopped anything; the gate passed on every commit through the hook and `scripts/check.sh all` passed on its first run.

## Known Stubs

None. Every flag is reachable from `scripts/guards.sh`, exercised in this session, and documented in its usage block. The sweep is not a stub; it is the checkpoint, and the checkpoint says it has not run.

## Threat Flags

None. `tasklist` and `git status` are read; nothing is written through either. No network endpoint, auth path or schema. T-08-SC: no package added.

## Self-Check: PASSED

`scripts/guards.py` holds `def verdicts_in`, `def foreign_builds_in`, `def is_quiet`, `def why_a_resume_is_refused`, `def the_resume_command`, `def the_closing_line` and `class Logged`, checked by `grep -c`; `scripts/guards.sh` holds `--wait-until-quiet`; `docs/changelog.md` holds "The guard sweep can be stopped and picked up from its own log"; the commits `812241ea`, `bbd1f28d` and `1837f93b` are in `git log --all`; `../wixen-mail-sweep` exists at `1837f93b` with `target/` built; `main` is ahead of `origin/main` and nothing was pushed.

## Status

`partial`. Task 1 of three, merged alone into `main` at `1837f93b`. Task 2 is the checkpoint and is Pratik's. Task 3 was not attempted and must not be until a start prints that nothing remains.

Criterion 5 does not close: the runner exists and the sweep has not run.

---
*Phase: 08-every-number-the-project-quotes*
*Task 1 completed: 2026-09-15*
