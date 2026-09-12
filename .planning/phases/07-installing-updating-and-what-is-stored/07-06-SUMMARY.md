---
phase: 07-installing-updating-and-what-is-stored
plan: 06
subsystem: ci
tags: [ci, cross-platform, linux, macos, wxdragon, guards, measurement]

requires:
  - "tests/house_style.rs::test_one_failing_target_does_not_hide_the_rest, which held the hardcoded pair of files that run the suite"
  - ".github/workflows/ci.yml's WIXEN_NO_AUDIO comment, copied rather than inherited, because workflow-level env belongs to the file it is written in"
provides:
  - ".github/workflows/other-platforms.yml, a dispatch-only build and test of the main crate on ubuntu-latest and macos-latest"
  - "The guard's file list now names three files rather than two, so a new workflow cannot hide later failures unseen"
affects: [07-09]

actuals:
  tokens: 2200
  tasks: 1
  commits: 1

tech-stack:
  added: []
  patterns:
    - "A probe workflow is dispatch-only until somebody has seen it pass, so a check nobody has run is never a check that is always red"
    - "A guard whose subject list grows is taken red by hand against the newly listed file, because a list extended and never proved reads as covered"
    - "A first step that runs with if: always() and prints the runner image, so the conditions of a measurement survive a job that fails at its first real step"

key-files:
  created:
    - .github/workflows/other-platforms.yml
  modified:
    - tests/house_style.rs
    - .planning/WINDOWS.md

key-decisions:
  - "No caching anywhere. The number this exists to produce is how long a cold build takes, and wxDragon puts the wxWidgets source and its two CMake build trees under the target directory, so a target cache would carry a C++ toolkit across runs"
  - "No changelog entry and no version bump. A CI workflow is met by nobody using the program, and CLAUDE.md ties both to a user-visible change"
  - "timeout-minutes: 120 on both jobs, against GitHub's default of 360, because whether this downloads prebuilt wxWidgets or builds the toolkit from source is the open question and the expensive answer has no measured upper bound"
  - "A macOS step that installs CMake only if it is absent, because upstream lists CMake under All Platforms and a missing build tool would answer a question nobody asked"
  - "The test step does not carry if: always(). A compile failure makes the test answer meaningless rather than missing, and the runner image is reported by a step that does run anyway"

patterns-established:
  - "A premise about an upstream document is re-read on the day it is used, and the re-reading is quoted with that day's date rather than the plan's"

requirements-completed: []

coverage:
  - id: D1
    description: "The workflow runs only when somebody dispatches it: no push, no pull request, no schedule"
    requirement: "SHIP-05"
    verification:
      - kind: other
        ref: "grep -rn 'workflow_dispatch\\|on:' .github/workflows/other-platforms.yml, four hits, two of them runs-on"
        status: pass
    human_judgment: false
  - id: D2
    description: "The new workflow is visible to the guard that holds CI steps to --no-fail-fast, and the guard can see a violation in it"
    requirement: "SHIP-05"
    verification:
      - kind: integration
        ref: "tests/house_style.rs#test_one_failing_target_does_not_hide_the_rest"
        status: pass
      - kind: other
        ref: "taken red by hand on 2026-09-12: a step running the suite without the flag, and the guard named .github/workflows/other-platforms.yml:135"
        status: pass
    human_judgment: false
  - id: D3
    description: "Nothing in the workflow can tag, publish or push"
    requirement: "SHIP-05"
    verification:
      - kind: other
        ref: "permissions: contents: read at workflow level and in both jobs; no secrets. or GITHUB_TOKEN anywhere in the file; release.yml unchanged by git diff"
        status: pass
    human_judgment: false
  - id: D4
    description: "The crate builds and its suite passes on Linux and on macOS"
    requirement: "SHIP-05"
    verification: []
    human_judgment: true
    rationale: "Unrun. Nobody has dispatched the workflow, so nothing about either platform is known. WINDOWS.md 323."
  - id: D5
    description: "The workflow itself does what it says when GitHub runs it"
    requirement: "SHIP-05"
    verification: []
    human_judgment: true
    rationale: "Unrun. No YAML parser for GitHub Actions has read it, the apt line has never run on an ubuntu-latest image, and the macOS CMake step has taken neither branch. WINDOWS.md 324."

