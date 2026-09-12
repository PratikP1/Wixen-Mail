---
phase: 07-installing-updating-and-what-is-stored
plan: 02
subsystem: infra
tags: [installer, commit-gate, shortcuts, icon, guards, which-checks]

requires:
  - phase: 07-installing-updating-and-what-is-stored
    provides: nothing this plan reads; wave 1 landed first and the two do not touch a file in common
provides:
  - scripts/which-checks.sh answers `all` for a change to any .iss, below the version-bump exception
  - Five cases in which-checks.test.sh covering an installer change on a branch, beside source, beside a version bump, on main, and a document in the installer folder
  - installer/Wixen-Mail-Setup.iss ships assets\icon.ico into {app}
  - Both [Icons] entries and UninstallDisplayIcon name {app}\icon.ico
  - tests/installer.rs, a target for rules about the installer script on its own
  - A guard record whose break is the half-fix rather than the absent one
affects: [07-07, 07-08]

actuals:
  tokens: 5600
  tasks: 2
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A gate rule placed by where an existing exception sits rather than by where it reads well"
    - "A text-reading check that compares two whole paths rather than asking whether each section mentions the thing"
    - "A companion over fixtures driving the same predicate the corpus walk uses, so the two cannot drift"

key-files:
  created:
    - tests/installer.rs
  modified:
    - scripts/which-checks.sh
    - scripts/which-checks.test.sh
    - installer/Wixen-Mail-Setup.iss
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "The .iss rule matches *.iss rather than installer/*.iss, because an .iss anywhere in this tree is a setup script and the extension is what decides"
  - "The rule sits below the manifest block, so an installer change that also bumps the version still earns the full gate"
  - "No guards.toml record for task 1, because a record applies a break to a file and names Rust tests, and this behaviour is held by a shell suite"
  - "The icon is shipped as a file rather than the shortcuts pointing at the executable, because a shortcut naming an installed file survives the executable losing its resources"
  - "UninstallDisplayIcon moves with the two shortcuts, because it had the same dependence and it is the picture beside the entry somebody goes to when they want the program gone"
  - "The test compares the installed destination and the named icon path as one value rather than asserting each section mentions an icon"

patterns-established:
  - "A load-bearing negative case chosen for what it discriminates rather than for repeating an existing case"
  - "A guard record whose file is not a Rust file takes its counts off the red list through `suite`"

requirements-completed: [SHIP-03]

coverage:
  - id: D1
    description: "A commit that changes only the installer script runs the tests that read the installer script"
    requirement: "SHIP-03"
    verification:
      - kind: other
        ref: "scripts/which-checks.test.sh::an installer script on a branch"
        status: pass
      - kind: other
        ref: "scripts/which-checks.test.sh::an installer script beside a version bump"
        status: pass
      - kind: other
        ref: "scripts/which-checks.test.sh::a document inside the installer folder"
        status: pass
    human_judgment: false
  - id: D2
    description: "Both shortcuts name an icon file the installer put there, and the path they name is the path the [Files] line installs to"
    requirement: "SHIP-03"
    verification:
      - kind: integration
        ref: "tests/installer.rs#test_the_icon_each_shortcut_names_is_the_one_the_installer_ships"
        status: pass
      - kind: integration
        ref: "tests/installer.rs#test_the_reading_can_tell_a_shortcut_that_names_the_shipped_icon_from_one_that_does_not"
        status: pass
      - kind: other
        ref: "scripts/guards.sh 'the icon a shortcut names is the one the installer really put there'"
        status: pass
    human_judgment: true
    rationale: "Nothing here compiles the installer with ISCC, installs anything or looks at a shortcut, so whether the picture really appears on a Start menu, a desktop and an Apps and Features entry is unverified. WINDOWS.md 306."
  - id: D3
    description: "Apps and Features is pointed at the same installed file"
    requirement: "SHIP-03"
    verification:
      - kind: integration
        ref: "tests/installer.rs#test_apps_and_features_shows_the_icon_the_installer_ships"
        status: pass
    human_judgment: true
    rationale: "Same limit as D2. WINDOWS.md 306."
  - id: D4
    description: "The desktop shortcut is still gated on the desktopicon task, so unticking the box still leaves no desktop shortcut"
    verification:
      - kind: integration
        ref: "tests/installer.rs#test_the_desktop_shortcut_is_still_the_users_choice"
        status: pass
    human_judgment: false

