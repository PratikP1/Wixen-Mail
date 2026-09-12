---
phase: 07-installing-updating-and-what-is-stored
plan: 03
subsystem: ui
tags: [accessibility, screen-reader, msaa, uia, disclosure, guards, platform]

requires:
  - phase: 01-folders-and-conversations
    provides: the first-run screen's the_window_that_draws_it, the pattern for a test that reads a window file as text
  - phase: 04.2-what-was-built-and-never-reached
    provides: the reading that found NativeBridgeStatus had no caller
provides:
  - A single answer to what this build's accessibility layer does not do, covering both halves of the bridge
  - screen_reader::ANNOUNCEMENTS_REACH_A_SCREEN_READER, supplied by whichever native module compiled
  - names::A_NAME_REACHES_THE_ACCESSIBILITY_TREE, the first marker the accessible-name half has had
  - A native module for the case where there is no bridge, so deliver has one shape rather than two
  - The About dialog and the startup line, both reading the same answer from one source
  - NativeBridgeStatus, ScreenReaderBridge::status and Accessibility::native_bridge_status removed
affects: [07-06]

actuals:
  tokens: 8306
  tasks: 2
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A per-platform module supplying a bool constant, read by a pure function that never asks what platform it is"
    - "A reading that refuses a caller supplying its own answer, with a fixture per way of supplying one"

key-files:
  created:
    - src/presentation/accessibility/platform_bridge.rs
  modified:
    - src/presentation/accessibility/screen_reader.rs
    - src/presentation/accessibility/names.rs
    - src/presentation/accessibility.rs
    - src/presentation/wx_app.rs
    - src/main.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml

key-decisions:
  - "The two accessors are removed rather than given a caller, because the constant says the same thing from the same source and has two real callers, so keeping the enum would have left two representations of one fact with the uncalled one still uncalled"
  - "screen_reader.rs gets a second native module and names.rs gets two cfg arms, because there is no per-platform code in names.rs to put in a module, only a fact about what wxWidgets does underneath"
  - "The startup line goes on the stream, not to a dialog and not to a log, and sits before the running claim so a start that then stalls has still said it"
  - "--help does not carry it: HELP is a const and its other four honesty paragraphs are true of every build while this one is not"
  - "All the tests live in platform_bridge.rs rather than being spread across three files, which cost 0 guard record re-measurements at the time instead of 7"
  - "Task 1's guard break is the half-fix rather than the platform comparison the plan prescribed, because the platform comparison is unobservable on Windows"
  - "The About dialog grows to its contents only when there is a disclosure, so a Windows dialog is byte for byte the one it was"

patterns-established:
  - "A compile-time fact exposed as a module-level constant taken from a per-platform arm, so a port changes the answer without anybody editing the code that says it"
  - "A text-reading check whose predicate refuses the near-miss as well as the absence, with a fixture for each way of writing the near-miss"

requirements-completed: [SHIP-06]

