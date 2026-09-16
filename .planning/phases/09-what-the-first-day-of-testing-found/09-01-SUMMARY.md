---
phase: 09-what-the-first-day-of-testing-found
plan: 01
subsystem: versioning, release workflow, settings
tags: [version, cargo-release, installer, settings-stamp, guards]

requires:
  - phase: 08-every-number-the-project-quotes
    provides: the closed milestone this phase continues, the measurements page the documents cite
provides:
  - "The tree at 1.0.0-alpha.1, with `--version` printing it"
  - "A test on common::version naming the step 0.125.1 to 1.0.0-alpha.1 to 1.0.0 both ways"
  - "A reading of scripts/build-installer.sh's four-field arithmetic in tests/installer.rs, holding 1.0.0.1001 above 0.125.1.4000"
  - "ConfigManager::save writing the running build's version into the settings stamp"
  - "The Release workflow's as-is level, publishing the version the tree carries, with a test and a companion"
  - "One rule for moving inside a prerelease in CLAUDE.md, the changelog, docs/BETA_RELEASE.md and the cutting-a-release skill"
affects: [every later plan of phase 9, which writes changelog entries under Unreleased at 1.0.0-alpha.1 and bumps nothing]

actuals:
  tokens: 9500
  tasks: 3
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A test file reads a shell script's arithmetic off the script's own case arms rather than copying the numbers, so the second copy is held to the first"
    - "A settings file's stamp is written by the save, not carried by the load"

key-files:
  created: []
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/common/version.rs
    - src/data/config.rs
    - tests/installer.rs
    - .github/workflows/release.yml
    - guards/guards.toml
    - CLAUDE.md
    - docs/changelog.md
    - docs/BETA_RELEASE.md
    - .claude/skills/cutting-a-release/SKILL.md

key-decisions:
  - "The bump is one hand edit of Cargo.toml and cargo's rewrite of Cargo.lock, no major level added to the workflow (#46's recommendation)"
  - "as-is is first on the dispatch form and patch and minor last, so the default choice publishes what is there and the two levels that strip a suffix are the furthest from the button"
  - "The re-stamp is a clone at the write with the version replaced; the in-memory struct keeps what load read, because nothing reads the stamp but the file"
  - "The as-is guard record misspells the option rather than deleting the line, because an empty after has no precedent in guards.toml and the misspelling is the shape a real break would take"

patterns-established:
  - "A workflow level whose meaning is a tool's, not this project's, is verified by reading the tool's source where a dry run cannot reach the branch in question"

requirements-completed: [FOUND-01]

coverage:
  - id: D1
    description: "Cargo.toml and Cargo.lock say 1.0.0-alpha.1 and wixen-mail --version prints it"
    requirement: FOUND-01
    verification:
      - kind: integration
        ref: "tests/installer.rs#test_the_version_the_tree_carries_is_at_least_one_point_oh"
        status: pass
      - kind: other
        ref: "WIXEN_MAIL_DATA=<temp> target/debug/wixen-mail.exe --version -> Wixen Mail 1.0.0-alpha.1"
        status: pass
    human_judgment: false
  - id: D2
    description: "The step from 0.125.1 to 1.0.0-alpha.1 to 1.0.0 is ordered both ways, and the installer's four-field version for the first alpha is above the last 0.x version"
    requirement: FOUND-01
    verification:
      - kind: unit
        ref: "src/common/version.rs#test_the_step_from_the_last_0_x_version_to_1_0_0_is_ordered_both_ways"
        status: pass
      - kind: integration
        ref: "tests/installer.rs#test_the_first_alpha_of_1_0_0_sits_above_the_last_0_x_version_the_way_windows_orders_it"
        status: pass
    human_judgment: false
  - id: D3
    description: "A saved settings file names the build that wrote it"
    requirement: FOUND-01
    verification:
      - kind: unit
        ref: "src/data/config.rs#test_a_saved_settings_file_names_the_build_that_wrote_it"
        status: pass
    human_judgment: false
  - id: D4
    description: "The Release workflow can publish the version the tree carries without bumping it"
    requirement: FOUND-01
    verification:
      - kind: integration
        ref: "tests/installer.rs#test_the_release_workflow_can_publish_the_version_the_tree_already_carries"
        status: pass
      - kind: other
        ref: "cargo release 1.0.0-alpha.1 --no-publish --allow-branch the-number-it-will-ship-as (dry run) -> plans the tag v1.0.0-alpha.1 and no bump"
        status: pass
    human_judgment: false
  - id: D5
    description: "Four pages state one rule for moving inside a prerelease, the old wording kept with its date"
    requirement: FOUND-01
    verification:
      - kind: integration
        ref: "cargo test --test house_style && cargo test --test the_words_that_say_nothing"
        status: pass
    human_judgment: false
  - id: D6
    description: "Whether 1.0.0-alpha.1 is published is a dispatch of the Release workflow"
    requirement: FOUND-01
    verification: []
    human_judgment: true
    rationale: "Publishing happens on purpose (guardrail 7); the dispatch is Pratik's and this plan made it possible without making it"