duration: 55min
completed: 2026-09-12
status: partial
---

# Phase 7 Plan 06: One dispatch says whether this crate builds off Windows Summary

**Task 1 of 3 landed. There is now a workflow anybody can dispatch to find out whether this crate builds and its suite passes on Linux and on macOS. Nobody has dispatched it, so the answer is still unknown, SHIP-05 does not close and phase 7's criterion 5 stays open. Task 3 was not attempted, because everything in it acts on an answer that does not exist yet.**

## The checkpoint: open, unrun, and not blocking anything

This is the part to act on later, written so nobody has to re-read the plan.

**What to do.** Open the repository's Actions tab on GitHub, pick the workflow
called **Other platforms**, press Run workflow, and choose either `main` or the
branch `one-dispatch-says-whether-this-crate-builds-off-windows`. Both carry the
file; the branch is merged, so `main` is the simpler choice. Let both jobs
finish, whichever way they go. They do not depend on each other on purpose:
macOS and Linux fail differently and one answer is not the other, so a Linux
failure still leaves the macOS answer on the table.

**What to report back, per platform, four things.**

1. Whether `cargo build --all-targets --verbose` succeeded, and if not, the
   first error in full, including which crate it came from.
2. Whether `cargo test --all-targets --verbose --no-fail-fast` succeeded, and if
   not, how many tests ran, how many failed, and the names of the first few.
3. How long the job took, wall clock, from the run summary.
4. The runner image label the job reported. The first step of each job prints
   it on purpose, under the name "Say what this ran on", and that step runs with
   `if: always()` so it reports even when the job fails at its first real step.

**If the build failed on the wxWidgets step specifically, that is the answer the
whole plan was written to get**, and the error text matters more than anything
else here. It decides whether SHIP-05 is two CI jobs or a port.

**Why a person and not an agent.** Guardrail 7 says publishing and anything
reaching outward happens on purpose. An agent dispatching a GitHub Actions run
is exactly the side effect that guardrail exists to prevent, and the workflow is
dispatch-only for the same reason.

**Nothing in phase 7 waits for this.** It is recorded as `WINDOWS.md` 323 and
324 and the phase carries on without it. When the answer arrives, task 3 of
`07-06-PLAN.md` is what acts on it, and the plan spells out both branches: two
push jobs in `ci.yml` if both went green, or a recorded finding that this is a
port if either did not.

## Performance

- **Duration:** about 55 min
- **Tasks:** 1 of 3
- **Commits:** 1 on code, plus the documents
- **Files:** 2 outside `.planning`, one of them new

`actuals.tokens` is 2,200, taken as the characters of the added and removed
lines of the whole branch divided by four: 8,801 characters, excluding
`.planning`. The command, so the next reader re-runs it rather than trusting the
figure:

    git diff b8857bc8..HEAD -- . ':(exclude).planning' | grep '^[+-]' | wc -c

Against an estimate of 54,000 with `raw_tokens` 45,000 that is a ratio of about
0.05 on raw, well under this phase's other five, which are 0.13, 0.10, 0.13,
0.13 and 0.23 for 07-01 to 07-05, taken now from their `actuals.tokens` against
their plans' `raw_tokens` rather than quoted from anywhere. Two
reasons, and neither is that the estimate was bad. One task of three ran, and
that task's whole output is a 160-line YAML file and a ten-line change to an
array. The estimate priced three tasks including a `ci.yml` expansion and a
ledger entry carrying a measurement that does not exist.

## What landed