coverage:
  - id: D1
    description: "On a build with no accessibility bridge, one answer names both halves: that nothing announced is spoken or brailled, and that controls with no visible label reach the accessibility tree with no name"
    requirement: "SHIP-06"
    verification:
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_a_build_with_no_bridge_names_both_halves"
        status: pass
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_a_build_that_only_cannot_announce_says_only_that"
        status: pass
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_a_build_that_only_cannot_name_a_control_says_only_that"
        status: pass
      - kind: other
        ref: "scripts/guards.sh 'the disclosure names both halves of the accessibility bridge'"
        status: pass
    human_judgment: false
  - id: D2
    description: "A build with both halves of the bridge says nothing, so the disclosure costs nothing where it works"
    requirement: "SHIP-06"
    verification:
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_a_build_with_both_halves_of_the_bridge_has_nothing_to_disclose"
        status: pass
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_the_two_facts_this_build_reports_are_the_ones_its_platform_supplies"
        status: pass
      - kind: other
        ref: "scripts/guards.sh 'the startup line says nothing on a build whose accessibility bridge is whole'"
        status: pass
    human_judgment: false
  - id: D3
    description: "The answer is derived from which platform module compiled rather than from a platform comparison, so a third module carries its own answer"
    requirement: "SHIP-06"
    verification:
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_the_two_facts_this_build_reports_are_the_ones_its_platform_supplies"
        status: pass
    human_judgment: true
    rationale: "The test pins that this build's constants are the ones its own platform should supply, which is the only half that can be run here. That a third platform module would change the answer with no edit to the code that says it is held by the shape of the code and a comment, not by a test: no third module exists to add and there is no compile-fail harness. WINDOWS.md 310."
  - id: D4
    description: "The About dialog says it, which is where the Help menu's own item lands"
    requirement: "SHIP-06"
    verification:
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_the_about_dialog_asks_this_build_what_it_does_not_do"
        status: pass
      - kind: integration
        ref: "tests/theme_reach.rs#build_about_dialog, unedited and still passing"
        status: pass
    human_judgment: true
    rationale: "A text test proves the window asks and draws. It cannot prove the words appear, that they fit the grown dialog, or that a screen reader reads them. No build without the bridge has ever been made. WINDOWS.md 307 and 308."
  - id: D5
    description: "Starting the program says the same thing in the same words, once, on the stream, before anybody has opened a menu"
    requirement: "SHIP-06"
    verification:
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_starting_the_program_says_it_before_anybody_has_opened_a_menu"
        status: pass
    human_judgment: true
    rationale: "The reachability from process start to the stream is named function by function below and every hop is a non-test path. Nothing here runs a process, and the only build on which the line is not empty does not exist. WINDOWS.md 307."
  - id: D6
    description: "The two accessors nothing had ever called are gone, and the reading that says so was re-run"
    verification:
      - kind: other
        ref: "grep -rn native_bridge_status src/ tests/ --include=*.rs"
        status: pass
    human_judgment: false
  - id: D7
    description: "The wording says what does not work without calling the program inaccessible and without promising a port"
    verification:
      - kind: unit
        ref: "src/presentation/accessibility/platform_bridge.rs#test_the_disclosure_says_what_does_not_work_without_judging_the_whole"
        status: pass
    human_judgment: true
    rationale: "The test refuses three judgements and four promises by name. Whether four paragraphs land as useful information or as a wall of apology in front of somebody who has just started a mail client is guardrail 5's question and only a person using a screen reader can answer it. WINDOWS.md 309."

duration: 89min
completed: 2026-09-12
status: complete
---

# Phase 7 Plan 03: The program says which parts of its accessibility layer do nothing here Summary

**Both halves of the accessibility bridge now say whether they work on the build they are running as, each from whichever platform arm compiled, and the answer is said in the About dialog and at startup. On Windows nothing changes, because there is nothing to disclose.**

## Performance

- **Duration:** 89 min
- **Started:** 2026-09-12T08:42:06Z (branch created)
- **Completed:** 2026-09-12T10:11:47Z (merge measured)
- **Tasks:** 2
- **Files modified:** 9, of which one is new

`actuals.tokens` is 8,306, taken as the characters of the added and removed
lines of the whole branch divided by four: 33,227 characters, excluding
`.planning`. The command, so the next reader re-runs it rather than trusting the
figure:

    git diff 5da0702..HEAD -- . ':(exclude).planning' | grep '^[+-]' | wc -c

Against an estimate of 74,000 with `raw_tokens` 62,000, that is a ratio of about
0.13 on raw. The five most recent summaries give 0.10, 0.13, 0.17, 0.15 and
0.20, so this sits in the middle of its neighbours and the estimate was high by
about the same factor they all were. The figure is not rounded toward the
estimate.

## Accomplishments

- A person starting Wixen Mail on a build with no accessibility bridge is told, on the stream, that part of what it tells a screen reader does not work there, and what that means in practice.
- The same words are in the About dialog, which is where the Help menu's own item lands, so the fact is somewhere to come back to rather than something that scrolls past.
- The accessible-name half of the bridge has a marker for the first time. It had none, and it is the larger of the two failures.
- Both answers come from whichever platform module compiled rather than from `cfg!(target_os = "windows")`, so a real bridge removes the warning without anybody remembering to.
- `NativeBridgeStatus`, `ScreenReaderBridge::status` and `Accessibility::native_bridge_status` are gone. `grep -rn native_bridge_status src/ tests/` returns nothing.
- `deliver` has one shape rather than two: the `#[cfg(not(target_os = "windows"))]` arm whose whole body discarded its arguments is replaced by a fallback module carrying the same names.

## Task Commits

1. **Task 1: One answer, said in the About dialog** (RED) `761dd65` (test)
2. **Task 1: One answer, said in the About dialog** (GREEN) `a385d36` (feat)
3. **Task 2: The same thing at startup** (RED) `56861fa` (test)
4. **Task 2: The same thing at startup** (GREEN) `7ef6f9d` (feat)
5. **Ledger** `381f083` (docs)
6. **This summary, STATE.md, ROADMAP.md and REQUIREMENTS.md** (docs)