duration: 1h 17m
completed: 2026-09-16
status: complete
---

# Phase 9 Plan 01: The version is 1.0.0-alpha.1 Summary

**The tree moved from 0.125.1 to 1.0.0-alpha.1 by hand once, two tests pin the step and the
installer's four-field order, a saved settings file names the build that wrote it, the Release
workflow gained an `as-is` level that publishes the version as it stands, and four pages say one
rule for how the number moves inside the alpha. Nothing was dispatched, pushed or published.**

## Performance

- **Duration:** about 1h 17m
- **Started:** 2026-09-16T13:12:54Z
- **Completed:** 2026-09-16T14:30Z
- **Tasks:** 3
- **Files modified:** 11

## What landed

`wixen-mail --version`, run from `target/debug` with `WIXEN_MAIL_DATA` pinned to a temp folder,
prints `Wixen Mail 1.0.0-alpha.1`. `grep -c '^version = "1.0.0-alpha.1"' Cargo.toml` is 1 and
`grep -c 'version = "1.0.0-alpha.1"' Cargo.lock` is 1, the lock rewritten by `cargo build` and
not by hand. `search-handler/Cargo.toml` stays at its own `0.1.0`.

**The step, both ways.** `test_the_step_from_the_last_0_x_version_to_1_0_0_is_ordered_both_ways`
in `src/common/version.rs` asserts five things: `compare("1.0.0-alpha.1", "0.125.1")` is `Newer`,
`compare("0.125.1", "1.0.0-alpha.1")` is `Older`, `compare("1.0.0", "1.0.0-alpha.1")` is `Newer`,
`compare("1.0.0-alpha.1", "1.0.0")` is `Older`, `compare("1.0.0-alpha.2", "1.0.0-alpha.1")` is
`Newer`. `cargo test --lib common::version::` passes with 25 tests where there were 24. Green on
arrival, as premise 3 said it would be: the ordering was right already, so this is a pin and its
red half is the guard record below.

**The installer's arithmetic, read rather than copied.** `tests/installer.rs` gained
`the_stage_table`, which reads the stage numbers off the script's own `case` arms
(`*-alpha.*) stage=1`, `*-beta.*) stage=2`, `*-rc.*) stage=3`, `*) stage=4`) and names the first arm
that is missing, and `windows_file_version`, which applies `stage * 1000 + step` with the step capped
at 999. The table test holds the reading to the script's own comment table (`0.5.0` to
`[0, 5, 0, 4000]`, `0.6.0-alpha.1` to `[0, 6, 0, 1001]`, `0.6.0-beta.2` to `[0, 6, 0, 2002]`,
`0.6.0-rc.1` to `[0, 6, 0, 3001]`) and asserts the script still holds `$((stage * 1000 + step))` and
`[ "$step" -gt 999 ] && step=999`. As the test computed them: `1.0.0-alpha.1` is `[1, 0, 0, 1001]`
and `0.125.1` is `[0, 125, 1, 4000]`. The ordering test holds `[1, 0, 0, 1001] > [0, 125, 1, 4000]`
by the derived `Ord` on the array, which compares the four fields as numbers left to right, and
holds first that `1001 < 4000`, so the fourth field alone would have put the old version above the
new and the field-by-field order is the thing worth holding. This is a second copy of the
arithmetic, held to the first by that reading; a companion proves the reading sees a missing arm.
Both green on arrival, because the script was right already; their red half is the record whose
break sets the alpha arm to `stage=2`.

**The major pin.** `test_the_version_the_tree_carries_is_at_least_one_point_oh` reads `Cargo.toml`
through `the_version` and requires a major of at least 1. Red against `0.125.1`
("Cargo.toml says 0.125.1, and the builds going to testers are the stages of 1.0.0"), green after
the bump.

