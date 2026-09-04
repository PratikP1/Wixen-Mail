---
phase: 03-mail-at-scale-on-the-wire
plan: 02
executed: 2026-09-04
status: complete
tasks: 2
requirements: [SCALE-02]
subsystem: presentation, gate
tags: [census, guard-record, source-reading, scale-02]
commits:
  - f0ea0d3 test(03-02) a census that counts the sign-ins going round the helper
  - eb9bfdd feat(03-02) the count the tree holds, which is twelve
  - 56284f2 test(03-02) couple the census to the file it watches, and find a case that pins the tree
  - 2734570 fix(gate) ask the exclusion cases about one name rather than the whole answer
  - b66e117 docs(03-02) point SCALE-02 at the test that knows the count
merged: not merged, and not pushed
key-files:
  created:
    - tests/one_sign_in_per_piece_of_work.rs
  modified:
    - guards/guards.toml
    - scripts/check.test.sh
    - .planning/REQUIREMENTS.md
requires: []
provides: ["a test that knows how many places sign in for themselves"]
affects: ["03-06, which is what makes the number fall and now has something that will say whether it did"]
decisions:
  - "The census carries line numbers through the test-half cut rather than reporting positions in the cut text, so a site is named by the line somebody can open."
  - "Two facts make a bypass, a controller built and IMAP connected before the next one is built, rather than a single grep."
  - "The exclusion cases in scripts/check.test.sh ask whether a name is absent rather than whether the whole answer is empty."
metrics:
  duration: about 2 hours
  files: 4
  commits: 5
actuals:
  tokens: 6800
  tasks: 2
  commits: 5
---

# Plan 03-02: The count of sign-ins going round the helper

**One-liner:** How many places in the main window build their own
`MailController` and connect with it is now a test that reads the tree, names
the sites it finds, and runs on the commits that could change the number.

## What works

`tests/one_sign_in_per_piece_of_work.rs` reads the shipping half of
`src/presentation/wx_app.rs`, finds every place that builds a controller and
connects to IMAP before the next one is built, and asserts the total. Twelve on
2026-09-04. When it disagrees it prints every site it found with the line it is
on, and says whether the number went up or down and what to do about each case.

Six companions say the counting is a reading rather than a constant. Made-up
source with two sign-ins answers two; with none, none; a controller that
connects to something other than IMAP is not one; a sign-in through
`a_session_at` is not one; one inside a `#[cfg(test)]` module is not one; and a
site is named by its line in the file rather than by where it lands after the
cut. Replacing the counter's body with a constant was run by hand and took all
six red, against the two the plan asked for.

`guards/guards.toml` couples the census to `src/presentation/wx_app.rs`, which
is what makes it run on the commits that could change the number instead of only
on the commits that change the census.

`.planning/REQUIREMENTS.md` no longer holds the count. SCALE-02's evidence names
the test, keeps the history that argues for it, and names the worst case as a
case.

## Verification

Every commit went through `scripts/check.sh` by way of the commit hook. Nothing
used `--no-verify`.

**Task 1 red, and it is a real red.** The census was written with the
requirement's own number, eight, and failed against the twelve in the tree,
naming all twelve. `scripts/red-commit.sh` accepted it: the one named test ran
and failed and nothing else did. The green commit changed eight to twelve.

**The guard record was measured by hand before it was written.** The break
routes the mark-read site, SCALE-02's own worst case, through
`application::mail_session::a_session_at`. What went red:

- the census, reading eleven where twelve were counted, naming the eleven
- nothing else

`tests/house_style` 64 passed, `tests/wired` 61 passed, and the library 6,078
passed with none failing. **That is the finding rather than an aside.** This
part of `wx_app.rs` had nothing reading it for this property before now, so the
whole defence of the count is the one test the record names, and a run that
reddened only that one is what says so.

**The coupling was proved by hand, both ways:**

```
$ scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_app.rs
one_sign_in_per_piece_of_work
$ scripts/check.sh --suites-for guards/guards.toml src/application/mail_sync.rs
(nothing)
```

**No guard re-measurement was owed.** The count check never printed a
`--remeasure` command, because nothing changed the test count of any file an
existing record names. The two fixtures in the census lost their inner test
attributes for that reason: `how_many_tests_are_in` counts a line by its exact
text wherever it sits, including inside a string, so the fingerprint this record
carries would have claimed nine tests in a file holding seven.