**Merge:** `85d84d3` on `main`, from branch
`the-program-says-which-parts-of-its-accessibility-layer-do-nothing-here`.

`scripts/check.sh all` on the branch tip, not piped and redirected to a file,
took **419 seconds** and passed all four. The merge itself then earned the full
gate again through the hook, because `main` cannot defer it, and passed in
**426 seconds**.

## The disclosure, whole, as somebody reads it

Rendered from the four constants in the order `what_is_missing` pushes them, not
retyped:

    Some of what Wixen Mail tells a screen reader does not
    work in this build.

    Wixen Mail announces what it is doing through a Windows
    call that has no counterpart here, so nothing it announces
    is spoken or sent to a braille display.

    Wixen Mail names the lists, trees and fields that carry no
    visible label through a Windows accessibility object that
    has no counterpart here. A screen reader reads those
    controls as "list" or "tree" with nothing to say which one.

    What is named above works on Windows. Nothing in this
    build makes it work here.

The two middle paragraphs are the two halves of the bridge and the plan asks for
both in full. The wording is taken from what the two modules' own headers
already record: `screen_reader.rs`'s about `UiaRaiseNotificationEvent` carrying
braille as well as speech, and `names.rs`'s about a list, a tree or a text field
with no label beside it, which is the measurement somebody took through both
Windows accessibility APIs against the running composer.

The closing line is worded so it is true of one missing half as well as of two.
The sentence building is a function of two independent facts, so a half written
bridge is a state it has to be able to describe, and "both of these" would have
been wrong in two of the four cases.

**This proves structure, not experience.** Every sentence above has only ever
been produced from arguments a test on Windows chose. Nobody has heard any of
it.

## What was red, and what was not

**Task 1: seven red, one green.** The stub was `what_is_missing` returning
`None` for every pair plus both platform constants hardcoded `false`, so every
red came from an assertion rather than from a missing symbol.

| test | stub it ran against | failed on | verdict |
|---|---|---|---|
| `test_a_build_with_both_halves_of_the_bridge_has_nothing_to_disclose` | `None` for every pair | the paired positive: `(true, false)` had nothing to say | red |
| `test_a_build_with_no_bridge_names_both_halves` | same | `expect` on `None` | red |
| `test_a_build_that_only_cannot_announce_says_only_that` | same | `expect` on `None` | red |
| `test_a_build_that_only_cannot_name_a_control_says_only_that` | same | `expect` on `None` | red |
| `test_the_disclosure_says_what_does_not_work_without_judging_the_whole` | same | `expect` on `None` | red |
| `test_the_two_facts_this_build_reports_are_the_ones_its_platform_supplies` | both constants hardcoded `false` | assertion, left `false` right `true` | red |
| `test_the_about_dialog_asks_this_build_what_it_does_not_do` | `wx_app.rs` untouched | assertion, the file never asks | red |
| `test_the_reading_can_tell_a_caller_that_asks_from_one_that_does_not` | the reading, which is finished code | nothing | **green against the stub** |

The green one is the companion and is here to stay green. It drives the reading
over fixtures rather than the sentence building.

**The trap the plan named was walked.** "The About dialog does not mention the
bridge" is green on Windows both when the disclosure is correctly silent and
when it was never written. What is asserted instead is the pure function's
output for the bridgeless pairs, and that `wx_app.rs` names the accessor.

**Task 2: one red, none green.** The stub is `src/main.rs` as it stood, which
asks nothing.

| test | stub it ran against | failed on | verdict |
|---|---|---|---|
| `test_starting_the_program_says_it_before_anybody_has_opened_a_menu` | `src/main.rs` as it stood | the first assertion, the reading found no question in it | red |

## The companion found something on its first run, and the fix went to the reading

`test_the_reading_can_tell_a_caller_that_asks_from_one_that_does_not` failed on
its first run, on this fixture:

    let missing = platform_bridge::what_this_build_does_not_do().unwrap_or_default();

The reading listed four exact ways to supply an answer where there is none:
`.or(`, `.or_else(`, `.unwrap_or(` and `.unwrap_or_else(`. `.unwrap_or_default()`
is a fifth and matched none of them, so the reading said the caller was fine.
`.unwrap_or` is now a prefix rather than a whole call, which covers all three of
that family at once, and the constant's comment says why so nobody tidies it
back into four exact calls.