**The settings stamp.** `save_app_config` serialises a clone of the config with `version` set to
`env!("CARGO_PKG_VERSION")`, with a comment saying what the stamp now means. The red test wrote a
file stamped `0.7.7` with `date_wording` `numeric` and `working_day_starts` 6 into a temp folder,
loaded it through `ConfigManager::in_dir`, saved, and read the file back. Before the fix the file
said `0.7.7` after the save (the assertion's `left: "0.7.7"`, `right: "0.125.1"`); after it, the
file says `1.0.0-alpha.1` with the other two fields intact. `load` still keeps the stamp as read,
which the test asserts on the way past, and a second test holds that a file this build created
keeps its own stamp across a save, so the re-stamp is the write and not the load. `config.rs`
held 66 tests on 2026-09-16 before this plan and holds 68. The in-memory struct is left as loaded;
nothing outside `config.rs` reads the field (`grep -rn 'app_config()\.version'` over `src/` finds
nothing).

**The `as-is` level.** `.github/workflows/release.yml` offers `as-is` first, with a comment saying
why it exists, and the `cargo release` step branches on it: the level is replaced by the version
read off `Cargo.toml` with the same `grep`/`sed` line `scripts/build-installer.sh:12` uses, and
`cargo release "$level"` runs as before. `grep -c 'as-is' .github/workflows/release.yml` is 4 (the
description, the comment, the option, the branch). The capture step still decides `prerelease`
from the tag's suffix, so an `as-is` publish of `1.0.0-alpha.1` is a GitHub prerelease, and
`test_an_as_is_publish_of_a_prerelease_is_still_a_prerelease` holds that line unchanged.
`test_a_release_still_happens_only_when_somebody_asks_for_one` passes unchanged.

**The dry run, quoted.** `cargo install cargo-release --locked` installed `cargo-release v1.1.5`,
the same tool `release.yml:90` installs on every release, a developer tool and not a crate in
`Cargo.toml`. On the branch at `23b54d77`, clean tree:

```
$ cargo release 1.0.0-alpha.1 --no-publish --allow-branch the-number-it-will-ship-as
warning: push target `origin/the-number-it-will-ship-as` doesn't exist
  Publishing wixen-mail
warning: push target `origin/the-number-it-will-ship-as` doesn't exist
     Pushing Pushing the-number-it-will-ship-as, v1.0.0-alpha.1 to origin
warning: aborting release due to dry run; re-run with `--execute`
```

No version change was planned, the tag is `v1.0.0-alpha.1`, which begins with the `v` the
`wixen-mail-v*.exe` glob expects, and with `-vv` the planned commands were `git commit -am chore:
Release wixen-mail version 1.0.0-alpha.1`, `git tag v1.0.0-alpha.1 -a -m ...` and `git push
--atomic origin <branch> v1.0.0-alpha.1`. The dry run cannot say whether that commit survives a tree
with nothing to commit, so cargo-release's source was read: `commit_all` in `ops/git.rs` checks the
working tree first and, when nothing changed, logs "No files changed, skipping commit" and returns
success, so the tag follows. The version-as-level shape worked and the `cargo release tag`
alternative was not needed. `cargo release` has no `--allow-dirty`; the first dry run, taken with
the workflow file modified, was refused for that reason and still printed the same plan. **Nothing
was executed, pushed, tagged or published**: `git tag` is empty and `git status` was clean after
each run. `--dry-run` is the default and the flag is superfluous, which the tool says.

**The four pages.** Each keeps its old wording with the date, in its own manner:

- `CLAUDE.md`, Versioning and releases. Old: "Development happens on plain `0.x.y`. Minor for
  feature work, patch for fixes. No suffix." New: "The version the tree carries is the next build to
  go to testers, and since 2026-09-16 that is a stage of `1.0.0`", with the old paragraph quoted as
  what it read until 2026-09-16. Old: "A prerelease suffix stages a release that is about to go to
  people. When builds start going to testers, cut `0.6.0-alpha.1`, then `-alpha.2`, then `0.6.0`
  when that round closes." New: "How a version moves inside a prerelease", the rule in the README's
  words, with the old sentence quoted and the note that it did not say when the counter moves. A
  third paragraph says `patch` is never dispatched while the version carries a suffix and why
  `as-is` exists. "Alpha as a state of the product belongs in the product, not in the number" stands
  as it was. `grep -n '0\.6\.0-alpha\.1' CLAUDE.md` finds the old example once, inside the dated
  sentence.
- `docs/changelog.md:5`. Old: "Development happens on plain `0.x.y`, because `0.x` already means
  unstable... A suffix like `0.6.0-alpha.1` stages a release that is about to go to testers... as
  `0.5.0+g64c73dd`". New: the 1.0.0 stages, the counter rule, `1.0.0-alpha.1+g64c73dd` as the build
  identifier, and the old sentence dated. The `[Unreleased]` entry says the number moved from
  `0.125.1` to `1.0.0-alpha.1` on Pratik's decision of 2026-09-15 and why, that nothing a person
  uses changed but the number they see, that the workflow can publish the version as it stands,
  that a settings file now says which build last wrote it, and under Known limitations that no
  `1.0.0-alpha.1` has been published.
- `docs/BETA_RELEASE.md`. The level list gains `as-is` with what it does. Old: "Wixen Mail develops
  on a plain `0.x.y` version with no suffix... A prerelease suffix is added only when a build is
  about to go to testers". New: the tree's version is the next build to go to testers, the counter
  rule, `as-is` is the level an alpha uses, never `patch` on a suffix, and the old sentence dated.
  "What the workflow does" says `as-is` has no bump and no commit, only the tag.
- `.claude/skills/cutting-a-release/SKILL.md`. Seven levels; "Every level but `as-is` bumps", with
  the note that until 2026-09-16 the page listed six and every one bumped; the "Check `Cargo.toml`"
  paragraph says why it matters more inside a prerelease and names the `patch` trap; the table gains
  `as-is` as a prerelease when the version carries a suffix and a full release when it does not,
  with the sentence that the suffix decides; "Before dispatching" gains the two lines the plan
  asked for.

## Task commits

| Commit | What |
|---|---|
| `120af171` | test(09-01): the red half of task 1, naming `data::config::tests::test_a_saved_settings_file_names_the_build_that_wrote_it`, `test_the_version_the_tree_carries_is_at_least_one_point_oh` and the count check |
| `01ff57bf` | feat(09-01): the bump, the lock, the re-stamp, three guard records, the sixteen re-measured |
| `ccec3d81` | test(09-01): the red half of task 2, naming `test_the_release_workflow_can_publish_the_version_the_tree_already_carries` and the count check |
| `23b54d77` | feat(09-01): the `as-is` level, its record, the five installer records re-measured |
| `9d4e7412` | docs(09-01): the four pages and the changelog entry |
| `c0606807` | Merge 09-01 into `main` |

Branch `the-number-it-will-ship-as` from `main` at `d30379c2`. Not pushed.

## Honest RED and GREEN

Task 1's red commit names two tests that were red and says the other three were green on
arrival: the step pin (premise 3), the arithmetic reading and its ordering test (the script was
right already), and the default-stamp test (`load` never rewrote the stamp). Task 2's red commit
names the reading and says the companion and the prerelease-capture test were green on arrival.
Task 3 is documents only, which `CLAUDE.md` lists among the exceptions to test-first; the
document-reading targets are what hold the pages. Both red commits also name
`test_every_guard_record_says_how_many_tests_the_files_it_names_held`, which fires when a file a
record names gains a test, as `CLAUDE.md` says to.

## What the gate selected

| File | On the branch |
|---|---|
| `src/common/version.rs` | `--lib common::version::` (red commit) |
| `src/data/config.rs` | `--lib data::config::` (red and green commits) |
| `tests/installer.rs` | `--test installer` (both red commits) |
| `Cargo.toml`, `Cargo.lock`, `guards/guards.toml` | no target; the whole-tree guards ran, and the installer target's reading of `Cargo.toml` is what reached the bump |
| `.github/workflows/release.yml` | `all`: the whole gate, five of CI's seven jobs |
| `CLAUDE.md`, `docs/changelog.md`, `docs/BETA_RELEASE.md` | the ten document-reading targets (`docs_only`) |
| `.claude/skills/cutting-a-release/SKILL.md` | nothing maps to it; it rode the `docs_only` commit and `the_words_that_say_nothing` reads it |

`scripts/check.sh all` on the branch at `9d4e7412`, run once and not piped: exit 0, 7,789 passed
and none failed over 60 result lines, 352 s, the release build included. Ten more than the 7,779
on the last whole gate: one in `version.rs`, two in `config.rs`, seven in `installer.rs`. The
keyring race (ledger 374) did not appear. `main`'s hook ran `all` again on the merge.

## Guard records

825 records by the TOML reader before, 829 after; census 802 + 23 before, 802 + 27 after, the
line at `guards/guards.toml:84` moved in the commits that added records.

| Record | Break | Red | Measured |
|---|---|---|---|
| a prerelease stays below the release it stages | `Release` moved to the front of the `Stage` enum | 4 tests in `common::version::tests`: the step pin, the release-newer test, the tag test, and the offer-decision table | `--remeasure`, whole library, 2026-09-16 |
| a saved settings file names the build that wrote it | the write copies the loaded stamp through again | the one new test | `--remeasure`, whole library |
| the installer's alpha arm is the first stage | `*-alpha.*) stage=2` in `scripts/build-installer.sh` | `test_the_four_field_version_follows_the_scripts_own_table`; the ordering test stays green because the first field decides it | `--remeasure`, `--test installer` |
| the release workflow offers a level that publishes the version as it stands | `- as-is` misspelled `- as_is` on the form | the one new reading | `--remeasure`, `--test installer` |

**The first record was written short and the runner said so.** Its red list named three tests;
`--remeasure` reported "1 test went red that this record does not name:
`test_whether_a_release_is_an_offer_depends_on_the_channel_somebody_chose`", because the offer
decision routes through the same comparison and a row of its table compares a release with its
own prerelease. The list was corrected to what the runner reported, the comment says so, and the
record was measured again to earn its counts: "all 4 tests named went red, and nothing else did".
Nothing in `src/service/update_check.rs` went red, which the first draft of the comment had
predicted it would; the comment was rewritten from the measurement. That is the plan's "never
predict" in one record.

**The count check's remedy, run and read.** After task 1 it named 16 records (ten naming
`config.rs`, two `version.rs`, four `installer.rs`); all 16 re-measured through `--remeasure` in
one detached run and all 16 agreed with what they said ("Wrote down the tree 18 records agreed
with", the 18 being the 16 and two of the three new ones). After task 2 it named 5 (the four old
installer records and the alpha-arm record); all 5 agreed. Counts now written: `version.rs` 25,
`config.rs` 68, `installer.rs` 20.

