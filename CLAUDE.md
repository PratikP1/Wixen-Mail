# Wixen Mail

A fully accessible, lightweight mail and personal information client built with Rust and
wxdragon (wxWidgets bindings). Windows-first. Screen reader users are the primary audience,
not an afterthought.

## Guiding Principles

The project answers four questions; let them judge every change. **What is it for?** Making
correspondence and personal information legible to people who cannot see it. Messages,
folders, threads, read state, attachments, events, contacts, tasks, notes, and reminders
declared to the platform accessibility API as structured facts, so a screen reader user
works through a full inbox rather than reconstructing it. **What does it strengthen?** Their
independence, and the idea that the *application* declares its meaning instead of the screen
reader inferring it from a rendered DOM. Open protocols too: IMAP, SMTP, CalDAV, and
iCalendar deserve a client as open as they are. **What does it replace?** Outlook,
Thunderbird, and webmail, technically usable with a screen reader and painful in practice.
Not the screen reader, not the mail server, and not Wixen Terminal; those are complements.
**What does it allow to be done poorly?** That question is the source of these guardrails.
A rich accessibility surface makes it easy to mistake *structure present* for *experience
good*, and easier still to mistake a call that looks like accessibility for one that is.

Guardrails (each exists because we got it wrong here at least once):

1. **No feature is done until it runs in production.** Compiling and passing tests is not
   done. It is done when a non-test path reaches it and it is exercised end to end. Check
   reachability before claiming completion. (All eight PIM update variants were handled in
   the UI and sent by nothing; five modules rendered empty in every build.)
2. **Accessibility isn't done until a screen reader confirms it.** Tests prove structure;
   only a real NVDA or Narrator run proves experience, and the automated scan can produce a
   finding against three of the fifty-five WCAG 2.2 AA criteria (`docs/wcag-coverage.md`;
   this line said "about half of WCAG" until 2026-09-14, and a single-line grep for the
   phrase missed it because the line wrapped). Worse, a call can look like accessibility
   and not be one. (Sixteen widgets
   were "named" with `set_name()`, which sets an internal wxWidgets identifier and never
   reaches the accessibility tree. It compiled and passed 324 tests.)
3. **No stubs presented as complete.** If something cannot be finished, say so and gate it.
   Never ship code that looks done and does nothing. (The note editor filled itself with
   "Note 1" and "(Note content loaded here)" on every selection.)
4. **A check nobody reads is worse than no check.** CI failing for months while looking
   maintained, or a scan reporting success while its scan step errored, buys false
   confidence. When a check can fail two ways, make it say which. (Both happened.)
5. **Feedback must be distinct and bounded.** Announcements and audio cues must be
   distinguishable from their siblings and must not flood under a syncing mailbox. Content
   read aloud needs controls and a fast mute, because private mail gets spoken in rooms.
6. **Untrusted input stays untrusted.** Message bodies come from strangers. Sanitizing them
   is security; preserving their heading structure and link text is accessibility. Neither
   excuses dropping the other.
7. **Publishing happens on purpose.** Anything that tags, releases, or pushes outward is
   triggered deliberately, never as a side effect. (A push to `main` cut two releases nobody
   asked for and promoted an alpha to beta.)
8. **Prefer few things excellent over many adequate.** For any new subsystem ask whether it
   is wired, exercised end to end, and raises the bar for the whole, or only adds surface.
   (Six modules shipped at once with one of them working.)
9. **Don't silently absorb upstream failures.** Where this papers over a sender's missing
   alt text, a provider's broken MIME, or a dependency with no accessible name, say so. The
   goal is a better ecosystem, not hidden gaps nobody is pressured to fix.

Fuller rationale in [docs/principles.md](docs/principles.md).

<!-- BEGIN GUARDRAILS (managed by /setup-guardrails) -->

## Guardrails

The standing guardrails in `~/.claude/CLAUDE.md` apply to this project. What follows is this
project's tightening and the concrete commands that back it.

### Test-driven development

Red, green, refactor, on every change. Invoke the `tdd` skill for any implementation, bug fix,
or feature. Write the failing test first, then the minimum code that passes it, then refactor.

```bash
cargo test --all-targets
```

Unit tests live in `#[cfg(test)] mod tests` beside the code they cover. Cross-layer tests live in
`tests/integration_tests.rs`. Async code uses `tokio-test`; anything touching the filesystem uses
`tempfile` rather than real user directories.

**Committing the red half.** The commit gate runs the tests reaching what you changed, so a commit
whose tests fail is refused, and `--no-verify` is not the answer here. A RED commit says so in its
message and is held to it:

```
test(02-02): failing tests for the narrower question set

Fails-until-green: application::saved_searches::tests::test_a
Fails-until-green: application::saved_searches::tests::test_b
```

This is not an exemption, and it costs more to misuse than to use honestly. The tests still run, and
`scripts/red-commit.sh` then requires three things at once: every named test ran, every named test
failed, and nothing else failed. Name a test that passes and the commit is refused. Leave an
unrelated failure in the tree and it is refused. A red commit is therefore stronger evidence than an
unchecked one, because it records which tests were red and proves they were.

A red commit may only be made on a branch. On `main` it is refused: every commit here lands on what
CI builds, and a failing test on it is a broken branch for everybody. The red and its green pair go
on a branch and arrive together at the merge.

**The red gate and the count check collide, and the way through is to name the
count check as one of the failures.** `red-commit.sh` requires that nothing
unnamed failed. Adding a test to a module some guard record names turns
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` red, and
its remedy needs the green code before a record's red list can be corrected. So
there is no ordering where a red commit adding such a test has a clean tree
around it. Measured on 2026-09-02 against `src/data/message_cache/messages.rs`,
which ten records name.

The answer is not to split the commit, because the fingerprint can only be paid
once. It is one red commit covering the whole task, naming the real failures and
the count check together, with the reason in the message. That keeps the red
honest: it still says exactly what failed and is still held to it.

**A case in a shell suite is committed red the same way, and the name is
`<suite>::<the case description>`:**

```
test(gate): failing case for the mode nobody routed