That is the companion doing the only job it has. A walk over two obedient files
passes whether the reading works or has been narrowed until it sees nothing, and
in this case the reading was wrong from the start and both real files happened to
obey it.

## The two accessors: removed, not given a caller

`grep -rn native_bridge_status src/ tests/ --include=*.rs` now returns nothing.
So does the same grep for `NativeBridgeStatus`. Both were run after the change
and both printed no output at all.

Removal rather than a caller, for one reason. The new answer is a compile-time
constant read by a pure function; the enum was a runtime field set from the same
`cfg!` at construction and carrying no information the constant does not. Giving
it a caller would have meant two representations of one fact, and the only
honest caller available was the new one, which does not want an instance: the
About dialog is a pure builder taking a parent and a palette, and
`tests/theme_reach.rs` builds it, so changing that signature to thread an
`Accessibility` through would have broken a file this plan was told not to edit.
It still passes, unedited.

`cargo clippy -D warnings` said nothing about the removal. There was no caller
to break, which is the whole reason the removal is the right answer: the plan
called this guardrails 1 and 3 in the same function, and it closes by the
function going away rather than by a caller being invented for it.

## Reachability, hop by hop, from process start to words on a stream

Every name here is a non-test function on a non-test path. Line numbers are from
the merged tree.

| hop | where |
|---|---|
| the process entry | `main`, `src/main.rs:13` |
| the only arm that falls through | `Command::Run(run) => run`, `src/main.rs:39`. `Help`, `Version`, `Refused` and `EraseAllData` all return or exit inside the match |
| the question | `platform_bridge::what_this_build_does_not_do`, `platform_bridge.rs:76`, called at `src/main.rs:57` |
| the answer | `platform_bridge::what_is_missing`, `platform_bridge.rs:88` |
| the two facts | `screen_reader::ANNOUNCEMENTS_REACH_A_SCREEN_READER` and `names::A_NAME_REACHES_THE_ACCESSIBILITY_TREE`, each from the arm that compiled |
| the stream | `say`, `src/main.rs:481`, then `put`, `:515`, then `wrote_to_stream`, `:537`, which is ungated and is the first thing `put` tries |

**What a person does to see it:** run `wixen-mail` from a terminal on a build
with no accessibility bridge. No flag, no menu, no setting.

The About dialog's hops are `show_about_dialog` at `wx_app.rs:21256`, reached
from `ID_ABOUT` on the Help menu at `wx_app.rs:6849` through the handler at
`:5281`, then `build_about_dialog` at `:21270`, which asks the same question at
`:21300`.

## Why the stream, and what the two rejected channels would have cost

**A `tracing` line** is cheap and is not the criterion. The criterion says the
application says so; a log somebody has to go and find is not that. It would also
have been invisible to the person it is for.

**A dialog** is ruled out by guardrail 5's bounded half. It is a modal
interruption on every start of a program somebody is trying to use, and it is
T-07-11 in this plan's own threat register.

**The stream** reaches a terminal and a redirect. The risk the plan asks about is
`put`'s third step, which is a dialog when the first two found nowhere to write.
That step cannot be reached by this line: the answer is `None` on Windows, so
`say` is never called there, and everywhere else `show_dialog`'s
`#[cfg(not(target_os = "windows"))]` arm at `src/main.rs:468` is an `eprintln!`.
So the fallback is unreachable for this text rather than merely unlikely, which
is the thing the plan asked to have said rather than left as a risk.

## What was decided about `--help`, and why

**It does not carry it.** `command_line::HELP` is a `const &str`, so a
conditional sentence means appending at print time in `main.rs`, which is two
lines. The cost is not the two lines.

The end of `--help` already carries four paragraphs of honesty: what
`--read-only` and `--allow` can and cannot do, that everything which writes is
experimental, and that the downloaded mail is not encrypted. Every one of those
is true of every build. A fifth that is true of some builds and not others makes
the page answer two different questions, and the one it would answer badly is
the one about this program rather than this build.

Against that, somebody who types `--help` on a build with no bridge is told one
command later, on the same stream, when they run it. The criterion asks for
startup and Help, and both are delivered.

**This is a judgement that could go the other way cheaply**, and it is recorded
here rather than hidden so that it can. If somebody decides a person deciding
whether to install should be told before they run anything, the change is two
lines in `main.rs` and one more test.