## Premises the tree contradicted

None of the plan's nine premises was wrong. Three things the plan could not know:

1. **cargo-release has no `--allow-dirty`**, so the dry run is refused while the tree holds an
   uncommitted file; it still prints its plan around the error. The clean run was taken after
   the workflow commit. `--dry-run` is the default and the flag is reported as superfluous.
2. **The dry run cannot show whether the commit step tolerates an unchanged tree.** It prints
   `git commit -am ...` without running it. The answer came from the tool's source, quoted above.
3. **The version pin's red list is four tests, not three.** Found by the runner, as above.

## Deviations from plan

**1. [Decision] `as-is` is first on the dispatch form, `patch` and `minor` last.** The plan said
"a seventh level"; the form's first option is its default choice, and `patch` was first. On a
prerelease `patch` is the trap the plan itself names, so the safe level took the first place and
the two that strip a suffix went to the end with a comment saying so. The description line was
rewritten to match. `the_release_levels` reads the list in order and no test depends on the order.

**2. [Rule 1 - Lint] Two clippy findings in new tests**, `field_reassign_with_default` in the
config test and `manual_contains` in the prerelease test, both fixed the way clippy asked before
the commits went through. Neither is a deviation from what the plan asked for.

Everything else executed as written. No scripted edit touched a tracked file: exception set zero,
and it stayed there. Carriage returns measured with `tr -cd '\r' | wc -c` on all four documents
and the workflow: zero on each.

## Threat register

T-09-01 (a `patch` dispatch cutting a full release): the skill, `CLAUDE.md` and
`docs/BETA_RELEASE.md` say never, and `patch` sits last on the form under a comment saying why.
T-09-02: the four pages and the changelog entry moved in one commit, dated. T-09-03: `--execute`
was never run, nothing pushed, `git tag` empty. T-09-04: the companion splices the option out of
the real workflow text and the reading names it. T-09-SC: no crate added; `cargo-release v1.1.5`
installed with `--locked`, the version named.

## Known stubs

None. Every test named here runs in a target the gate selects, and the re-stamp is reached by every
save the running program makes.

## Not done here, on purpose

No `1.0.0-alpha.1` has been published, tagged or pushed. The first alpha is a dispatch of the Release
workflow with `as-is`, and it is Pratik's. The 18 commits unpushed before this plan are 25 now.

## Self-Check: PASSED

Files: `Cargo.toml` says `1.0.0-alpha.1`; `Cargo.lock:6805` the same; the four documents and the
workflow hold what this summary quotes. Commits `120af171`, `01ff57bf`, `ccec3d81`, `23b54d77`,
`9d4e7412` and `c0606807` are in `git log --oneline` on `main`.