Fails-until-green: which-checks::a branch and a red commit touching only documents
```

The suite name is the file name without `.test.sh`, and the description is the
string the case was written with. Both halves of the gate had to change for that
to work, and they were windows ledger 39 until 2026-09-03. `check.sh` ran every
`scripts/*.test.sh` under `set -e` before it branched on the mode, so a failing
suite stopped the run several branches above `red` and `red-commit.sh` was never
asked; it now collects their output into the same run log the scoped cargo runs
append to, and refuses on it in every mode but `red`. And `red-commit.sh` reads
cargo's `test NAME ... FAILED` lines, which a shell suite never produced, so a
named case reported as never having run; `scripts/shell-suite.sh` now prints one
such line per case, passing as well as failing, so all three conditions hold
across both kinds of test at once. `red-commit.sh` itself did not change.

The passing line is the half worth understanding. Without it, a named case that
passed and a name nobody ever wrote are the same silence, and "every named test
ran" cannot be asked at all.

Three things a suite must therefore do. Report every case, on the passing path
as well as the failing one, through `suite_case_passed` and `suite_case_failed`.
End at `suite_verdict`, which prints the line saying it got there; `check.sh`
refuses a run where a suite stopped short of it, in every mode including `red`,
because the cases it never reached said nothing and a run like that cannot be
judged. And give each case a description that is a name a commit can carry.
`shell-suite.sh` refuses three that are not, where they are written rather than
where they are read: an empty one, one holding ` ... `, which is the separator
between a name and its outcome, and one holding a comma, which is the separator
between two names. Two cases sharing a description are refused as well, because
a commit naming one of them cannot say which.

The comma was found by using this rather than by reading it. The first commit
that tried to name a case named `which-checks::main, code changed`, and the gate
reported two tests called `which-checks::main` and `code changed`, neither of
which had ever run. About ten descriptions were reworded. Guessing at the other
end which commas were separators and which were prose would have been a gate
deciding by heuristic, and every version of that heuristic leaks on a
description whose parts happen to look like test paths.

**A suite the hook runs never acts on the repository that runs it, and since
2026-09-19 that is the harness's promise rather than luck.** Git hands a hook the
environment of the commit in progress, and `check.sh` runs every suite as a
child of the hook: `GIT_DIR`, absolute in a linked worktree; `GIT_INDEX_FILE`,
absolute when a partial commit builds a temporary index; `GIT_WORK_TREE`,
`GIT_PREFIX` and `GIT_COMMON_DIR` where set. `git -C <dir>` changes the
directory and not the repository once one of those is absolute, so a suite that
builds a repository of its own with `git -C` builds it in the real one instead.
That happened twice on 2026-09-18: a commit made through the hook from the
linked worktree `wixen-mail-sweep` let `which-checks.test.sh`'s fixture mark this
repository bare, replace its hooks path and put a stray commit on `main`, all
undone by hand; and a partial commit from the primary worktree handed the same
fixture a temporary index, which it wrote to, and the subject read it and
answered `all`. In the main checkout the relative `.git` had kept the fixture
isolated, by luck. `shell-suite.sh` now unsets the five variables right after
`set -uo pipefail`, before any suite's first `git`, and two cases in
`which-checks.test.sh` are red if that stops, each handing a fresh `bash` the
harness and one variable and asking whether a throwaway repository, `elsewhere`,
moved. Neither case points at this repository. The gate's own decision is made
before the suites run, in the hook's process, and is untouched. Ledger 536 and
#85; the second incident's rule, never `git commit --only <paths>` while another
process may commit in the same checkout, still stands, because a partial commit
takes no `index.lock`.

**One kind of shell change still cannot be split, and it is this mechanism
itself.** A case asserting how the red gate treats shell suites is red until the
gate treats them that way, and committing it red needs the very thing it is
about. The commit that closed ledger 39 was therefore one commit, and said so.

This existed as a hole for one day and nobody noticed, which is the point of writing it down here.
Phase 1 committed failing tests freely because no hook was installed. Turning the hook on closed the
gate, the next plan worked around it by recording red as a measurement in its summary rather than
reporting a rule it could not follow, and the workaround was better evidence than the rule required
but it was still a workaround. A rule that lives only in a document is one somebody has to notice
being broken.

**This binds GSD too, and GSD disagrees by default.** GSD Core is installed here and plans work
through `.planning/`. Its planner decides per task whether the test comes first, and with
`workflow.tdd_mode` off, which is its default, it decides opportunistically. That is not the
rule here.
`workflow.tdd_mode` is `true` in `.planning/config.json`, which makes every eligible task
`type: tdd` and checks the RED and GREEN gate commits. `.claude/guardrails/tdd-mode-check.js`
runs at session start and says so if the setting drifts back off; it is silent when the setting
is right and has its own tests:

```bash
node --test .claude/guardrails/tdd-mode-check.test.js
```

The setting and the check exist rather than only this paragraph, for the reason the rest of this
file keeps giving: a rule that lives in a document is one somebody has to notice being broken.
The only exceptions are the ones GSD already lists for `tdd="true"`: configuration-only files,
documentation, glue code wiring already-tested components, styling. Anything that changes
behaviour gets a failing test first; if a change seems to need an exception, say so and ask.

**GSD's completion step is the other place it disagrees with what this project
needs, and the brief an executor is handed names every mark, not the file.**
When a plan completes it leaves marks in four places: the roadmap's progress
row, the plan's own `- [ ]` line in the phase's plan list, the current plan
number in `STATE.md` in its frontmatter and again in its body, and the
requirement's `[D]` lines. A brief that says "update the roadmap" gets the one
mark the executor already knows about, and a brief copied to every plan in a
phase copies the omission to every plan. Phase 11's brief said "the ROADMAP
progress row", twenty-five executors did exactly that, and on 2026-09-20 the
list showed three ticks against a row reading 25 of 31; it was found because a
later brief happened to name one unticked line and that executor reported the
twenty-one beside it. GSD's own `roadmap update-plan-progress` writes the row
and the plan count and ticks no plan line, read in
`gsd-core/bin/lib/roadmap.cjs` the same day, so the tick is done by hand, and
`tests/the_planning_files_agree_with_themselves.rs` compares `STATE.md`'s plan
number with the roadmap's row and counts no ticks, so the two drifted for two
days with a green test on every commit. Widening that test to count ticked
lines against the row is the check this paragraph wants and does not have yet;
until it exists this is a rule that lives in a document, which the rest of this
file says is not enough.

GSD's vendored tooling under `.claude/` is gitignored and reinstallable with
`npx @opengsd/gsd-core@latest --claude --local --profile=full`. `.claude/guardrails/` and
`.planning/` are tracked on purpose.

Network-dependent code (IMAP, SMTP, POP3, Google, Microsoft Graph, CalDAV, iCal subscriptions) is
tested against parsing and error-mapping logic, not live servers. Keep the transport thin and the
parsing pure so the pure part is testable.

### Elegant code

Invoke the `elegant-code` skill whenever writing, reviewing, or refactoring. In this codebase that
means:

- Errors flow through `common::Error` and `common::Result`. Do not use `unwrap` or `expect` outside
  tests and `build.rs`. Map foreign errors at the boundary where they enter.
- Prefer typed values over stringly-typed ones. When a database column holds a fixed set of strings
  ("confirmed", "tentative", "cancelled"), model it as an enum and convert at the SQL boundary.
- Small, self-documenting functions. If a name needs a comment to explain what it does, rename it.
- Reach for `From` conversions rather than hand-written mapping functions between layer types.

### CI must stay green

Every commit builds, passes tests, and passes lint. Clippy is enforced with `-D warnings`, so a
warning is a build failure:

```bash
bash scripts/check.sh
```

**This paragraph said "the same four checks CI runs" until 2026-09-13, and CI has seven jobs.**
A full local run covers five of them: Rustfmt, Clippy, the `scripts/*.test.sh` suites, the Test
Suite, the release half of Build, and, since 2026-09-13, Security Audit. It does not run the debug
build, the setup executable, or the search handler's own fmt, clippy and tests, because that crate
is not a workspace member and every cargo command here walks past it.

The wrong count was not harmless. `cargo audit` was one of the two jobs nothing local ran, it went
red on 2026-09-10, and it sat unread for three days behind a sentence saying the local script
covered everything. A count in a document is a claim like any other, and this one had drifted in
the file whose whole argument is that claims need checks. The last block of `scripts/check.sh` now
prints what a run covered and what it did not, so the next person reads it from the run rather than
from here.

Use the script rather than running the commands by hand. Cargo shares build
fingerprints between `check`, `build`, `test`, and `clippy`, so a clippy run that
follows a build can be treated as fresh and report success without linting
anything. That has already put a clippy failure on `main` after a local run
reported clean. The script touches `src/lib.rs` first to force the work.

Better, run them on every commit, so the answer cannot be lost between getting
it and committing:

```bash
git config core.hooksPath .githooks
```

That has gone wrong twice now: the stale fingerprint above, and once when the
script's output was piped elsewhere so the shell read the pipe's exit status
rather than the script's, and the commit went through against an unformatted
tree. Never pipe `check.sh` into anything you then test the result of. With the
hook on, the commit itself runs them and a failure stops it.

**What runs depends on where you are and what you changed, and
`scripts/which-checks.sh` decides it.** On `main`, all four for anything that
touches code: every commit here lands on it. **Corrected 2026-09-10, found by
`05.1-05` while running the thing rather than reading about it.** This used to
say "all four, whatever changed", and the script has not agreed with that
sentence for some time: on `main`, a commit touching only `.md` and `.txt` files
answers `docs_only` and runs formatting, clippy and the document-reading targets,
with the reasoning written into the script at the branch that decides it. The
rule the sentence was reaching for is elsewhere in this file and is the true one:
a document change still runs the document-reading targets, because
`tests/house_style.rs` reads documents and its em-dash guard has caught two real
breaks in markdown. On a branch nobody builds, the slow half waits for the merge,
and what runs is scoped to the change. A commit touching only
documents runs formatting, clippy and the targets that read documents, which
are listed in `scripts/check.sh`'s documents-only branch with the reason for
each (three when this sentence was written on 2026-08-31, nine on 2026-09-14);
a commit touching code runs those plus the tests reaching the modules it changed,
plus the guards that read the whole tree. Measured 2026-08-31: a four-file
markdown commit went from about 330 seconds to 36.

**A guard living in `tests/` also runs on the commits that could break it, and
`guards/guards.toml` is what says which those are.** A unit test lives beside
the code it covers, so `--lib a::b::` reaches it. A guard that covers a `src/`
module from outside was reached only when its own file changed, which is every
commit except the ones that matter. Each record already carries a `file`, the
source its break is applied to, and some carry a `suite`, the target that goes
red, so `check.sh` reads that coupling rather than keeping a second list beside
it. A guard with no record is therefore a guard the gate cannot find: two had
none until 2026-09-02, both covering `src/presentation` from outside. If you
write an integration test that guards a source module, write it a record too, or
it runs on every commit except the ones that could break it.

**Read those two numbers as warm, and as a floor.** The same documents-only
path took 2m56s on 2026-09-02, because it followed two commits that had changed
test files and so paid for a clippy rebuild, and because the document-reading
list includes two targets that build a live window. The saving is real and the
shape holds every time; the figure does not, and quoting it to somebody planning
work quotes a measurement without its conditions. This project already asks that
of test counts under PERF-06, and a duration is the same kind of claim.

**The obvious version of that rule is wrong here.** "Nothing Rust changed, so
skip the tests" would be false, because `tests/house_style.rs` reads documents
and its em-dash guard has caught two real breaks in markdown. So a document
change still runs the document-reading targets, and that was proven by breaking
one on purpose and watching it redden before the rule was trusted.
Measured warm on 2026-08-30, the whole gate was 311 seconds, of which the test
suite was 239 and the release build 56, so the quick pair was 15 seconds and the
slow two were everything else. Those are that day's figures and the shape is
what to keep. What the gate costs now is on `docs/development/measurements.md`:
the full-gate row is a band harvested from the commit bodies, 275 to 654
seconds across the branches that recorded one on 2026-09-14, and the row for
`cargo test --all-targets` read 104 seconds the same day with every target
already built, which is the suite's term taken on its own rather than inside a
gate run, so read the two against each other with that difference in mind. On
a branch the slow two wait for the merge, where they run once rather than once
per commit. Whoever merges runs `scripts/check.sh all` first, and that is the
run they are paid for.

`which-checks.sh` answers `all` for anything it cannot place, including an
empty branch name and a detached `HEAD`. A check that cannot tell where it is
must not answer "safe".

The hook is `commit-msg`, not `pre-commit`, and that is not arbitrary. Only the
commit message can say that a commit is the red half of red/green, and at
`pre-commit` time the message does not exist yet. As a `pre-commit` hook this
gate could not tell a red commit from a broken one and refused both.

**Do not reach for `--no-verify`.** The advice used to be that it was for a work
in progress on a branch nobody builds. That was right about the case and wrong
about the tool: it skips formatting and clippy as well, which cost fifteen
seconds and catch real things, and it is a habit that does not stay on the
branch it was learned on. Both cases it existed for are now handled: a branch
defers the slow half, and a commit whose tests must fail says so in its message
and is measured against what it said.

**`cargo test` takes one `--lib`, and 55 plans have told their executor to
pass several.** `cargo test --lib a:: --lib b::` is refused by cargo. It has
been written into plans since phase 1, in `<verify><automated>` blocks and in
acceptance criteria, and it went unnoticed until 2026-09-07 because nobody had
ever run one. Each plan copied the shape from the plan before it.

Several module paths need several runs, joined so a failure stops the line:

```bash
cargo test --lib application::sending_later:: && cargo test --lib data::config::
```

What it cost was not the typo. It is that a plan's verification command is
written as evidence the plan can be checked, and for a year none of these could
be run at all. **A command in a plan is a claim about that plan, and an untested
one is the same kind of thing this file keeps warning about: a check nobody
reads.** Two executors found it independently, three plans apart, because the
finding could not reach a plan that was already written.

**Two more shapes of a check that cannot fail, both found in phase 8's plans on
2026-09-14.** A plan that claims a sentence is absent, or present at exactly N
sites, searches for it with a pattern that tolerates line breaks and
indentation, or joins the lines first. Prose wraps, and a single-line regex has
a hole exactly one line break wide: a phase 8 plan asserted "the four sites" of
a sentence by `grep -rn` over named files and made "the grep finds nothing" its
acceptance criterion, and one of the four carried the sentence wrapped across
two comment lines, so the grep could not see that site before the edit and could
not have proved its removal after. And a `<verify><automated>` command must be
one whose exit status carries the answer. A command ending in `| wc -l`,
`|| true`, `| cat`, or anything else that exits 0 whatever it found, cannot fail,
and a plan checker treats one as a blocker rather than as verification. The same
plan's verify ended in `| wc -l`, which prints a count and then succeeds. The
vendored checker refuses `|| true` on the right of an assignment and nothing
else in this family, which is why the rule is written here.

**The scripts that decide all this have their own suites, and the gate runs
them.** `scripts/*.test.sh` runs on every invocation of `check.sh`, in every
mode, before anything else, and in CI. For one day these suites existed and
nothing ran them, which is guardrail 4 exactly: a check nobody reads is worse
than no check, because it reads as covered.

**They cost 108 seconds, not the milliseconds this paragraph used to claim.**
Measured 2026-09-10 at `eda2719`, running every `scripts/*.test.sh` in turn, the
way `check.sh` does. That is paid on every commit in every mode, so it is the
floor under the cheapest possible run: a documents-only commit cannot come in
under it however narrow the rest of the scoping gets. Re-taken the same way on
2026-09-19 by 11-06.3: 43 seconds at `a7758f83`, before its two cases, and 47
at `d5c3483e` after them, `which-checks.test.sh` going from 14 to 17 for two
cases that each start a `bash` and build a repository. Less than half the
figure above on the same machine, with nothing in the suites removed between
the two days; what moved is not diagnosed, so quote neither without its date.

Where the figure came from is worth knowing, because the same mistake is
available to the next person. These suites did once cost milliseconds. They grew
a case at a time, each addition genuinely cheap, and nothing re-measured the
total: `which-checks.test.sh` alone went from 28.7s to 34.4s in one sitting on
2026-09-09 when seven cases were added to it. A cost that grows only in
increments nobody prices is the shape this project keeps meeting, and this
sentence sat wrong for long enough to be quoted in scheduling decisions.

Never silence a lint with `#[allow(...)]` to get a commit through. Fix the code, or if the lint is
genuinely wrong for this case, add the allow with a comment saying why.

### Tests that would notice

A green suite says the code does what the tests say. It does not say the tests
would notice if it stopped, and those are different claims. Red/green is what
keeps them together, and it started at `18a02454` on 2026-07-26, the commit
that added this file. How much of the history predates that commit is
computed on every commit by
`test_the_share_of_history_before_red_green_is_computed_and_printed` in
`tests/every_number_carries_its_command_and_its_date.rs`, which prints it
with the day: 8.9% on 2026-09-14. This sentence used to give the share as two
absolutes, the commit's position and the count on the day it was written,
which was 53% on 2026-07-29 and stayed word for word true while the share
fell. Most tests here were still written after the code they cover, because
red/green began after most of the early work, and they describe it rather
than specify it; that is a claim about tests, which no commit count settles
either way. Three tests written in one session to catch a named bug passed
against that bug.

```bash
scripts/mutants.sh src/service        # one directory, slow
scripts/mutants.sh --since v0.19.0    # only what changed, minutes
```

Name a real commit or tag to compare against. Every commit here lands on `main`,
so `--since main` compares `main` with itself, finds nothing, and now says so
instead of passing. That was written down here as the way to check a change for
275 commits and could never have tested a line.

Mutation testing alters the code in small ways and runs the suite. Anything
nothing catches is either untested behaviour or dead code. A whole-tree run
costs a rate times a count, and the count is a row on
`docs/development/measurements.md`: `cargo mutants --list` answered 12,335
mutants over 247 files on 2026-09-14. The rate is not written anywhere yet on
purpose; it is measured on one shard, under the suite shape the run will use,
before any whole run is scheduled, and the product goes on that page beside
the count. This sentence said a whole-tree run is "about two days" from
2026-07-29 until 2026-09-14, with no date, no machine, no thread setting and
no record of which count or rate it was built on, so nothing in it could be
re-taken. So it is used scoped, and the pull request check runs it on the diff
only. Before trusting a new regression test, take the fix out and watch the
test fail; a test that has never been red proves nothing.

Taking it red once is not enough either, because the code around it goes on
changing and nothing re-asks. One commit added an arm to a decision in the
contacts sync and two guard tests started reaching the new arm instead of the
one they were about; both arms do nothing, so every count and every sentence
they assert came out the same either way. Two guards became one, both kept
their names, nothing was red at any point, and it was found by hand three
commits later.

```bash
scripts/guards.sh              # every recorded guard, one build each
scripts/guards.sh deletion     # only the guards whose names match
```

`guards/guards.toml` holds, for each guard, the exact edit that should break it
and the tests that should go red when it does. The script applies each one, runs
the whole library, and requires the tests that failed to be exactly the ones
named. Both directions, because a run that only ever asked whether the named
tests went red could not see a record falling behind: one named eight tests for
a break that reddens seventeen, and said so for three commits. Run it after a
change that touches code a guard is about, with nothing else building at the
same time. When a recorded break no longer matches the file the run fails and
says so: that is the moment to measure that guard by hand again, not to edit the
record until it applies. Add an entry the same way, by taking the break by hand
first and writing down all of what really went red.

**A record is a measurement with a date, and it perishes.** Phase 1 found four
stale records, one that had fallen behind within the same day and one within the
same session. The cause is always the same and never announces itself: a later
change adds tests that reach a rule an existing record is about, so the record
now names too few, and nothing fails. `01-02`'s writer record named 5 tests when
it was written on 2026-08-30, 17 a day later, 21 the day after, and 31 by the
end of the phase. So: **any change that adds tests near a rule re-measures that rule's
record.** The filter you would naturally pick for your own subject is not
enough. One record turned out to redden nine tests, two of them in a module
nobody working on that feature would have filtered for.

That sentence sat here unenforced and did not happen. A run of 2026-09-01 found
21 records naming too few, one of them written days earlier. So every record now
carries the tree it was last checked against: for the files its red list names
and for the file it breaks, how many test functions each held.
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` compares
those with the tree on every commit, and when a file gains or loses a test it
names the records and prints the command:

```bash
scripts/guards.sh --remeasure "a name" "another name"
```

That measures those records and writes the counts down again for each one whose
red list turns out still to be right. A record that comes out short is corrected
by hand first; the run after that records it.

The check is a net under the common case and not a replacement for the run.
Three things it cannot see. A test added to a file no record names can still
redden a record, and nothing about the counts predicts that. A count is a size
rather than a set, so a test deleted and another added leaves the number where it
was. And what it costs when it fires is not flat. Measured 2026-09-06 against
`main` at `485030f`, when the file held 617 records: 477 of them name one file,
but a test added to `src/application/contacts_sync.rs` flags 77, which at a build
and a run each is hours. That case is what the paragraph below about the critical
path is for, and the answer is to run the command it prints in the background
rather than to lower the check.

**Count records, not mentions, and do not use a ratio to convert between them.**
This file used to say a grep for a file name overcounts by roughly two, and ten
plans were misled by quoting a grep. The ratio is not two and it is not stable: on
the same tree, on 2026-09-06, `grep -c contacts_sync guards/guards.toml` answered
363 against the 77 records that really named it in a `tests_last_seen` block, which is nearly five
to one, because a file appears in a record's `file`, its `before`, its `after`,
its `red` list and its prose comment as well. Read the file with the format's
own parser and count the records whose `tests_last_seen` names the file:

```bash
python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(sum(1 for r in g if any(e['file']=='src/application/contacts_sync.rs' for e in r.get('tests_last_seen',[]))))"
```

Print `len(g)` instead of the sum for the number of records. The rows on
`docs/development/measurements.md` are taken with these two commands and
nowhere else.

**Until 2026-09-14 this paragraph prescribed an awk in place of the parser,
and the awk miscounted.** It read the file a line at a time, setting a flag on
a line beginning `tests_last_seen` and clearing it on a bare `]`. TOML also
allows the whole block on one line, `tests_last_seen = [{ file = "...", tests
= N }]`, and the file holds four records spelled that way. On such a record the
awk consumes the line before it looks for the file, so the record is never
counted, and the flag it set stays set until a later bare `]`. Measured
2026-09-14 at `3accd6e1`, the awk against the parser, per file: it agrees for
`src/presentation/wx_app.rs` (50 and 50), `tests/house_style.rs` (21 and 21)
and `src/application/contacts_sync.rs` (77 and 77), and for every file no
inline-table record names; it undercounts by at most one for each file an
inline record names, which that day was one file,
`src/presentation/wx_send_later.rs`, 0 against 1. The miss is small today and
the shape is the point: a line reader for a structured file is right for the
spelling the file happens to use and silently wrong for the others the format
allows, and it hands back a plausible number rather than an error. It arrived
here as the correction to a known miscount, which is the part worth
remembering.

**A renamed test is worse than a stale record, and it is the second half of
that middle limit.** A count cannot see a rename: 71 tests before, 71 after. And
a record naming a test that no longer exists is not stale, it is
*unmeasurable*, because `scripts/guards.py` refuses it before reporting anything
about the break, so the message reads as a broken tool rather than as a finding.
One record sat that way from 2026-08-16 until 2026-09-02, surviving a six-hour
sweep that could not report it, and when it was finally measured the break
reddened 20 tests rather than the 10 written down.

The commit that caused it shows how little warning there is: it renamed two
tests, corrected those two names in the record above, and left them in the
record below. Its message says it re-measured both records it touched, and it
had, by the only reading anyone applies. It counted the records whose *code* it
changed and missed the one it broke by renaming a test that record merely
*names*.

So `test_every_test_a_guard_record_names_is_a_test_that_exists` asks the other
direction, in milliseconds, on every commit. **If you rename a test, that check
tells you which records name it, and the record then wants re-measuring rather
than editing**, because a rename can change what the break reddens.

**A record is coupled to the tree a second way, by the text it anchors on, and a
plan lists the records it will touch by both readings.** The first reading is
the one plans already make: records naming a changed file in `tests_last_seen`
or in `file`. The second is records whose `before` or `after` text lies inside
any file the plan edits. For each of those the plan says whether the edit
duplicates the anchor, removes it, or moves it, and names the record as one the
executor rewrites and re-measures in the same commit. A task that extracts,
inlines, moves or re-indents a block moves every anchor inside that block, even
where no line in it changes meaning. Both halves were paid for on 2026-09-17,
by `test_every_guard_record_still_names_one_place_in_the_tree` refusing a green
commit each time. Plan 10-02.2 added `fetch-depth: 0` to a second checkout in
`ci.yml`, and the record for the first checkout, anchored on that bare line,
named two places from then on; the plan's record list, taken by
`tests_last_seen` and `file` alone, had not seen it. Plan 10-04 extracted the
inline Feedback block into `read_the_feedback_page`, re-indented by four spaces,
and a record anchored on a line inside the block stopped naming one place the
moment the extraction landed; the checker had grepped the register for the one
line the task rewrote and not for the block another task moved. So when you
write a record, anchor a text break on the longest unique context that still
names one place, the comment line above plus the line, because a short generic
anchor, a YAML key or a one-word arm, is right on the day it is written and
wrong the first time a sibling is added.

**Guard re-measurement is not on the critical path.** This used to say "run
`scripts/guards.sh` unfiltered before you finish", and that instruction put the
whole library on a branch once per record. Plan 02-01 ran fourteen records, 49
minutes of a 189-minute plan and the largest single cost in it. It also
contradicted the rule that until a branch is about to be merged, only the tests
reaching what changed need to run: that rule was written about commits, and
nobody extended it to the thing that runs the full suite most often.

The reason it can come off the critical path is worth stating, because it is
what makes the rest safe. **A stale record does not break anything.** The build
is green and the tests pass. What it means is that a guard is weaker than its
record claims, and nobody has been told. That has to be caught reliably. It does
not have to be caught before a merge.

So: the executor does not run guards, the merge does not run guards, and neither
does anything after a merge. **One sweep, once every phase is complete.** That is
a decision of 2026-09-03 and not a measurement. Nothing about a sweep gets
cheaper by happening sooner, and a sweep per merge, `scripts/guards.sh` scoped
to the branch, spent about 90 minutes of every branch to find, across the last
220 records measured by then, one problem.

What holds the line in between is not the sweep, and saying which checks do the
work matters more than the rule itself. On every commit,
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` names the
records a changed test count could have made stale and prints the command that
re-measures them, and `test_every_test_a_guard_record_names_is_a_test_that_exists`
catches a rename before it turns a record unmeasurable. Both cost milliseconds.
**Running the scoped remedy when a commit prints it is not optional**, and it is
the reason the deferral is affordable: phase 2.1 ran that remedy throughout, and
its 220-record sweep afterwards found one problem where an earlier 208-record
sweep had found 23.

```bash
scripts/guards.sh --touched-by <a commit to compare against>   # scoped, still there
scripts/guards.sh                                              # the whole sweep
```

**What the decision costs, said plainly.** The sweep will run against a tree many
phases past the changes it is judging, so a record found stale will have gone
stale for a reason somebody has to reconstruct rather than remember, and
everything it finds will arrive while the milestone is trying to close. That is
accepted because a stale record breaks nothing and the two per-commit checks
cover the common case. It is carried as a success criterion of phase 8 rather
than left here as a sentence, for the reason this file keeps giving: a rule that
lives in a document is one somebody has to notice being broken.

**Know what it costs before starting it, and take the count again rather than
reading it here.** This paragraph quoted 565 records, and two others quoted 548
and 536, none of them dated, in a file whose whole argument is that a measurement
without a date perishes. Phase 6 and phase 8 research both went to count and found
617, on 2026-09-06 against `main` at `485030f`.

**The sweep's cost is a rate times a count, and both terms live on
`docs/development/measurements.md`, not here.** The count is the record row,
taken with the parser above. The rate is the row read off the `timed:` line
`scripts/guards.py` prints after every record since 2026-09-14, rebuild and
run as two figures, because the two move for different reasons and have moved
in opposite directions. Their product is a third row, and that row is the
only place the total is written. Until 2026-09-14 the tree gave four figures
for this one job, none of them dated and none of them the same: "roughly 15
hours" in the roadmap, "about 20 hours" in this paragraph, "eighteen hours"
in `tests/house_style.rs` and "an hour or two" in `guards/guards.toml` and
`scripts/guards.sh`. Each was right on its day at that day's count and rate.
The 20 hours here was 617 records at about 119 seconds on 2026-09-06, and it
was already wrong three days later.

What moved it: on 2026-09-09 the run term halved, because the library run
stopped rebuilding the database schema once per test that opens a cache.
Measured that day at 8 threads, the two commits interleaved so the machine is
in the same state for both: 120.3s and 120.2s at `02ddbcb`, 64.8s and 63.8s
with the schema template. The rebuild term then rose on its own, from 29 s on
2026-09-10 to 44 s on 2026-09-14, which the rate row records, and which is why
a total read off either term alone is wrong. This paragraph used to say the
rebuild term was never measured beside the run term; it has been since
2026-09-10, and the runner has printed both apart since 2026-09-14. The
instruction is unchanged: take the count today and measure the rate today.

**A plain sweep has no resume.** It writes nothing back to `guards.toml`; its only
artefact is the flushed log. A job of that size that cannot be stopped and picked
up is a scheduling question before it is a technical one.

Scoped to a single branch it was 63 records of the 536 that
existed on 2026-08-31, about 90 minutes, for plan 02-01 that day, counting the
records in `guards/guards.toml` that named a test in a module the plan
changed. Narrowing that to modules which
actually gained a test only reached 52, because one large shared file gained
tests and many records name a test in it, so there is no clever selection that
makes a scoped run quick either.

That setting is `WIXEN_TEST_THREADS` and **it defaults to 8, raised from 4 on
2026-09-09 because the turning point moved and nothing re-asked.** Re-measured at
`10effea` on 24 logical cores over the 6,720-test library run on its own: 1 thread
375s, 2 threads 226s, 4 threads 161s, 8 threads 140s, 16 threads 234s. The suite
is contended rather than compute-bound, which is unchanged; where the contention
bites is not.

The figures this paragraph used to give were taken on 2026-08-30 over 5,837
tests, `cargo test --lib` with `WIXEN_TEST_THREADS` set to each value in turn:
2 threads 131s, 4 threads 88s, 8 threads 106s, 16 threads 164s, one per core
196s. They put the best at four. **The suite grew past that and the default
went on costing about 13% of every guard run, for an unknown number of weeks,
with nothing failing.** That is this file's own rule about dated measurements
arriving as a bill: read the numbers above as a snapshot of one machine and one
suite size, and re-take the curve rather than trusting it the moment a run feels
slower than it says.

Two conditions that held when the old curve was taken are now gone and neither
changed the shape: `target/debug` had reached 786 GB, and a virus scanner was
reading it.

**The curve was taken a third time later the same day, after test caches stopped
building the database schema, and this time the shape moved.** Whole library, one
pass, same machine: 4 threads 93.9s, 8 threads 50.6s, 16 threads 51.8s. Against
the same three at `02ddbcb`, taken the same way: 135.3s, 133.0s and 183.4s. Four
and eight used to be level and sixteen cost 38% over eight; now four costs 86%
over eight and sixteen costs 2%. Eight is still the right default. What changed
is where the contention was: the writes that punished sixteen were the schema
being built once per test, and they are gone. Read each of those as one pass. Two
passes of the same pair on the same day differed by about 10%, which is wider
than the gap between eight and sixteen.

**One setting is not enough, and that was measured too.** A *scoped* run is
contended harder than a whole-library one, because there is less work to spread
against the same contention. Measured the same day, 2026-09-09, against
`application::tasks_sync::`, 111 tests: 14s at the harness default, 4s at four
threads, 6s at eight. So `scripts/check.sh` pins the scoped `--lib` run to four
while the guard runs use eight.

**Neither carries to the `--all-targets` run**, which `scripts/check.sh` still
leaves at the default: measured 2026-08-31, the test term fell from 197s to
111s and the whole gate did not move, 335s against 353s. Those are that day's
figures; the same run read 104 seconds on 2026-09-14 with every target already
built, the row on `docs/development/measurements.md`, which is the one to
quote. Reading an isolated figure as though it applied everywhere was the
original mistake, and timing the thing you are about to change rather than
assuming is what has now caught it twice.

Running the records in parallel across git worktrees was measured on
2026-08-31 with `cargo test --lib` and rejected: two concurrent suites took
131s each against 88s alone, so the contention is on something shared rather
than on the processor and crosses process boundaries. Five workers would have
bought about 1.8x for 43GB of disk and a five-minute build each, on that day's
suite; nothing has re-taken it since the schema change above.

**The other environment setting is `WIXEN_NO_AUDIO`, and it says the machine
cannot play sound.** CI sets it. Set it anywhere else a sound device opens and
then does not work, which is not the same as having none and is the case
nothing can detect: GitHub's Windows runners ship no audio driver at all
(`actions/runner-images#6983`), `rodio` 0.22.2 opens something there regardless,
and the first write faults.

**It is not a way to skip the sound tests, and reading it as one is the mistake
to avoid.** Every test that plays still runs and still asserts; the sound goes
to a mixer with nothing listening instead of to a card, and
`test_a_player_with_no_device_behind_it_still_plays_and_still_suppresses` holds
it to that. What stops being exercised is `rodio`'s own device handling, which
no test here covered anyway.

Two things this cost before it existed, both worth remembering because they are
the same shape as the CRLF break above. **A crash is not a failing test.** The
run, on 2026-09-03, died partway and about 3,500 tests after it never ran, while the summary
named two, so the number that mattered was the one nothing reported. And
**fixing the tests a grep finds is not fixing the class**: four in
`feedback.rs` were corrected, the next run crashed on one in
`sound_scheme_import` whose name ends "and its files really play", and
`Accessibility::new` builds a player at 50 sites in `src/` and 12 files under
`tests/`. The flag went on the player for that reason.

**It is a candidate set and it says so when it runs.** A test added in a module a
record has never named can still redden it, and no reading of the record predicts
that. Only the whole sweep does.

**A census that asserts a floor is itself a guard, and it weakens others.** A
constant saying "at least 8 of these exist" stops being load-bearing the moment
there are 9: with a spare above the floor, removing one gated write no longer
trips the guard that counts them. That happened here, silently, and
`scripts/guards.sh` was the only thing that saw it. When you add a member to
anything a census counts, re-measure every record that reads that census.

**A guard whose trigger is "a document mentions X" is disarmed by the workaround
it recommends.** `test_no_status_page_names_a_version_the_code_does_not_ship`
compares versions named in `README.md` and `docs/IMPLEMENTATION_STATUS.md`
against the shipped one. Neither file names a version, so it iterates over
nothing and passes unconditionally, and its own comment advises that a page
wanting to stay out of the way should point at the changelog rather than name a
number. Somebody took the advice and the check stopped checking. The version
rule has now lapsed five times behind it. A guard that reads documents needs a
companion proving the reading can see a violation when one exists; several
guards here already carry one, and that is why.

`application::filters`, `due`, `tagging` and `sign_off` are clean as of
2026-08-01: 157 mutants, 141 caught, 16 that would not compile, none missed. It
took three passes to get there, and what the first two found is the pattern
worth remembering. The tests covered the paths somebody would think to write a
test for and left whole families of behaviour untouched: four of the fields a
filter rule can name, six of the eleven ways it can match, five of the actions
it can carry out. Each family had one member with a test and the rest with
none. When a function switches on a string, test every arm both ways, or expect
mutation testing to find the ones you skipped.

Read a partial run as partial. `mutants.out` is written as it goes, and reading
it mid-run once produced a commit message quoting seventeen of eighteen caught
when the real figure was thirty-seven of fifty-one. Wait for the process to
exit.

The script now refuses a partial run rather than summarising one, and it refuses
three other things that used to read as results. A mutant recorded as unviable
means one of two things, and only one of them was tested: the compiler looked at
it and rejected it, or the compiler never started and nothing looked at it at
all. The whole-tree run of 2026-08-05 recorded 595 that way and 473 of them had
never reached a compiler, so a third of that run was untested and its summary
said so nowhere. A run whose build failed before anything was changed used to
print that every mutant was caught and exit clean.

The third is a run where the suite was never once run against a mutant. That
happens two ways and the report says which: every mutant was rejected by the
compiler, or there were no mutants at all because nothing in those lines can be
changed. Either way the run learned nothing, so it fails rather than printing a
headline nobody can tell from a clean result. This reverses an earlier decision
that called having nothing to mutate an honest way to test nothing: it is
honest, and it is still not a result. A change that touches only comments or
documentation inside `src` now fails this gate on a pull request. The answer to
that is a sentence on the pull request saying so, not a lower check.

A compiler that never starts is this machine failing to start a process, and it
says nothing about the mutant. It comes and goes: six mutants that never built
on 2026-08-11 all built when the same twelve were asked again twenty minutes
later, and five of the six were caught. So the answer to that refusal is to run
those files again, not to change the code and not to lower the check. What
causes it is not diagnosed.

```bash
cargo llvm-cov --lib --summary-only
```

Coverage is the cheap wide sweep and answers a weaker question: what never runs
at all. Low coverage in `service/protocols`, `service/oauth` and the provider
clients is the network transport that has never been run against a live account,
which is tracked as work rather than fixable by writing more tests.

### Premises a plan checks before execution

A plan is a set of claims about the tree, and the executor pays for every one
nobody checked. GSD's planner and checker are vendored under `.claude/` and
rewritten on reinstall, so the checks that came out of running Wixen Mail's
plans live here rather than in the skill files. Each rule below opens with what
the plan does and then says which plan taught it. They are about premises, the
things a plan takes as given before its first task, and every one was found by
an executor meeting a tree that differed from the one the plan described.
Added 2026-09-20 from the skill review; three related rules sit where they act:
the absence check that cannot fail under "CI must stay green", the anchor-text
reading of the guard register under "Tests that would notice", and the
completion marks under "Test-driven development".

**Try the read before accepting "cannot be read from here".** When a plan says
a value cannot be read from here, cannot be known until execution, or only
somebody else can confirm it, the checker treats that as an absence claim like
any other and attempts the read with a targeted read-only command on the machine
it is running on, reporting the result either way: the value and what it
settles, or the command that failed and why. Plan 09-02 said on 2026-09-16 that
the tester's stored language "cannot be read from here". The checker was running
on the tester's own machine, the settings file was two commands away, and the
value, a bare `en`, confirmed the plan's cause. A strong confirmation had been
turned into a hedge because nobody tried.

**Read both places an "or" offers, or say which one the requirement is about.**
When a verify or acceptance line names two places to read one fact, the plan
either names the one the requirement is about and says why the other is not it,
or requires both readings and a sentence on any difference. An "or" between
artefacts is a claim that they agree, and it needs a command behind it like any
other premise; when a value has two carriers, the one that disagrees is usually
the finding. Plan 10-02.2's brief said on 2026-09-17 to read the Windows file
version off "the built setup exe or the exe inside target/release". Both were
read. The setup file carried `1.0.0.14114`; the exe inside it carried
`1.0.0.0`, because `winresource` in `build.rs` stamps that resource from the
crate version and nothing hands it the build. Reading the exe alone would have
reported the feature absent, and reading the setup file alone would have missed
that the installed program's own Properties never show the counter, which
`docs/changelog.md` then had to say.

**Add the files this project writes by rule to every plan's `files_modified`
before deciding whether two plans share a wave.** Those files are
`docs/changelog.md` for any user-visible change, `guards/guards.toml` for any
new or re-measured guard, `.planning/WINDOWS.md` where a plan opens or closes a
ledger entry, and `docs/development/measurements.md` where it takes a figure.
Only after they are added are two plans' lists compared. A brief's claim that
two plans touch different subjects says nothing about the files, and a wave
check that starts from the subjects passes two plans that collide on the
commit. Phase 10's brief for two plans inserted after 10-02, one a list's sort
order and one the build counter, said on 2026-09-17 that they shared no file and
could share a wave; both write the changelog and both write the records file,
so the roadmap's own definition of a wave put them in two. It was written by
somebody who knew both rules. Phase 11's README applies this by hand, "One per
wave" with the reason, which is how the rule was seen to hold across phases.

**When another phase is executing against the same tree, derive the overlap
from its plans rather than from the brief.** Research for a phase run beside
another lists the files the other phase's own plans name, ranked by how often
they name them, intersects that with the files this phase will touch, and
reports the intersection as a named section that separates what the brief
anticipated from what the intersection found. Two phases collide where their
shared infrastructure lives, not where their subjects meet, and the strongest
predictor of a collision is a mechanism cheap enough that both chose it. On
2026-09-12 a brief named the collision surface between two phases as a version
module, the About dialog and two accessor functions, and it was careful and
incomplete: one command over the sibling's plan directory found that both
phases edit the settings-reachability test module in `src/data/config.rs`,
because both had found that a new field there "fails on arrival, which is the
RED half for free", and both add a control to the same settings dialog. Neither
side's documents mentioned the other. The parallel-planning rule in memory,
that a planner working beside an executor writes to the scratchpad, is the
adjacent rule and does not cover this step.

**Check a packing formula against the property the field exists to keep, at the
boundaries, not only against its bound.** When a brief or research note hands
the plan a formula that packs several values into one bounded field, the plan
states the property the field is for, usually an order, a uniqueness or a round
trip, and checks the formula at the largest value of each term and at each
transition between terms, such as the last step of one stage against the first
step of the next. A formula that only fits under the maximum is not yet
checked, and the boundary cases go into the test as rows rather than being
trusted as arithmetic a second time. The brief for 10-02.2 offered
`stage * 10000 + step * 1000 + counter` for the 16-bit fourth field of the
Windows file version on 2026-09-17 and said it held counter to 999 and step to
25 under 65535. The bound claim was wrong, stage 3 with step 25 and counter 999
is 65999, and the worse fault was quiet: under that formula `rc.11` at 41000
sits above the release at 40000, and ordering builds is the only reason the
field is stamped. The weights in `src/common/version.rs` are derived from the
order instead, with the caps of 12 and 999 that the Versioning section gives
and a largest value of 64999, and the transitions are rows in its test.

**A criterion that greps for a call inside a named function is checked against
what that function can reach.** The checker reads the function's parameters and
captured environment and asks whether the callee is reachable from them. A
function on a worker thread that talks to the interface through a sender cannot
call the interface's layers directly, so a grep for that call inside the worker
cannot be satisfied; the plan then names the update the worker sends and the
arm where the call really lands, and writes the criterion against that site.
Plan 10-04 moved the new-mail signal into `spawn_mail_sync` on 2026-09-17 and
made "`grep -c 'a11y.signal(FeedbackEvent::NewMail'` is 1, inside
`spawn_mail_sync`" its acceptance criterion. The worker holds state, a sender
and a runtime, and the accessibility layer is only reachable on the interface
thread through the update handler, so the executor had to decide where the
signal lands, in the arm the worker's end-of-check update reaches, and record a
deviation for a criterion that could not have been met as written.

**Before reporting that a plan leaves a check unrun, read its verify command
and acceptance criteria, not only its prose.** Three questions, answered apart:
does the plan name the check, does the plan run it, and does the recommended
action violate it. Only the second is a gate failure; the first is a note on
the plan's writing, and the third alone is a design error. On 2026-09-08 a
review claim said a plan set a trap by recommending a menu placement that would
redden a whole-tree guard the plan never named. Both halves were true. The claim
still did not hold, because the plan's `<verify>` command and its first
acceptance criterion each ran the suite the guard lives in, so the guard would
have fired inside the plan's own verification rather than after it. Unnamed and
unrun had been read as one finding.

**A ledger entry that hands work to a named later plan is not filed until that
plan's own text mentions it.** The same commit adds one line to the plan's
`<premise_corrections>` or its file list if the plan exists on disk, or to the
phase README's row for it if it does not. `.planning/WINDOWS.md` is read by
whoever reads the ledger; a plan's executor reads the plan, and an assignment
written into only the first depends on the executor happening to read both.
Plan 08-01's summary ledgered a fifth stale figure in `scripts/guards.py` "for
08-06 to point at the page" on 2026-09-14, and 08-06's plan, written the same
day from the research document and the phase README, listed four figures and
never named the file. The executor found it only because the brief said to read
all five earlier summaries, and fixed it as a deviation on a file the plan did
not list.

**A plan that prescribes an instrument to explain what a screen reader hears
names the channel that screen reader reads for the control, and puts the logger
there.** On Windows that is UI Automation or MSAA and win events, and where
both carry the control the plan logs both and says which one the reporter's
screen reader uses. A quiet capture on the other channel is not evidence that
the producer is clean, and a plan branch that reads it that way is a premise to
correct before execution. The case, 09-06 and #33 on 2026-09-16, is in the
Accessibility section beside the two channels it is about.

### Done means it runs

Compiling and green tests are not done. A feature is done when a non-test path reaches it and it is
exercised end to end in the running application. A `dead_code` warning on a UI field is evidence
that a panel was built but never wired, not noise to be silenced.

Run `dead-code-hunter` after finishing a feature. If something cannot be finished because it depends
on hardware, an external service contract, or missing infrastructure, say so and gate it. Never
present a stub as complete.

### Accessibility

Target WCAG 2.2 Level AA, applied to a Windows desktop application. This is the product's reason to
exist, so accessibility work is not a review-time cleanup pass, it is part of building the feature.

Automated checks catch roughly half of accessibility defects. That is a figure about defects, and
it is not the share of criteria a scan can judge: the scans here can produce a finding against
three of the fifty-five WCAG 2.2 Level A and AA success criteria, 1.3.1, 2.1.1 and 4.1.2, and the
MSAA walk judges the Name part of 4.1.2 alone. `docs/wcag-coverage.md` has the row for every
criterion. Four copies of this sentence in the tree had turned the defect figure into a criteria
figure, corrected 2026-09-14; the first sentence here was left as it was, and the qualification is
what was missing. They do not replace testing with real assistive technology. Structure present
is not experience good.

**Windows has two accessibility channels and this project needs both right.** UI Automation is what
Narrator reads. MSAA, through `IAccessible`, is what NVDA reads for native controls, and it is the
only place `set_accessible_name` writes: for an edit box or a button, Windows supplies its own UI
Automation provider that shadows the MSAA object underneath. So a UI Automation scan reports the
system's name for those controls and never the one the code set. The accessibility workflow runs
Axe.Windows over UI Automation and `scripts/msaa-names.ps1` over MSAA, per window, and a name that
fails on either channel is a name somebody does not hear.

**The same two channels decide where a diagnostic instrument goes, and a plan
that names the instrument names the channel.** When a plan prescribes a logger
to explain what a screen reader does with a control, it says which channel that
screen reader reads for that control and puts the logger there; where both
channels carry the control, it logs both and says which one the reporter's
screen reader uses. A capture on the other channel that shows nothing wrong is
not evidence that the producer is clean, and a plan branch that reads it that
way is a premise to correct before execution. Measured on 2026-09-16 against
#33, a Settings tab NVDA spoke twice: plan 09-06 prescribed a UI Automation
event logger with three subscriptions, and run as written it showed exactly one
`ElementSelected` per arrow key, which the plan's own third branch would have
read as "the second reading is NVDA's own, ask for its debug log" and changed no
code. NVDA reads a native `SysTabControl32` through MSAA and win events, not
through UI Automation. A `SetWinEventHook` on the same process, added in the
same run, showed `EVENT_OBJECT_FOCUS` raised twice on the same tab one
millisecond apart, and moving the selection through `TCM_SETCURSEL` raised it
once, which named the fix (`src/presentation/wx_settings.rs`). The instrument
was right for Narrator and blind for the screen reader that reported the bug.
`scripts/uia-events.ps1` logs both channels since then, so the question a plan
has to answer is not which script to run but which channel's lines to read.

A control with a visible label beside it gets that label as its MSAA name even when nothing set one,
because Windows falls back to the nearest static text. That is a real name and it is really spoken,
so a clean run does not mean every name came from this code. Two further rules on the same
channels, verifying a name at the handle that takes keyboard focus on both of them and treating
mnemonic letters as one shared set per dialog, live in the Accessibility section of
`~/.claude/CLAUDE.md`, because each was learned here the way `set_name()` was and neither is
particular to this project.

- **Blind, screen readers.** Every control exposes a correct UI Automation Name, Role, Value, and
  State. Focus is managed and never lost when a panel or dialog changes. Announce dynamic changes
  through `presentation::accessibility::screen_reader` rather than relying on the user to discover
  them. Verify with NVDA, and spot-check with Narrator and JAWS.
- **Low vision.** Contrast at least 4.5:1 for text and 3:1 for UI components and meaningful graphics,
  in both light and dark themes. Never convey information by color alone; pair every color cue with
  text or an icon. Honor the system font size, respect Windows high contrast themes, and keep a
  clearly visible focus indicator.
- **Physical and motor.** Everything reachable and operable by keyboard alone, with no mouse-only or
  drag-only interaction. Accelerators follow standard Windows conventions and are documented in
  `docs/KEYBOARD_SHORTCUTS.md`, which is updated in the same commit as the shortcut. No timing traps.
- **Learning and cognitive.** Plain language in labels, messages, and errors. Predictable, consistent
  navigation across the mail, calendar, contacts, tasks, notes, and reminders modules. Errors say
  what happened, why, and what to do next. Do not make the user re-enter information they already
  gave (3.3.7), and do not require a memory or transcription test to authenticate (3.3.8).
- **Hearing.** Every audio cue has a visible equivalent. Never signal something by sound alone.
  Mail carries audio and video as attachments and embedded media: surface any captions or
  transcript the sender provided, and say plainly when none exists rather than presenting the
  media as though it were accessible.
- **Vestibular and photosensitivity.** Honor the system reduced-motion setting. Nothing flashes more
  than three times per second.

The email preview renders untrusted HTML in a WebView. Accessibility and security both apply there:
sanitize with `ammonia` first, and keep the rendered document's heading structure and link text
intact so the screen reader can navigate it.

Use the accessibility specialist agents when they fit: `Desktop Accessibility Specialist` for control
patterns and UIA exposure, `Desktop A11y Testing Coach` for NVDA and Accessibility Insights
workflows, `cognitive-accessibility` for language and flow, `contrast-master` for theme colors.

### Documentation and writing

- User-facing docs (`README.md`, everything under `docs/`, release notes): invoke `writing-craft`.
  Plain language, semantic structure, worked examples. No em-dashes; use commas, colons, or separate
  sentences. Avoid AI-slop vocabulary such as delve, robust, seamless, leverage, comprehensive,
  empower.
- Commits, PR descriptions, issue comments, code review: invoke `writing-style`. Direct and brief,
  why over what.
- User-visible changes get a `docs/changelog.md` entry under `[Unreleased]` in the same commit.

### Finish what you start, and say plainly what works

Do not stop partway through a job to describe what has been done so far. Do not
pause after each item in a task list to summarise. Stop early only for a real
blocker: a decision that is genuinely Pratik's and changes what gets built, or
something broken upstream. "This is a coherent stopping point" is not a blocker.

Shipping a partly wired feature and describing the gap in the report is the same
mistake in disguise. If a change leaves five of six paths not working, the job is
not done. An alpha build exists to be used, so anything in it that does not work
makes the alpha pointless.

Write back in plain language. No jargon, no framing that makes partly-finished
work sound finished. Lead with whether it works, not with what was built.

**If you expect bug reports from something, that belongs in the product.** Mark
it experimental where the person using it will see it, say why, and say what
could go wrong. A warning that only exists in a chat message is a warning nobody
gets. `application::allowed` and `presentation::first_run` are how this is done
here: everything that writes says it is experimental in the settings screen, in
the first-run screen and at the end of `--help`, because none of it has run
against a real account.

### Working style

Report outcomes faithfully. If tests fail, say so and show the output. If a step was skipped or
gated, say that. If something is done and verified, say it plainly. A feature that compiles but
was never reached is not implemented, and reporting it as implemented is the failure mode this
whole file exists to prevent.

Do not silently absorb upstream failures. Where this codebase papers over a broken or
inaccessible dependency, a sender's missing alt text, or a provider's malformed data, note it so
the gap stays visible.

### Project rules

- **No AI attribution anywhere.** No `Co-Authored-By` lines naming an AI, no AI or assistant names in
  commit messages, branch names, code comments, or documentation. This applies to every commit going
  forward.
- **Windows-first, and the accessibility layer is more Windows-only than it looks.** wxWidgets
  gives native, accessible controls on all three platforms, but two things this project relies on
  exist only on Windows: `wxAccessible`, which is how `set_accessible_name` reaches the
  accessibility tree, and `UiaRaiseNotificationEvent`, which is how announcements are spoken and
  brailled. Both compile and silently do nothing elsewhere. A macOS or Linux port needs its own
  bridge for each, not a framework change. Platform-specific code sits behind
  `#[cfg(target_os = "windows")]` with a fallback that keeps the crate building.
- **Secrets stay out of the tree, and out of the database.** OAuth client credentials load from
  `oauth.toml` (gitignored) with `oauth.toml.example` as the tracked template. Every other secret
  goes to the OS credential store via `keyring`: account passwords through `service::credentials`,
  tokens through `service::oauth`, CalDAV sign-ins through `service::caldav`. Nothing sensitive is
  written to `message_cache.db`, so the database can be copied and backed up without carrying
  credentials, and uninstalling can clear the secrets by clearing one place. Each service name has
  exactly one owner, because the code that erases them has to name the same entries as the code that
  wrote them. Never log a token, password, or message body.
- **The cached mail is not encrypted, and the docs say so.** Do not claim otherwise anywhere.
  Encrypting it means encrypting the whole database, which is a decision with a build cost, not
  something to imply in a feature list.
- **Every setting a user has is reachable from the settings screen, and sorted into a section
  somebody would look in.** Not "the model supports it", not "it can be set by editing the stored
  file". A setting nobody can find is a setting nobody has, and one buried where it does not belong
  is nearly as bad for somebody moving by keyboard through a screen reader, because they meet the
  sections in order and cannot skim. This binds a new setting in the plan that introduces it, rather
  than as a later pass.

  **Two settings break this rule today and neither is new.** Per-event feedback channels: `per_event`
  and `set_event_channels` in `src/presentation/accessibility/feedback.rs` are both private, so no
  screen *could* write one, and an override can only arrive by hand-editing the stored settings.
  That is FEEDBACK-01 and it belongs to phase 6. And the per-account Allow Changes answer, which the
  model holds and nothing offers. Phase 1's criterion 8 already said a phase must not add a third.

  **A check does enforce this, for top-level settings, and the first version of this paragraph said
  otherwise.** Corrected 2026-09-06, the same day it was written, which is the rule this file gives
  about writing a rule down: it claimed nothing enforced this without going to look.
  `every_setting_is_acted_on` in `src/data/config.rs` reads the `AppConfig` struct itself, so a
  field added later is covered without anybody remembering, and
  `test_every_setting_somebody_can_change_is_offered_by_a_screen` asserts every public field is
  offered by a screen. Its own comment says why it is the mirror of its neighbour and what each is
  blind to. So **a new setting added as a top-level field fails on arrival**, which is the red half
  for free.

  **The paragraph above was corrected once and the correction was also wrong.** It said both
  exceptions were nested settings and that widening the check to follow nesting would close them.
  Neither half is true, found on 2026-09-06 by phase 6's research going to look. Twice wrong about
  the same four sentences is worth leaving on the record rather than tidying away, because the cause
  both times was writing down what the shape of the problem ought to be instead of reading it.

  The two exceptions are two different shapes, and neither is nesting.

  `allowed_per_account` is a **top-level** `AppConfig` field, `config.rs:276`. The check can see it
  perfectly well. It is excused by name, through `STORED_AND_OFFERED_BY_NOTHING` at `config.rs:1799`,
  a list that today holds exactly one entry.

  The per-event feedback channels are a third shape that no name-based check can ever reach: they
  live inside the **serialised string value** of `feedback_channels`, not as fields of anything. No
  amount of following nesting finds them, because there is no field to find.

  So **ledger 114 as written closes neither of the two things it was raised for.** What would is a
  hand-named companion on the pattern of `test_whether_message_text_may_be_fetched_is_offered_by_a_screen`
  at `config.rs:1829`, which costs two guard records rather than disturbing all five tests in that
  module.

  **And emptying the exception list disarms the guard watching it.** With one entry gone,
  `test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing` at `config.rs:1947`
  iterates over nothing and passes unconditionally. That is the census-emptying failure this file
  already describes two sections up, waiting to happen again, so whoever offers the per-account
  answer from a screen retires that guard in the same commit rather than leaving it green and blind.

  `test_nothing_offers_a_setting_per_account_that_no_screen_writes` in `tests/house_style.rs` is not
  that check and should not be mistaken for it: it reads documents for a phrase and catches a *page*
  promising a control that does not exist.

- **Schema changes are additive.** `MessageCache` opens existing user databases, so add tables with
  `CREATE TABLE IF NOT EXISTS` and columns with `ensure_column_exists`. Never drop or rename a column
  that shipped. The one exception taken so far is dropping a table that held secrets nothing read;
  if that case comes up again, say why in the commit.

### Versioning and releases

**The version the tree carries is the next build to go to testers, and since 2026-09-16 that is a
stage of `1.0.0`.** Pratik decided on 2026-09-15 (#46), testing the first build handed to him, that
the builds going to testers are the alpha, beta and release-candidate stages of `1.0.0`, so the tree
moved from `0.125.1` to `1.0.0-alpha.1` by hand, once. Until 2026-09-16 this paragraph read
"Development happens on plain `0.x.y`. Minor for feature work, patch for fixes. No suffix. `0.x`
already means unstable in SemVer, so a version does not need `-alpha` on top of it to say the same
thing twice, and it should not claim a testing programme that is not running." That held while no
build went to anybody. One does now, so the suffix says something true.

**How a version moves inside a prerelease.** The tree stays at `1.0.0-alpha.1` through the fixes
that follow the first day of testing, because that build has not been cut and the number already
names something newer than any build handed out. After a build is cut, the first behaviour change
moves the prerelease counter, once, in the commit that makes it: `1.0.0-alpha.2`. Later behaviour
changes before the next cut do not move it again, because the version is a name for the next build
and not a count of commits. Documents and bug fixes that change no behaviour move nothing, as
before. `alpha` becomes `beta` and `rc` the same way when Pratik says the round has moved on, and
`release` drops the suffix to `1.0.0` when it closes. After `1.0.0`, minor for features and patch
for fixes. Until 2026-09-16 the paragraph here read "A prerelease suffix stages a release that is
about to go to people. When builds start going to testers, cut `0.6.0-alpha.1`, then `-alpha.2`,
then `0.6.0` when that round closes. That is what prerelease identifiers are for." It was the same
rule with the numbers left open, and it did not say when the counter moves, which is the sentence
above. Twenty-five `0.1.0-alpha.N` versions were cut before either rule existed, none of them
tagged or published, because the version was being used as a build counter.

**`patch` is never dispatched while the version carries a suffix.** cargo-release reads `patch` on
`1.0.0-alpha.N` as "remove the suffix" and cuts `1.0.0` as a full release. The Release workflow's
`as-is` level publishes the version the tree carries without bumping it, and is the level the alpha
uses; without it, `alpha` on a tree at `1.0.0-alpha.1` would publish `-alpha.2` and nothing could
ever publish `-alpha.1`.

**"Alpha" as a state of the product belongs in the product, not in the number.** Nobody reads a
version number and learns that sending mail has never touched a real server. The first-run screen,
the Allowed Changes settings, the end of `--help` and `docs/ALPHA_TESTING.md` say it in sentences.

**Bump when the software changes, not when a build changes hands.** Several builds share a version,
so `scripts/build-installer.sh` appends, since 2026-09-17, how many commits the build is past the
commit that set its version and then the commit it was built from: `1.0.0-alpha.1+42.g59c5b6a4`, in
the file name, in Apps and Features, in `--version` and in the first line of the log. The same
counter goes into the last part of the Windows file version, under the stage and the step, so Apps
and Features orders the builds the way the counter does. After a `+` because that is build
metadata, which version ordering ignores: the counter moves nothing in the version, and the update
check reads `1.0.0-alpha.1+42.g59c5b6a4` as `1.0.0-alpha.1`. Nothing is appended at a tag, since
that build is the release, and the file version keeps the counter there so the release sits above
the candidates that staged it. See `src/common/version.rs`.

Why the counter: Pratik decided on 2026-09-17, holding two builds of `1.0.0-alpha.1` that no number
could put in order, that the round's name stays and the order goes after the plus. Two shapes were
declined. A counter inside the prerelease, `alpha.1.2`, which `version::parse` refuses, which
cargo-release's `alpha` level would count on from, and which is the version-as-build-counter the
rule above warns against. And a release per tester build. Until 2026-09-17 the script appended the
commit alone, `0.5.0+g64c73dd`, and two builds of one version were tellable apart and not orderable.

Two caps, because a Windows file version field holds 65535 at most: the step is held to 12 and the
counter to 999 in the file version, and a build past either says so on the console when it is built
and keeps the true number in the string. A clone that does not hold the commit that set the version
cannot count and is refused with a sentence rather than built; CI's Build job checks out the whole
history for that reason.

A bug fix or a docs pass does not need a bump. A new feature, a schema change, or a behaviour change
does, in the same commit as the change rather than in a jump at release time. Inside a prerelease
the bump is the counter moving on the first behaviour change after a cut, as above, and only then.

`docs/changelog.md` is the record. Every user-visible change gets an entry under `[Unreleased]` in
the commit that makes it, and honest "Known limitations" notes belong there too. A feature list that
implies something works when it does not is worse than no entry.

Releases are cut deliberately, never as a side effect of a push. The mechanics of dispatching one,
and which levels publish as prereleases, are in the `cutting-a-release` skill.

<!-- END GUARDRAILS -->