## The derivation, and what a third platform would take

`screen_reader::ANNOUNCEMENTS_REACH_A_SCREEN_READER` is
`native::ANNOUNCEMENTS_REACH_A_SCREEN_READER`, where `native` is the Windows
module or the fallback one. Nothing in `platform_bridge.rs` asks what platform
this is, except one test whose whole subject is that question.

**What a port would take:** a third `#[cfg(target_os = "macos")] mod native` in
`screen_reader.rs` carrying `true` and the calls that earn it, and a third
`#[cfg(target_os = "macos")]` arm in `names.rs` carrying `true` beside the
`setAccessibilityLabel:` bridge its header already names. The fallback module
already fixes the shape those calls have to have, which is the part that used to
be a `#[cfg(not(...))]` arm nobody could see.

**What it would not take:** any edit to `platform_bridge.rs`, to
`build_about_dialog`, or to `main`. The warning would stop appearing because the
constants changed, not because anybody remembered.

**The asymmetry between the two files is deliberate and is written into the doc
comments.** `screen_reader.rs` has two modules because it has per-platform code.
`names.rs` has two `cfg` arms on one constant because it has none: it calls
`set_accessible` unconditionally and the fact being recorded is about what
wxWidgets does underneath, not about which code compiled. Making it a module
with one constant in it would have been a module for the shape's sake.

**This property is structural, not tested.** No third platform module exists to
add, and `grep -n trybuild Cargo.toml` returns nothing, so there is no
compile-fail harness in which the absence of one could be expressed. That is
`WINDOWS.md` 310 and it is said in the module's own header too.

## The two guard records, measured by hand

Both were taken with `--all-targets --no-fail-fast` at eight threads, on the
final tree with the version bump and the changelog in place, and both red lists
are quoted rather than expected.

**`the disclosure names both halves of the accessibility bridge`.** The break is
the half-fix rather than the absent one: where the accessible-names sentence
belongs, the opening line is repeated instead, so a disclosure still appears and
still names announcements.

    before: said.push(CONTROLS_WITH_NO_VISIBLE_LABEL_HAVE_NO_NAME);
    after:  said.push(SOME_OF_IT_DOES_NOT_WORK);

**7,029 passed in the library and exactly two failed**, with nothing else red
across 7,444 passing tests:

    presentation::accessibility::platform_bridge::tests::test_a_build_with_no_bridge_names_both_halves
    presentation::accessibility::platform_bridge::tests::test_a_build_that_only_cannot_name_a_control_says_only_that

Both are new with this change, so this break measures this guard rather than an
older test catching it by accident. Two tests in the same file stay green and
that is correct rather than a short list, which the record says in its comment:
`test_a_build_with_both_halves_of_the_bridge_has_nothing_to_disclose` asks about
the pair `(true, true)`, which this break does not reach, and
`test_the_about_dialog_asks_this_build_what_it_does_not_do` reads `wx_app.rs`
rather than the sentences.