duration: 43min
completed: 2026-09-12
status: complete
---

# Phase 7 Plan 02: An installer change earns the gate, and the shortcuts name a shipped icon Summary

**A commit that changes the installer script now runs the four tests that read it, and the two shortcuts and the Apps and Features entry name an icon file the installer really put on the machine rather than depending on the executable's resource table.**

## Performance

- **Duration:** 43 min
- **Started:** 2026-09-12T03:11:37-04:00 (first commit)
- **Completed:** 2026-09-12T03:54:47-04:00 (summary written; the merge follows)
- **Tasks:** 2
- **Files modified:** 9, of which one is new

`actuals.tokens` is 5,600, taken as the characters of the added and removed
lines of the whole branch divided by four: 22,365 characters. It excludes
`.planning`, so it excludes this summary, the state file and the ledger entry.
The command, so the next reader re-runs it rather than trusting the figure:

    git diff 38f1ba1..HEAD -- . ':(exclude).planning' | grep '^[+-]' | wc -c

Against an estimate of 66,000 with `raw_tokens` 55,000, that is a ratio of about
0.10 on raw. The four most recent summaries give 0.13, 0.17, 0.15 and 0.20 on
the same measure, so this plan came in cheaper than all of them and cheaper than
the estimate by the largest factor yet. The figure is not rounded toward the
estimate. Two things account for it: both premise correction rounds had already
found what would otherwise have been discovered here, and the plan's own
correction of 2026-09-11 named the exact placement trap, so nothing had to be
found by getting it wrong first.

## Accomplishments

- An installer change now earns every check. The three tests that already read `installer/Wixen-Mail-Setup.iss`, and the four this plan added, all run on the commits that could break them.
- The rule survives the combination this project makes most often. An installer change that also bumps the version answers `all`, which it did not before and would not have if the rule had gone one block higher.
- The installer ships `assets\icon.ico` into the program folder, and both `[Icons]` entries and `UninstallDisplayIcon` name it.
- `tests/installer.rs` exists, with a guard record, and `07-07` and `07-08` write into it.
- Nobody's picture changes, and the changelog says so rather than claiming a fix.

## Task Commits

1. **Task 1: An installer script earns the full gate** (RED) `2259552` (test)
2. **Task 1: An installer script earns the full gate** (GREEN) `73548e9`
3. **Task 2: Both shortcuts name an icon the installer put there** (RED) `9d0c460` (test)
4. **Task 2: Both shortcuts name an icon the installer put there** (GREEN) `f28d565`
5. **Ledger** `ff47498` (docs)
6. **This summary, STATE.md and ROADMAP.md** (docs)

**Merge:** `2d6ffeb` on `main`, from branch
`an-installer-change-earns-the-gate-and-the-shortcuts-name-a-shipped-icon`. The
hash is written here by the follow-up commit, since a summary committed before
its own merge cannot name it.

`scripts/check.sh all` on the branch tip, not piped and redirected to a file,
took **419 seconds** and passed all four. The merge itself then earned the full
gate again through the hook, because `main` cannot defer it, and passed.

## What was red, and what was not

Both tasks had a real red half and neither had to be manufactured.

**Task 1, three shell-suite cases.** The names a commit carries are
`which-checks::<the case description>`, and `scripts/red-commit.sh` accepted all
three in `2259552`.

| case | answered | wanted |
|---|---|---|
| `an installer script on a branch` | `affected` | `all` |
| `an installer script beside a source file` | `affected` | `all` |
| `an installer script beside a version bump` | `affected` | `all` |