**Task 2 is documents only,** confirmed rather than assumed:

```
$ scripts/which-checks.sh worktree-agent-adfe9fd1f79320a27 .planning/REQUIREMENTS.md
docs_only
```

`grep -n '^version' Cargo.toml` reads `version = "0.47.0"`, unchanged.

## Deviations from plan

**1. The plan's own line numbers were stale before it was executed.** The plan
and `03-RESEARCH.md` name the sites at 7555, 8124, 8405, 8627, 8803, 16289,
16450, 17509, 18249, 18350, 18439 and 18713, measured 2026-09-03. Five of those
are still right. The other seven had moved twenty-one lines by 2026-09-04, to
16310, 16471, 17530, 18270, 18371, 18460 and 18734, because `wx_app.rs` grew
between the research and this plan. The count of twelve is unchanged, and the
census was built against what is really there rather than against the list.

This is the plan's own thesis happening to the plan, one day after the
correction that was written to stop it, and it is worth more than the fix: the
number had now gone stale three times, not twice.

**2. The same drift was found in a second citation in the same paragraph.**
SCALE-02 says `watch_folder` is reached from `wx_app.rs` line 17212. It is at
17233, the same twenty-one lines. Corrected to name `spawn_mail_watch` instead.
Nobody was watching that one either, and it is not about the count, so nothing
this plan built would have caught it.

**3. `scripts/check.test.sh` had to be corrected, and the correction is a
finding.** [Rule 3, blocking] The record refused to commit: a case there
asserted that the whole answer for `wx_app.rs` is empty, and this is the first
record coupling that file to a target no scoped run reaches on its own, which is
the entire purpose of the mapping. The case went red for somebody having used
the feature.

The suite had already been caught by this shape once. Its own header records
equality breaking within the hour and the positive cases being narrowed to
containment, and then writes down an exemption in prose: the exclusions stay
exact, because an exclusion really is a claim about the whole answer. It is not.
It is a claim about one name. The exemption was where the rule broke next.

Committed as red, naming the case, then narrowed to `expect_not_among`. Taken
red by hand afterwards with `guards_that_read_the_whole_tree` emptied in
`check.sh`: both narrowed cases fail, so narrowing did not make them blind.

**4. A duplicate check was written and thrown away.** Narrowing those two cases
gives up the other half of the question, so a fixture was written to prove the
exclusion still fires. The suite already had it, further down the same file: "a
target the run already ends with is dropped and the other kept". The fixture was
deleted and the comment now points at the case that exists. The finding is that
the complement of a check is often already in the same file, below where you are
reading.

## What this cannot see

The census reads source. It says where a sign-in is written, not whether that
code is reached, and nothing about how long a session is held open or whether
one is reused. Those are 03-06's.

`code_of` strips a `//` comment and nothing else, so a `/* */` comment could in
principle carry a construct past it. It cannot invent a site, because a site
needs a construction and a connect on two lines of real code, and a companion
proves a `//` comment naming both halves is not counted.

The window for the last controller in the file runs to the end of the shipping
half, so a connect written far below an unconnected controller with nothing
between them would be attributed to it. Nothing in the file is shaped that way
today.

## Found and not fixed

`src/presentation/wx_app.rs` around line 23031 reports findings as
`{path}:{at + 1}` where the index comes from `what_ships(&text).lines()
.enumerate()`, over every Rust file under `src`. Those numbers are the file's own
line numbers only while nothing was cut above the finding. For any file with a
`#[cfg(test)]` item above a `send_status` line, the reported position is short by
however many lines were deleted, silently. It is correct today for the files it
reports on, and it is the precedent a new source-reading check would copy. Out
of this plan's scope; recorded here rather than fixed.

## Owed

`scripts/check.sh all` before the merge, and the phase-end guard sweep. Neither
was run here: this branch is not merged and not pushed, and `CLAUDE.md` puts
guard re-measurement off the critical path at one sweep per completed phase.

## Self-Check: PASSED

- `tests/one_sign_in_per_piece_of_work.rs` exists and holds 7 tests, all passing.
- `guards/guards.toml` holds the record, and `scripts/check.sh --suites-for`
  answers `one_sign_in_per_piece_of_work` for `src/presentation/wx_app.rs`.
- All five commits are in `git log`: f0ea0d3, eb9bfdd, 56284f2, 2734570, b66e117.
- The working tree is clean.