- **`.github/workflows/other-platforms.yml`**, dispatch-only, read-only. Two
  jobs, `ubuntu-latest` and `macos-latest`, neither needing the other, each
  building the main crate with `cargo build --all-targets --verbose` and running
  `cargo test --all-targets --verbose --no-fail-fast`.
- **The guard's file list names three files rather than two.**
  `test_one_failing_target_does_not_hide_the_rest` read the literal
  `["scripts/check.sh", ".github/workflows/ci.yml"]`, so a new workflow could
  have built the same hole in a new place with nothing noticing.
- **Three ledger entries.** 323 and 324 say what is unrun and what that leaves
  unknown. 325 is a finding about two agents sharing one working tree, written
  up under deviation 6.

## Task commits

| # | Task | Kind | Commit |
|---|---|---|---|
| 1 | A workflow anybody can dispatch to find out | chore | `31345c67` |
| 2 | Run it on both platforms | checkpoint | not run, `WINDOWS.md` 323 |
| 3 | Act on the answer | not attempted | there is no answer to act on |

Branch `one-dispatch-says-whether-this-crate-builds-off-windows`, off `main` at
`b8857bc8`, merged at **`c6546e66`**.

**The full gate before the merge: all four passed in 654 seconds over 7,492
tests.** Not piped anywhere, redirected to a file, and the exit status read from
the script rather than from a pipeline. That is slower than the 419 to 471
seconds the last four branches report, and the reason is not this branch: the
run followed several test rebuilds, so it paid for compilation the warm figures
did not. Quoting it without that condition would put a number into the next
plan's estimate that nothing here earned.

## Every acceptance criterion of task 1, with what proves it

**The triggers.** The whole `on:` block, quoted:

```yaml
on:
  workflow_dispatch:
```

The plan's verification command and its output, run on the branch:

    $ grep -rn "workflow_dispatch\|on:" .github/workflows/other-platforms.yml
    14:on:
    15:  workflow_dispatch:
    68:    runs-on: ubuntu-latest
    136:    runs-on: macos-latest

Four hits, two of which are `runs-on`. No `push`, no `pull_request`, no
`schedule`, no `workflow_run`, no `repository_dispatch`. The words `push`,
`pull request` and `release` appear in the file only inside comments explaining
why they are absent.

**Permissions, and nothing that could publish.** `permissions: contents: read`
at workflow level and again in each of the two jobs. No `secrets.` reference and
no `GITHUB_TOKEN` anywhere in the file, so no step is handed a credential that
could tag or publish. `.github/workflows/release.yml` is untouched:
`git diff b8857bc8..HEAD -- .github/workflows/release.yml` is empty.

**Nothing here changes when a release can happen.** Said plainly, because the
plan asks for it in as many words. Releases are cut by dispatching
`release.yml`, which this branch does not modify, and nothing added here tags,
publishes or pushes anything.

**`WIXEN_NO_AUDIO` with the reason.** Set at workflow level, with ci.yml's
comment carried over rather than summarised, including its last paragraph: this
does not skip a single sound test, they all still run and all still assert, and
the sound goes to a mixer with nothing listening instead of to a card. The
comment also says why it is copied rather than inherited, which is the premise
this plan turned into a rule: workflow-level `env` belongs to the file it is
written in.

**The Linux package list, quoted from upstream with the date read.** Thirteen
packages, word for word from the wxDragon README's "Linux Additional
Requirements", read on **2026-09-12**, in a comment above the step that installs
them. They come from Ubuntu's own repositories. Nothing is fetched at run time
and run.

**The search handler is excluded with a comment saying why.** It is a second
crate rather than a workspace member, so every cargo command in the file walks
past it, and that is deliberate: it is a Windows COM server. `working-directory:
search-handler` appears nowhere in the new file.