**Task 2, two tests, both failing on an assertion rather than on a symbol that
was not there.** The subject under test is the installer script, which exists,
so the red comes from the script saying nothing about an icon:

    the installer ships no .ico at all, so a shortcut has nothing to name and
    its picture is whatever the executable's resource table happens to hold

    the installer ships no .ico at all, so there is nothing for Apps and
    Features to be shown

The other two tests in that file were green on their first run and both say so
in their own doc comments. `test_the_desktop_shortcut_is_still_the_users_choice`
is here to stay green: nothing in this plan touched the task, and an edit that
gave the desktop entry its icon by rewriting the line could take the gate off it
without anybody noticing. `test_the_reading_can_tell_a_shortcut_that_names_the_shipped_icon_from_one_that_does_not`
is the companion, and it is green because the reading works.

## The load-bearing negative, and why it is not one of the obvious ones

The plan suggested a markdown-only branch commit answering `docs_only`. That
case is already in the file three times over: `one planning file`,
`several docs` and `a readme` all assert it, and any rule wide enough to take
markdown with it would redden all three. A fourth copy would have proved nothing
the file did not already prove.

What was genuinely untested is whether the rule keys on the extension or on the
folder, so the case added is:

    expect docs_only "a document inside the installer folder" gsd/x installer/README.md

`installer/*` is the obvious way to write "the installer earns the gate" and it
would take a document in that folder with it. No other case in the file would
notice, because `which-checks.sh` never touches the filesystem and the answer
depends only on the path it is handed. That case is green today and after, so it
is not in the red list.

The second negative is `an installer script on main`, which is green because
main's own branch rule already answers `all` for anything that is not a
document. It is in the file so a later reader can see that the new rule is not
what makes it right.

## Every case description is a name a commit can carry

`scripts/shell-suite.sh` refuses an empty description, one holding ` ... ` and
one holding a comma, and it refuses two cases sharing a name. All five new
descriptions passed, which is not an assertion here but the condition of the
suite having run at all: `suite_case_line` exits 70 on any of them, and the
suite reached `suite_verdict`.

## Where the rule went, and the measurement that decided it

**Below the manifest block.** The plan's correction of 2026-09-11 predicted that
a rule placed inside the manifest branch would let an installer change that also
bumps the version answer `affected`, and said to measure it with a fixture diff
rather than argue it. Measured both ways with the suite's own `version_bump`
fixture:

| file list | before | after |
|---|---|---|
| `installer/Wixen-Mail-Setup.iss` | `affected` | `all` |
| `installer/Wixen-Mail-Setup.iss Cargo.toml Cargo.lock` with a version-only diff | `affected` | `all` |
| `Cargo.toml Cargo.lock` with a version-only diff | `affected` | `affected` |
| `docs/changelog.md` | `docs_only` | `docs_only` |

**The correction was right and the mechanism is worth stating, because it is not
where the danger looks.** `only_the_packages_own_version_moved` is called with
the collected manifest paths. A rule that added `*.iss` to that `manifests`
array, which is the natural way to copy the shape of the rule above it, would
hand the installer path to a function that reads a diff of the manifests and
answers about the version line. With a version-only diff that function returns
true, the `!` in the escalation makes the whole condition false, nothing
escalates, and the fall-through answers `affected`. The plan's premise 4 is what
stops that: it says an `.iss` rule is simpler than the manifest rule and should
not copy its shape, because there is no `.iss` equivalent of "only the version
moved". Every line of an installer script is a build input.

**The pattern is `*.iss`.** An `.iss` anywhere in this tree is a setup script,
there is exactly one today, and the wider pattern cannot miss a second one added
somewhere else later. The narrower `installer/*.iss` would have been correct
today and silently wrong the day somebody put a second script elsewhere.

## What `affected` really ran for an installer change, before this