**`the startup line says nothing on a build whose accessibility bridge is
whole`.** The break makes the answer unconditional rather than deleting the
call, which is the failure that would put a false warning in front of every
Windows user.

    before: if let Some(missing) = platform_bridge::what_this_build_does_not_do() {
    after:  ...what_this_build_does_not_do().or_else(|| Some("Some of what Wixen Mail ...".to_string())) {

**7,031 passed in the library and exactly one failed**, with nothing else red
across 7,446 passing tests:

    presentation::accessibility::platform_bridge::tests::test_starting_the_program_says_it_before_anybody_has_opened_a_menu

**That one was measured twice, and the first measurement is the finding.** The
first run also reddened
`test_every_guard_record_says_how_many_tests_the_files_it_names_held`, from the
task 1 record needing its count corrected, not from the break. A red list taken
in that state would have named a test that has nothing to do with this guard. The
correction was run first, through the remedy, and the break measured again on the
clean tree.

Both records were then run through the tool, which is what confirms the quoting
really matches the file:

    scripts/guards.py "the disclosure names both halves of the accessibility bridge"
    -- all 2 tests named went red, and nothing else did

    scripts/guards.py "the startup line says nothing on a build whose accessibility bridge is whole"
    -- the one test named went red, and nothing else did

Both are written with single-quoted TOML literal strings rather than basic
strings, so no escape is processed, and both were parsed back with a real TOML
reader and checked against the source with an exact match before committing.

`guards/guards.toml` goes 723 records to 725, with the census at lines 79 and 80
bumped 531 to 533 in the same commits that added the records.

## Guard record re-measurement: none at first, then one, and the plan predicted the wrong commit

**The plan budgeted seven records and about eleven minutes.** Its premise 6 says
the `screen_reader.rs` and `names.rs` tests should land in one commit so the
count check fires once over 3 + 4 = 7 records rather than twice.

**It cost nothing there, because no test went into either file.** Every test in
this plan is in `platform_bridge.rs`, which is new and which nothing
fingerprinted. Counted with the `awk` over `tests_last_seen` blocks rather than
with a grep, before the branch:

| file | records fingerprinting it | plan said |
|---|---|---|
| `src/presentation/accessibility/platform_bridge.rs` | 0, it did not exist | 0 |
| `src/presentation/accessibility/screen_reader.rs` | 3 | 3 |
| `src/presentation/accessibility/names.rs` | 4 | 4 |
| `src/presentation/accessibility.rs` | 7 | 7 |
| `src/presentation/wx_app.rs` | 48 | 48 |
| `src/main.rs` | 1 | 1 |

Every figure in the plan's premise 5 table re-derived exactly.

**Then it cost one, through a record this plan created two commits earlier.**
Task 1's guard record names `platform_bridge.rs`, so when task 2 added a ninth
test to that file the count check fired. The commit gate printed it and refused
the red commit until the check was named:

    1 guard record was measured against a tree that no longer holds those tests:
      the disclosure names both halves of the accessibility bridge:
          src/presentation/accessibility/platform_bridge.rs has gained 1 test: it held 8 and holds 9

    scripts/guards.sh --remeasure "the disclosure names both halves of the accessibility bridge"

The remedy was run and its output read rather than its exit code:

    -- the disclosure names both halves of the accessibility bridge
       all 2 tests named went red, and nothing else did

    Wrote down the tree 1 record agreed with, so a test added
    to any file they name fails the commit that adds it.

**The mechanism arrived one commit later than the plan expected rather than not
at all**, and it is worth stating because the shape recurs: a plan that adds both
a test file and a record naming it pays the count check on the next test it adds
to that file, not on the one that created the record.

## Test counts, before and after

No `#[test]` was added to `wx_app.rs`, to `accessibility.rs`, to
`screen_reader.rs` or to `main.rs`. Read from the diff rather than from a grep,
because the checker's count and `grep -c '#\[test\]'` disagree on `wx_app.rs`,
which is why the plan says to read the `tests_last_seen` blocks:

    git diff 5da0702..HEAD -- src/ | grep -E '^[+-].*#\[test\]' | sort | uniq -c
          8 +    #[test]

All eight are in `platform_bridge.rs`, which is the only file in the diff that
gained more than 30 lines, and none was removed anywhere. The count check stayed
green on every commit except the one described above.

| file | `tests_last_seen` before | after |
|---|---|---|
| `src/presentation/wx_app.rs` | 199 | 199 |
| `src/presentation/accessibility.rs` | 23 | 23 |
| `src/presentation/accessibility/screen_reader.rs` | 22 | 22 |
| `src/presentation/accessibility/names.rs` | 32 | 32 |
| `src/main.rs` | 0 | 0 |
| `src/presentation/accessibility/platform_bridge.rs` | absent | 9 |

## The half-built artefact the red commit had to carry

Clippy refused the first red commit, and the refusal was right:

    error: constant `SOME_OF_IT_DOES_NOT_WORK` is never used
    = note: `-D dead-code` implied by `-D warnings`

The four sentence constants are private and the stub `what_is_missing` ignored
them, so under `-D warnings` the red half did not compile. `cargo clippy` without
`--all-targets` builds the library without `cfg(test)`, so the tests that do read
them are not in scope.

They were made `pub` for that one commit, which the unused-item lint does not
police for an item reachable from the crate root, with a comment saying so, and
narrowed back in the green commit once `what_is_missing` read them. Neither a
suppression nor a padded stub: the lint was right, the constants really were
uncalled, and the visibility is the part that was allowed to be temporary.

## Deviations from Plan

### Auto-fixed

**1. [Rule 1 - Bug] The reading missed a fifth way to supply an answer**
- **Found during:** Task 1, RED
- **Issue:** `SUPPLIES_AN_ANSWER` listed four exact calls and `.unwrap_or_default()` matched none of them, so the reading accepted a caller that turns "nothing to say" into something to say. The companion fixture caught it on its first run.
- **Fix:** `.unwrap_or` as a prefix covering the whole family, with the constant's comment saying why it is a prefix so nobody expands it back into exact calls.
- **Files modified:** `src/presentation/accessibility/platform_bridge.rs`
- **Verification:** The companion passes, and the fixture for each of the three near-misses refuses.
- **Committed in:** `761dd65`

**2. [Rule 2 - Missing Critical] The About dialog grows when there is something to draw**
- **Issue:** The dialog is a fixed 380 by 260 with four short labels and a button. Four more paragraphs would have been drawn outside it, so the disclosure would have existed and not been readable, which is the stub presented as complete that guardrail 3 is about.
- **Fix:** `set_sizer_and_fit` when there is a disclosure, `set_sizer` when there is not, so a build with a whole bridge gets the dialog it always had.
- **Files modified:** `src/presentation/wx_app.rs`
- **Verification:** `tests/theme_reach.rs` still passes unedited. Whether the grown dialog lays out properly is `WINDOWS.md` 308 and cannot be checked here.
- **Committed in:** `a385d36`

**3. [Rule 2 - Missing Critical] SHIP-06's evidence asserted the absence of what this built**
- **Found during:** Closing the requirement
- **Issue:** `.planning/REQUIREMENTS.md` said "the accessor is built and nothing calls it", quoted a grep returning one hit, and carried a caveat saying the derivation was still a platform list of one. All of that stopped being true at `a385d36`, and the evidence never mentions the second half of the bridge at all. `CLAUDE.md`'s rule is that the commit building a thing must find the sentences saying it does not exist.
- **Fix:** A dated correction above SHIP-06's evidence naming the four false sentences, leaving them in place because they are what the requirement was measured against.
- **Files modified:** `.planning/REQUIREMENTS.md`
- **Verification:** `grep -rn native_bridge_status src/ tests/ --include=*.rs` returns nothing, which is the sentence the correction replaces.
- **Committed in:** the metadata commit

### Departures from what the plan prescribed

**4. Task 1's guard break is not the one the plan named, and the reason is a measurement**

The plan says to break the derivation by making the answer read
`cfg!(target_os = "windows")` again. That break is **unobservable on this
platform**: on Windows both constants are `true` and the platform comparison is
also `true`, so every test gives the same answer before and after, and the record
would have had an empty red list. A candidate break that reddens nothing is a
finding rather than a failure to paper over, so it is recorded here and a break
that does discriminate was used instead: the half-fix that drops the
accessible-names sentence, which is the failure premise 3 of the plan spends a
paragraph on.

The property the prescribed break was meant to hold, that the derivation follows
the module, is held structurally and is `WINDOWS.md` 310.

**5. The tests are in one file rather than three**

The plan says tests go in `platform_bridge.rs`, `screen_reader.rs` and
`names.rs`, and to land all three in one commit so the count check fires once
over seven records. Putting them all in `platform_bridge.rs` fires none, which
is the cheaper of the two and proves the same things: the constants are read
through `screen_reader::ANNOUNCEMENTS_REACH_A_SCREEN_READER` and
`names::A_NAME_REACHES_THE_ACCESSIBILITY_TREE`, which are module-level and
public, so a test in a third file reaches them exactly as well.

**6. The count check was not named in the first red commit**

The plan says to name
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` among the
first red commit's trailers. It was not, because it did not fire: no test went
into any file a record fingerprinted. Naming a test that passes is refused by
`scripts/red-commit.sh`. It **was** named in the second red commit, where it did
fire, for the reason the plan gives.

---

**Total deviations:** 3 auto-fixed (1 bug, 2 missing critical) and 3 departures
from what the plan prescribed, each with the measurement that caused it. No
scope creep: two of the auto-fixes are correctness requirements of what the plan
asked for and the third is the truthfulness rule `CLAUDE.md` states.

## Issues Encountered

**A shell heredoc corrupted a backslash pair and the run failed twice before it
was diagnosed.** A `python3 - <<'PY'` block carrying a regular expression with
`[^"\\]` arrived with one backslash, which makes the character set unterminated.
The delimiter was quoted, so this is not the ordinary expansion case. This is
exactly what `CLAUDE.md` describes, and the response it prescribes was taken
rather than escaping more carefully: the script was written with the editing tool
and run from a file for the rest of the session.

**`gsd-tools requirements mark-complete` inserted two stray blank lines**, which
is the defect wave 2 reported. It ticked SHIP-06 and set its traceability row to
Complete correctly. One of the two blank lines landed inside SHIP-04's evidence,
which belongs to `07-01`, and was reverted by hand; the other is inside SHIP-06's
own block and was kept. The whole diff was read before anything was accepted.

**`gsd-tools roadmap update-plan-progress` was not run**, because both earlier
waves in this phase reported it writing the wrong fraction for a nine-plan phase,
adding a stray checkbox for `PLANS-README.md` and blanking the status note. The
row and the plan list were written by hand, and
`tests/the_planning_files_agree_with_themselves.rs` was run afterwards to check
the counts agree with the files on disk.

## Things the plan said, re-derived

**Both corrected citations re-derived to exactly what the correction said.**

Premise 9's Help menu. The correction says the old range `6437` to `6473` was the
**Tools** menu at the commit the plan was written against, and to find the Help
menu by symbol instead:

    $ grep -n 'ID_ABOUT' src/presentation/wx_app.rs
    128:    ID_ABOUT,
    5281:                        _ if id == ID_ABOUT => show_about_dialog(&frame),
    6849:            ID_ABOUT,

`6849`, exactly as the correction gives it.

Premise 8's `#[cfg(not(target_os = "windows"))]` arm. The correction says it is
in `show_dialog`, not in `put`, and that `put`'s console step is spelled
`#[cfg(windows)]`:

    $ grep -n 'fn put\|fn show_dialog\|fn wrote_to_stream\|cfg(not(\|cfg(windows)' src/main.rs
    413:fn show_dialog(message: &str, tone: Tone) {
    448:    #[cfg(not(target_os = "windows"))]
    495:fn put(words: &str, tone: Tone) {
    500:    #[cfg(windows)]
    517:fn wrote_to_stream(words: &str, tone: Tone) -> bool {

Every line matches the correction, including the two spellings.

**The corrected gate premise is right and the correction mattered again.** Every
commit on this branch answered `affected` and said so in its own output, and the
two commits that bumped `Cargo.toml` were among them. The old sentence would have
had this report a full gate that never ran.
`scripts/check.sh all` was run once on the branch before the merge, not piped and
redirected to a file, and passed all four.

**Premise 5's whole table re-derived**, counted with the `awk` over
`tests_last_seen` blocks: 0, 3, 4, 7, 48, 1 records, and 22, 32, 23, 199 and 0
tests. So did premise 1's two greps, premise 3's `grep -n 'cfg(target_os'
names.rs` returning nothing, and premise 7's `build_about_dialog` at `21270` with
`tests/theme_reach.rs:837`.

**One thing in the plan is now false and it is the plan's own doing.** Premise 5
says `platform_bridge.rs` is "a file no guard record fingerprints", which was
true when written and stopped being true within this plan, at the commit that
added task 1's record. Premise 6's eleven minutes is therefore right about the
rate and wrong about which commit pays it.

**Nothing else in the plan was found false.** The `--lib` warning in premise 11
was taken: every verification command here is a single `--lib` with the module
paths after `--`.

## User Setup Required

None.

## Next Phase Readiness

- `07-06` is the first thing that could build this crate on Linux or macOS, and it is therefore the first thing that could tell anybody whether these sentences appear at all. It does not answer whether the About dialog draws them, and it cannot answer whether they are useful to hear.
- `07-04` and `07-05` touch none of these files. `platform_bridge.rs` is fingerprinted by one record and `main.rs` by two now, so a test added to either fires the count check; the remedy is one build and one run each.
- SHIP-06 closes. Criterion 6 closes structurally, all three clauses, and is unseen and unheard: `WINDOWS.md` 307 to 310 are the four things nobody has run.
- Anyone writing the macOS or Linux bridge should start at `screen_reader.rs`'s fallback `native` module, which fixes the shape the calls have to have, and at `names.rs`'s header, which names the two calls a port needs.

## Self-Check: PASSED

`src/presentation/accessibility/platform_bridge.rs` and this summary are both on
disk. All six commits resolve: `761dd65`, `a385d36`, `56861fa`, `7ef6f9d`,
`381f083` and the merge `85d84d3`. Every plan-level verification command in
`<verification>` was run and its real output is quoted above, including the two
greps that now return nothing and `cargo test --test theme_reach` passing with
that file unedited.

---
*Phase: 07-installing-updating-and-what-is-stored*
*Completed: 2026-09-12*