**No `#[test]` was added to `tests/house_style.rs`.** `grep -c '#\[test\]'`
answers **78** before and **78** after. The checker's own count, which is what
the per-commit staleness check compares against, is **69** before and after, seen
directly in the test run output: "69 passed" on the full target, and "68
filtered out" when one test was run by name.
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` passed on
the commit and printed no remedy, so **no re-measurement is owed** and none was
run.

**The changelog, decided rather than left.** No entry, and no version bump.
`CLAUDE.md` ties a changelog entry and a version bump to a user-visible change.
Nobody running Wixen Mail meets a CI workflow. Version stays at `0.116.0`.

## Taking the guard red by hand, with the failure quoted

A guard extended to cover a new file and never shown to see a violation in it is
a guard that reads as covered. So the list was extended, then a step was put
into the new workflow that runs the suite in the way the guard exists to catch:

```yaml
    - name: A step that hides every target after the first failure
      run: cargo test --all-targets --verbose
```

`cargo test --test house_style test_one_failing_target_does_not_hide_the_rest`:

    thread 'test_one_failing_target_does_not_hide_the_rest' panicked at
    tests\house_style.rs:6119:5:
    1 place(s) run the suite in a way that hides every target after the first
    failure:
      .github/workflows/other-platforms.yml:135: run: cargo test --all-targets --verbose

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 68 filtered out

It names the new file and the line. The step came back out and the full target
passed, 69 of 69.

**What this does not prove, said rather than left.** The second half of that
test, `steps_an_earlier_failure_would_skip`, reads `ci.yml` alone and only looks
at steps carrying `working-directory: search-handler`. The new file has none, so
that half is silent about it. That is correct rather than a gap: the new
workflow does not touch the search handler on purpose. If task 3 ever adds a
search handler step anywhere, that reading needs widening too, and nothing
currently says so.

## The decisions the plan left to the executor

**Caching: none, on purpose.** The plan said to add it if it is cheap and to say
what was decided. It is cheap and it was not added. The number this workflow
exists to produce is how long a first build takes on a machine that has never
built this, and a warm target directory is the one condition that makes that
number meaningless. There is a second reason found while reading upstream
today: the wxDragon README says the wxWidgets source and its two CMake build
trees default to the profile level of the target directory. A cache of `target`
would therefore carry a C++ toolkit build across runs, and the second run would
measure something nobody asked about. Caching belongs to the push jobs of task
3, if there are any, where the question is speed rather than truth.

**`timeout-minutes: 120` on both jobs.** Not in the plan, added as a deviation.
GitHub's default job timeout is 360 minutes. Whether this downloads prebuilt
wxWidgets libraries or builds the toolkit from source is the open question of
the whole plan, and the expensive answer has no upper bound anybody here has
measured. Two hours is generous for a cold source build plus this crate plus a
suite of about 7,000 tests, and it bounds a run that is going nowhere instead of
spending six hours of somebody's runner minutes to say the same thing.

**A macOS step that installs CMake only if it is absent.** Also a deviation. The
plan says nothing beyond Rust and a C++ compiler for macOS, and upstream's
macOS section does list no platform-specific packages. But upstream's "All
Platforms" prerequisites are Rust, CMake and a C++ compiler, and the Linux job
gets CMake from the apt line. Without the same guarantee on macOS, a runner
image that has dropped CMake fails the job for a reason that says nothing about
whether this crate builds there, and the run would be read as "this does not
build on macOS". `cmake --version || brew install cmake` costs a second when
CMake is present. Homebrew is preinstalled and is its own repository, so this
fetches no script at run time, which keeps T-07-SC's mitigation intact.

**The test step does not carry `if: always()`.** Considered, because guardrail 4
is about steps hiding steps. Rejected: if `cargo build --all-targets` fails, the
test step fails with the same compile error, so running it anyway adds a second
copy of one error rather than a second answer. What the checkpoint actually
needs when a job dies early is the runner image, and that is reported by the
first step, which does carry `if: always()`.

## What the gate decided for a workflow-only change, measured

The brief asked for this rather than an assumption, and the assumption would
have been wrong in an interesting direction. `scripts/which-checks.sh` has no
rule about `.github/` at all. A workflow file reaches the last loop in the
script, which asks only whether every changed path ends in `.md` or `.txt`, and
a `.yml` does not, so it answers `affected`:

    $ bash scripts/which-checks.sh some-branch .github/workflows/other-platforms.yml
    affected
    $ bash scripts/which-checks.sh some-branch .github/workflows/other-platforms.yml tests/house_style.rs
    affected
    $ bash scripts/which-checks.sh main .github/workflows/other-platforms.yml
    all

**What `affected` then runs for this particular change is narrower than it
sounds, and it is worth naming.** `run_the_tests_that_reach_what_changed` in
`check.sh` maps a changed file to a target by its path and knows two shapes,
`src/*.rs` and `tests/*.rs`. A `.github/workflows/*.yml` is neither, so the
workflow file selects **no scoped target of its own**. What checked it on this
commit is `tests/house_style.rs`, which was changed in the same commit and maps
to `--test house_style`, and the whole-tree guards that `check.sh` runs at the
end of every scoped run regardless. `house_style` is one of those guards, and
`ours()` at line 55 collects `.github/**/*.yml`, so the new file was read by the
document checks either way.

**That is the same shape as the `.iss` hole this project closed on 2026-09-09**,
one layer along: a file that something reads as data, mapping to no scoped
target. It is smaller here, because `house_style` reads every workflow file in
the tree and runs on every commit, so a workflow change is never wholly
unchecked. It is not nothing, though: a commit that changes only a workflow file
and nothing else would run the four tree guards and no target chosen for it.
Recorded here rather than fixed, because it is a finding about `check.sh` rather
than about this plan.

The commit's own gate output, for the record:

    == rustfmt ==
    == clippy ==
    == the scripts that decide what runs ==
    -- check
    -- red-commit
    -- which-checks
    == the tests that reach what changed ==
    -- house_style
    -- the guards that read the whole tree

## Deviations from plan

**1. `timeout-minutes: 120` added to both jobs.** Not asked for. Rule 2, a
missing operational safeguard: GitHub's default is six hours and the expensive
branch of this measurement is unbounded. Reasoned above.

**2. A macOS step that installs CMake if it is missing.** The plan says nothing
beyond Rust and a C++ compiler for macOS. Rule 2 again: upstream lists CMake
under All Platforms, the Linux job already gets it, and a missing build tool
would contaminate the measurement the plan exists to take. Reasoned above.

**3. A first step in each job that prints the runner image, with
`if: always()`.** Not in the plan. The checkpoint asks for the runner image
label as one of its four answers, and a job that fails at its first real step
would otherwise leave that answer buried. Rule 2: the measurement needs its
conditions.

**4. Task 3 not attempted, and task 2 not run.** By instruction, and by the
plan's own shape. Task 3 acts on an answer that does not exist. No agent
dispatched the workflow: guardrail 7, and the checkpoint's own last acceptance
criterion.

**5. No `docs/changelog.md` entry, which the plan's `files_modified` lists.**
The plan asked for the decision rather than the entry, and the decision is no.
Nothing user-visible changed.

**6. The roadmap's phase 6 row was edited, which is not this plan's work.**
Rule 3, a blocking issue. `test_the_roadmap_counts_the_files_that_are_on_disk`
is one of the four guards that read the whole tree, so it runs on every commit,
and it reads the **disk** rather than the index. A second agent was planning
phase 6 in the same working tree while this ran, and its plan files made the
roadmap's `0/TBD` false: that cell is honest only while a phase has no plans on
disk, and phase 6 had some. The commit was refused for it.

**Chasing the number failed three times.** The refused commit reported 1 plan on
disk, a re-run reported 2, the commit written against 2 was refused at 3, and
the commit written against 3 was refused at 5. The planner was writing faster
than a gate run takes, so every attempt was stale before it finished. What
worked was to stop guessing and watch: the count was polled until it held still,
which it did at **8** for several minutes, and the row was written then.

Eight is corroborated rather than only observed. Phase 6's own
`README.md` says, under what it owes to documents, that "the roadmap's progress
row for phase 6 has to say `0/8`, and the gate is red until it does", and that
it deliberately did not write it because `.planning/ROADMAP.md` and
`.planning/STATE.md` "are being edited by phase 7's executors in parallel and
were moving during this work". So the edit was handed over rather than taken.
The same README asks whoever owns the merge to correct
`progress.total_plans` by counting rather than incrementing; it says 92 against
89 files at the time it was written, and counting today gives **97**, which is
what `.planning/STATE.md` now says.

The finding worth carrying is not the edit. It is that **a whole-tree guard
reading the disk cannot tell one agent's uncommitted work from another's**, so
two agents in one working tree block each other's commits on bookkeeping neither
of them owns, and neither can see when the other has stopped. `WINDOWS.md` 325
raises whether such a guard should read the index instead.

## The roadmap row for this plan, and why it counts a plan that did not finish

`| 7. Installing, updating and what is stored | 5/9 |` became `6/9`, and the
checkbox for `07-06-PLAN.md` stays unticked. Those two facts disagree on their
face, so the row says why: the check counts `*-SUMMARY.md` files on disk, not
plans that closed what they were written for. A summary exists for this plan and
says `partial`. `progress.completed_plans` in `.planning/STATE.md` is 86 for the
same reason and by the same count.

The roadmap was edited by hand and the diff was read. `roadmap
update-plan-progress` was not run: it counts `PLANS-README.md` as a plan, writes
the wrong fraction, adds a stray checkbox and blanks the phase's status note,
which is four confirmed failures across waves 4 and 5.

## What is still false in the plan

**1. Premise 9's record count and test count are both stale by one day.** It
says, re-measured 2026-09-11 against `main` at `febe8e4`, that
`tests/house_style.rs` is fingerprinted by **18** records and holds **67**
tests, with a grep answering **76**. Re-measured today, 2026-09-12, against
`main` at `b8857bc8`, version 0.116.0, by the awk over `tests_last_seen` blocks
that `CLAUDE.md` prescribes: **19** records, **69** tests stored, and a grep
answering **78**. Four plans landed in between. The conclusion the premise draws
is unaffected, because this plan added no test to that file, but the figure a
later reader would quote has moved.

**2. Task 3's `read_first` says `.planning/WINDOWS.md` ends at entry 301 with
280 open, corrected from 146.** It ended at **322** before this plan and at
**325** after it, with 303 open. The success criteria block repeats the 301.

**3. The frontmatter's `files_modified` names `docs/changelog.md` and
`.github/workflows/ci.yml`.** Neither is touched, and neither should be:
the changelog because nothing user-visible changed, and `ci.yml` because task 3
did not run. This is the same class of disagreement the frontmatter's own
2026-09-11 correction fixed for `Cargo.toml`.

**4. Premise 1's optimism is weaker than it reads, and the same README says
both things.** The premise quotes upstream saying wxDragon "automatically
downloads pre-built wxWidgets libraries during the first compilation, reducing
build times from 20+ minutes to under 3 minutes", against the research's claim
that it builds the toolkit from source. Re-read today: **every place in that
README that names a platform beside the word "prebuilt" names Windows**, MSVC or
MinGW, and the file also says, in its Prerequisites section, that "wxDragon's
builder downloads wxWidgets source code from GitHub using git's proxy settings"
and that the wxWidgets source and two CMake build trees default to the profile
level of the target directory. The Linux requirements list includes `cmake` and
GTK development headers, which is what a source build needs and not what
linking a downloaded library needs.

That does not settle it either way, and this summary is not claiming it does.
What it changes is the expectation: the plan reads as though the cheap answer
were the likely one, and the evidence for that is a sentence whose every
platform-specific neighbour is about Windows. The 120-minute timeout is written
for the other case.

## Nothing here has been run by GitHub

Said plainly, because this is the failure mode `CLAUDE.md` opens with. The
workflow compiles nothing and proves nothing until it is dispatched.

- No GitHub Actions YAML parser has read the file. A syntax error would surface
  on the first dispatch, not before.
- The thirteen apt packages have never been installed on an `ubuntu-latest`
  image. A package renamed or dropped from Ubuntu's repositories fails the job
  for a reason that has nothing to do with this crate.
- The macOS CMake step has taken neither branch.
- No `cargo build` has ever been attempted for this crate on any platform other
  than Windows, which is the whole point of the plan and remains true after it.

That is `WINDOWS.md` **324**, and the measurement itself is **323**.

**Nothing here claims the application is accessible or usable on Linux or
macOS**, and if the answer comes back green, nothing may. SHIP-05's second `[D]`
line, quoted: "No criterion here claims the application is accessible on Linux
or macOS, or usable there. Building is not the same claim." The reason is
structural and this project already writes it down: `wxAccessible` and
`UiaRaiseNotificationEvent` are Windows only, and both compile and silently do
nothing elsewhere. Plan 07-03's disclosure is the other side of that pair and
already says what does nothing there.

## SHIP-05 and criterion 5

Neither closes. `requirements-completed` is empty, `.planning/REQUIREMENTS.md`
is untouched, and SHIP-05's box stays unticked. The criterion reads "The crate
builds and the suite passes on Linux and on macOS in CI", and after this plan
there is a way to find out and no answer. Nothing was half added to `ci.yml` as
a placeholder, and no job that is known to fail sits on a push trigger anywhere:
the new file's only trigger is `workflow_dispatch`.

## Verification, run rather than asserted

| What | Command | Result |
|---|---|---|
| The guard target | `cargo test --test house_style` | 69 passed, 0 failed |
| The guard sees a violation in the new file | the step above, put in by hand | red, naming `other-platforms.yml:135` |
| The commit gate | the `commit-msg` hook, `affected` | rustfmt, clippy, the three script suites, `--test house_style`, the tree guards, all green |
| The full gate before the merge | `bash scripts/check.sh all > file 2>&1` on the branch tip | all four passed, 654 seconds, 7,492 tests |
| Triggers | `grep -rn "workflow_dispatch\|on:" .github/workflows/other-platforms.yml` | 4 hits, 2 of them `runs-on` |
| `release.yml` untouched | `git diff b8857bc8..HEAD -- .github/workflows/release.yml` | empty |
| What the gate decides | `bash scripts/which-checks.sh <branch> <files>` | `affected` on a branch, `all` on `main` |
| Line endings | `tr -cd '\r' < .github/workflows/other-platforms.yml \| wc -c` | 0 |
| Test count | `grep -c '#\[test\]' tests/house_style.rs` | 78 before, 78 after |

No commit used `--no-verify`. No tracked file was edited by a script: every
change went through Read then Edit or Write.

## Guard records

None added, none removed, none re-measured. `guards/guards.toml` holds **729**
records with the census at lines 79 and 80 reading 192 and 537, unchanged by
this plan. The one record naming a workflow file,
`.github/workflows/mutants.yml`, is untouched, and no record can name a file
this plan created.

The per-commit staleness check passed and printed no remedy, which is what says
the 19 records fingerprinting `tests/house_style.rs` are owed nothing.
`scripts/guards.sh --touched-by b8857bc8` is owed after the merge as usual and
must not block it. It is not on the critical path.

## Self-Check: PASSED

- `.github/workflows/other-platforms.yml` exists on disk.
- `tests/house_style.rs` carries the three-file list.
- Commit `31345c67` is in `git log`.
- `.planning/WINDOWS.md` carries 323, 324 and 325 in both the markdown table and
  the JSON block, 325 rows against 325 ids.