Not nothing, and the plan's premise correction 1 is the reason that matters.
`run_the_tests_that_reach_what_changed` at `scripts/check.sh:387` maps `src/*.rs`
to `--lib module::` and `tests/*.rs` to `--test target`. An `.iss` matches
neither, so no scoped target was chosen. What ran was formatting, clippy, the
three shell suites, and the four targets at `scripts/check.sh:26`:

    guards_that_read_the_whole_tree=(house_style wired the_planning_files_agree_with_themselves the_words_that_say_nothing)

`house_style` is one of those and `ours()` collects `installer/*.iss` at
`tests/house_style.rs:54`, so a prose rule over the installer script was already
checked on every commit. What was never checked is anything the script has to
say. The three tests that read it are all unit tests in `src/`:

    application::forget::tests::test_an_uninstall_that_cannot_run_the_erase_step_says_so
    application::running::tests::test_the_installer_looks_for_the_same_name
    presentation::first_run::tests::test_the_installer_ships_the_page_beside_the_program

All three are named in the comment in `which-checks.sh`, so the next reader can
see the size of what this closes without going to look.

## What the `all` answer costs

**482 seconds, warm.** Measured 2026-09-12 on the commit that changed the
installer script, `f28d565`, by timing `git commit` end to end with the hook
running the whole gate. Conditions, because a duration without them is a number
somebody will quote: one machine, `WIXEN_TEST_THREADS` unset so the
`--all-targets` run takes the harness default, a warm `target/` immediately
after a hand-run of the full suite, and 7,438 tests. The release build inside
that run took 1m16s.

That is the whole cost of the rule, paid only by a commit that changes an
installer script. The last such commit before this branch is not in recent
history at all, which is the same argument the manifest rule made and won on:
the file changes rarely, so `all` costs little in aggregate and is the honest
answer, because every line of it is a build input.

The decision itself costs nothing worth measuring: `bash scripts/which-checks.sh
gsd/x installer/Wixen-Mail-Setup.iss` answers `all` in 0.62 seconds wall, almost
all of it process start on this machine.

## Why task 1 added no guard record

A guard record applies a named edit to a file and names the Rust tests that must
go red. `scripts/guards.py` builds and runs a cargo target to judge it. The
behaviour task 1 added is held by a shell suite, which is a different mechanism
with its own gate: `scripts/check.sh` runs every `scripts/*.test.sh` in every
mode before it looks at anything else, refuses a run where a suite stopped short
of its own verdict, and refuses the commit on a failure in every mode but `red`.
So the absence of a record here is the right shape rather than an omission, and
it is written down so it does not read as one.

## The icon, and what did not change

**The shortcuts were not iconless before.** `build.rs:34` calls
`res.set_icon("assets/icon.ico")`, so `wixen-mail.exe` carries the icon in its
resources, and Inno's own help for the `[Icons]` section says of
`IconFilename`: "If this parameter is not specified or is blank, Windows will
use the file's default icon."
[CITED: jrsoftware.org/ishelp/topic_iconssection.htm] Both shortcuts already
showed the right picture and they still show the same one.

**What changed is what the picture depends on.** It no longer depends on the
executable's resource table being right, which has failed here: the comment at
`build.rs:30` records the executable having been built with no icon at all, so
Windows drew the generic one in the taskbar, in Alt+Tab, on the shortcut and in
Apps and Features. A shortcut naming an installed file survives that happening
again. The changelog entry says exactly this and does not claim the shortcuts
gained an icon.

**The criterion's own wording was wrong by one line.** Success criterion 3 says
this is "`IconFilename` on the two `[Icons]` entries and nothing else". It is
not. `IconFilename` naming a path nothing installs makes Windows fall back to
the default in silence, so the `[Files]` line is part of the change rather than
an extra, and the two have to name the same path.

## The assertion that ties the two halves together

Quoted because the plan asks for it, and because the shape is the point:

```rust
let wrong: Vec<String> = shortcuts
    .iter()
    .filter(|shortcut| shortcut.icon.as_deref() != Some(installed.as_str()))
```

`installed` is built from one `[Files]` line, as `DestDir` followed by the file
name, and `shortcut.icon` is the `IconFilename` parameter of one `[Icons]`
entry. Two separate assertions, one per section, would both pass with the icon
shipped to one folder and named in another. That is the half-fix rather than the
absent one, and it is exactly what somebody would see as a shortcut with no
picture.

The trap the plan named was walked by hand: an assertion that the `[Icons]`
block contains the string `IconFilename` is red today and green after a change
that names a path nothing installs. It is not the assertion that was written.

## What a text-reading test cannot see

The module's own doc comment says it, and it is repeated here because a green
run reads as more than it is:

> `installer/Wixen-Mail-Setup.iss` is the artefact and it cannot be run from
> here. Nothing in this repository compiles it with ISCC, installs anything, or
> looks at a shortcut on a real machine, so every test in this file is a claim
> about what the script says and not one of them is a claim about what an
> install does.

That is `WINDOWS.md` 306.

## UninstallDisplayIcon, and why it moved

It named `{app}\wixen-mail.exe` and so had exactly the dependence the two
shortcuts had. It moves to `{app}\icon.ico` with them, for one reason beyond
consistency: Apps and Features is where somebody goes when they want the program
gone, and an entry showing the generic icon beside a Start menu shortcut showing
the right one reads as two different programs, which is a confusion this
installer already has code to clean up after. It is one line and it has its own
test, so each rule fails under its own name.

## The guard record, measured by hand

**`the icon a shortcut names is the one the installer really put there`.** The
break redirects the destination rather than deleting a line:

    before: Source: "..\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
    after:  Source: "..\assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion

Measured 2026-09-12 by hand on the final tree, after the version bump and the
changelog were in place, with `cargo test --all-targets --no-fail-fast` at eight
threads. **7,436 passed and exactly two failed**, which is the whole red list:

    test_apps_and_features_shows_the_icon_the_installer_ships
    test_the_icon_each_shortcut_names_is_the_one_the_installer_ships

Both are new with this change, so this break measures this guard rather than an
older test catching it by accident. Run through the tool afterwards as well,
which is what confirms the TOML quoting really matches the file:

    scripts/guards.sh "the icon a shortcut names is the one the installer really put there"
    -- all 2 tests named went red, and nothing else did

Written with `'''` rather than `"""`, because a TOML basic string reads a
trailing backslash as a line continuation and both the `before` and the `after`
are full of backslashes. Parsed back with a real TOML reader before committing,
and the `before` string was checked against the file with `grep -F`, because a
break that matches nothing reports that somebody moved the code.

**The count check handles a record whose `file` is not a Rust file by skipping
the count for it, and that was checked rather than assumed.**
`tests/house_style.rs` pushes the guarded file into the set only when it ends
`.rs`. The record's counts therefore come off its red list, which with
`suite = "installer"` resolves to `tests/installer.rs` through
`the_file_a_test_lives_in`. `tests_last_seen` records 4 tests, which is what the
file holds.

## Guard record re-measurement

**None was owed and none was printed.** `tests/installer.rs` is a new file, so
before this plan no record fingerprinted it, measured with the `tests_last_seen`
parse rather than quoted from the plan:

```bash
awk '/^\[\[guard\]\]/ {if(n>0) c++; n=0}
     /^tests_last_seen/ {b=1; next} b && /^\]/ {b=0; next}
     b && /file *=/ && /installer/ {n++}
     END {if(n>0) c++; print c+0}' guards/guards.toml
```

That answered 0 before the record was added and names only the new record now.
No `#[test]` was added to any file a record fingerprints: `tests/installer.rs`
went from absent to 4 tests, and no other file gained or lost one.
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` stayed
green on every commit and `scripts/guards.sh --remeasure` was never printed, so
there was no remedy to run.

`guards/guards.toml` goes 722 records to 723, with the census at lines 79 and 80
bumped 530 to 531 in the same commit that added the record.

## Confirming the new target really runs on an installer commit

Both links were run rather than reasoned about.

    $ bash scripts/which-checks.sh gsd/x installer/Wixen-Mail-Setup.iss
    all

and `all` reaches `cargo test --all-targets --no-fail-fast` at
`scripts/check.sh:531`, which in the gate run of `f28d565` produced:

         Running tests\installer.rs (target\debug\deps\installer-3b501ecc1eb9515b.exe)
    test test_the_reading_can_tell_a_shortcut_that_names_the_shipped_icon_from_one_that_does_not ... ok
    test test_apps_and_features_shows_the_icon_the_installer_ships ... ok
    test test_the_icon_each_shortcut_names_is_the_one_the_installer_ships ... ok
    test test_the_desktop_shortcut_is_still_the_users_choice ... ok

That run also carried `application::running::tests::test_the_installer_looks_for_the_same_name`
and `presentation::first_run::tests::test_the_installer_ships_the_page_beside_the_program`,
which is the other half of what this closes: those two had never run on a commit
that changed the file they read.

## The gate premise, and what every commit really answered

The corrected version is right and the correction mattered. Every commit on this
branch said in its own output what it ran:

| commit | what changed | answer |
|---|---|---|
| `2259552` | `scripts/which-checks.test.sh` | `red` |
| `73548e9` | `scripts/which-checks.sh` | `affected` |
| `9d0c460` | `tests/installer.rs` | `red` |
| `f28d565` | the installer script, both manifests, changelog, guard records | `all` |
| `ff47498` | `.planning/WINDOWS.md` | `docs_only` |

The version bump in `f28d565` did not buy the full gate. The new installer rule
did, which is the rule proving itself on the first commit that could use it.
`scripts/check.sh all` is still owed once before the merge and was run, not
piped, with its output redirected to a file.

Nothing used `--no-verify`. Every commit went through the hook.

## Things the plan said that are no longer true

Two, both drift caused by wave 1 landing between the plan's correction of
2026-09-11 and this execution, and neither changes a conclusion.

**1. The three installer-reading tests have moved again.** Premise 2 gives
`src/presentation/first_run.rs:409` and says the other two are unchanged. It is
now `:495`, because `07-01` added about 86 lines above it in the same file.
`forget.rs:650` and `running.rs:228` are both still right. The premise gives the
grep rather than only the numbers, which is why this cost nothing:

```bash
grep -rn "Wixen-Mail-Setup.iss" src/ tests/ --include=*.rs
```

**2. Premise 3's grep returns two hits now, not one.** The second is a comment
added by `07-01` at `guards.toml:3408`, about the installer page that promised
the Windows warning would go away. The conclusion held and was re-checked
properly rather than by hit count: before this plan,
`grep -c '^file = "installer/Wixen-Mail-Setup.iss"' guards/guards.toml` answered
0 and no record carried `suite = "installer"`.

Two numbers in the plan's own frame have also moved, both for the same reason:
`tests/house_style.rs` is fingerprinted by 19 records rather than 18, and
`.planning/WINDOWS.md` reached 305 before this plan rather than the 301 its
success criteria give. Both are `07-01`'s.

**Everything else held.** `build.rs:34` still sets the icon and the comment at
`:30` still records the executable having had none; `SetupIconFile` is still the
only `.ico` line and still the wizard's own icon; `run_the_tests_that_reach_what_changed`
is still at `scripts/check.sh:387`; `guards_that_read_the_whole_tree` is still at
`:26` with four targets; `ours()` still collects `installer/*.iss` at
`tests/house_style.rs:54`; and `tests/installer.rs` was still fingerprinted by
nothing.

## Deviations from Plan

### Auto-fixed

**1. [Rule 2 - Missing Critical] `UninstallDisplayIcon` moved with the shortcuts**
- **Found during:** Task 2, deciding what the plan asked to be decided
- **Issue:** The plan asks for a decision and does not make one. Left alone, one of the three places naming the program's icon would still have depended on the resource table, and it is the one somebody sees while trying to remove the program.
- **Fix:** `UninstallDisplayIcon={app}\icon.ico`, with its own test so each rule fails under its own name.
- **Files modified:** `installer/Wixen-Mail-Setup.iss`, `tests/installer.rs`
- **Verification:** `test_apps_and_features_shows_the_icon_the_installer_ships`, which was red before the change.
- **Committed in:** `9d0c460` and `f28d565`

**2. [Rule 2 - Missing Critical] A companion for the text reading**
- **Found during:** Task 2, RED
- **Issue:** The plan asks for one test comparing the two paths. A walk over one obedient script passes whether the reading works or has been narrowed until it can see nothing, which is the failure `CLAUDE.md` names for a document guard and which wave 1 measured happening.
- **Fix:** `test_the_reading_can_tell_a_shortcut_that_names_the_shipped_icon_from_one_that_does_not`, over four fixtures, driving the same predicate the real check uses so the two cannot drift.
- **Files modified:** `tests/installer.rs`
- **Verification:** The fixture for the half-fix is the guard record's break, and both refuse.
- **Committed in:** `9d0c460`

**3. [Rule 1 - Bug] The load-bearing negative the plan suggested was already in the file**
- **Found during:** Task 1, RED
- **Issue:** A markdown-only branch commit answering `docs_only` is asserted three times already, so a fourth copy would have proved nothing.
- **Fix:** A document inside the installer folder, which discriminates `*.iss` from `installer/*` and which nothing else in the file covers.
- **Files modified:** `scripts/which-checks.test.sh`
- **Verification:** Green before and after, and it would redden under the wrong pattern.
- **Committed in:** `2259552`

---

**Total deviations:** 3 auto-fixed. No scope creep: two are the plan's own
questions answered, and the third replaces a case the plan named with one that
discriminates.

## Issues Encountered

**`gsd-tools roadmap update-plan-progress 07` wrote wrong values again and was
reverted.** It counted `PLANS-README.md` as a plan, giving 1/10 for a phase with
nine, added a `- [ ] PLANS-README.md` checkbox above the plan list, and replaced
the status note with an empty cell. This is the same defect wave 1 reported, so
it is not a one-off. The whole diff was read before anything was accepted, the
two changes were undone by hand with the editing tool rather than by a checkout,
and the row was then written by hand as 2/9 with a note.

**Nothing else.** The commit gate accepted all five commits first time, which is
worth recording because the previous two plans in this tree both had a gate
refusal to work through.

## Guard sweep owed after the merge

`scripts/guards.sh --touched-by 38f1ba1` selects **one** record, and it is the
record this branch added, which has already been measured by hand tree-wide and
run through the tool on this branch. Counted rather than assumed: no record's
`file` names `scripts/which-checks.sh`, `scripts/which-checks.test.sh`,
`Cargo.toml`, `Cargo.lock`, `docs/changelog.md`, `guards/guards.toml` or
`.planning/WINDOWS.md`, and the only one naming
`installer/Wixen-Mail-Setup.iss` is the new one. So the sweep this branch owes
has nothing left in it, and it does not block the merge.

## User Setup Required

None.

## Next Phase Readiness

- `07-07` and `07-08` both write into `tests/installer.rs`. It is left with four helper functions and one named rule: `section`, `parameter`, `setup_directive`, `the_installed_icon`, `the_shortcuts`, and `every_shortcut_names_the_installed_icon`. A new rule about the script goes beside the last of those, and a new reading goes beside the others. The module doc says what the file can and cannot claim, so neither plan has to re-derive it.
- Every commit either of them makes to the installer script now earns the full gate, which is about eight minutes each on this machine. Both plans should expect that rather than diagnose it.
- `07-08` corrects three pages that say the build is unsigned. The signing guard `07-01` added reads all of them, and `house_style` runs on a documents-only commit, so that plan's document task still wants `cargo test --test house_style` before committing.
- SHIP-03 closes. Criterion 3 closes structurally and is unheard and unseen: `WINDOWS.md` 306 is the real install nobody has run.

---
*Phase: 07-installing-updating-and-what-is-stored*
*Completed: 2026-09-12*
