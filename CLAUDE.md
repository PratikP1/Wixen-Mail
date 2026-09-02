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
   only a real NVDA or Narrator run proves experience, and the automated scan covers about
   half of WCAG. Worse, a call can look like accessibility and not be one. (Sixteen widgets
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

Every commit builds, passes tests, and passes lint. These are the same four checks CI runs, and
clippy is enforced with `-D warnings`, so a warning is a build failure:

```bash
bash scripts/check.sh
```

Use the script rather than running the four commands by hand. Cargo shares build
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
`scripts/which-checks.sh` decides it.** On `main`, all four, whatever changed:
every commit here lands on it. On a branch nobody builds, the slow half waits
for the merge, and what runs is scoped to the change. A commit touching only
documents runs formatting, clippy and the three targets that read documents; a
commit touching code runs those plus the tests reaching the modules it changed,
plus the guards that read the whole tree. Measured 2026-08-31: a four-file
markdown commit went from about 330 seconds to 36.

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
Measured warm on 2026-08-30: the whole gate is 311 seconds, of which the test
suite is 239 and the release build 56, so the quick pair is 15 seconds and the
slow two are everything else. On a branch they wait for the merge, where they
run once rather than once per commit. Whoever merges runs `scripts/check.sh all`
first, and that is the run they are paid for.

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

**The scripts that decide all this have their own suites, and the gate runs
them.** `scripts/*.test.sh` runs on every invocation of `check.sh`, in every
mode, before anything else, and in CI. It costs milliseconds. For one day these
suites existed and nothing ran them, which is guardrail 4 exactly: a check
nobody reads is worse than no check, because it reads as covered.

Never silence a lint with `#[allow(...)]` to get a commit through. Fix the code, or if the lint is
genuinely wrong for this case, add the allow with a comment saying why.

### Tests that would notice

A green suite says the code does what the tests say. It does not say the tests
would notice if it stopped, and those are different claims. Red/green is what
keeps them together, and it started at commit 182 of 344, so most of the tests
here were written after the code they cover and describe it rather than specify
it. Three tests written in one session to catch a named bug passed against that
bug.

```bash
scripts/mutants.sh src/service        # one directory, slow
scripts/mutants.sh --since v0.19.0    # only what changed, minutes
```

Name a real commit or tag to compare against. Every commit here lands on `main`,
so `--since main` compares `main` with itself, finds nothing, and now says so
instead of passing. That was written down here as the way to check a change for
275 commits and could never have tested a line.

Mutation testing alters the code in small ways and runs the suite. Anything
nothing catches is either untested behaviour or dead code. A whole-tree run is
about two days, so it is used scoped, and the pull request check runs it on the
diff only. Before trusting a new regression test, take the fix out and watch the
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
it was written, 17 a day later, 21 the day after, and 31 by the end of the
phase. So: **any change that adds tests near a rule re-measures that rule's
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
was. And what it costs when it fires is not flat: 471 of the 548 records name one
file, but a test added to `src/application/contacts_sync.rs` flags 74 of them,
which at a build and a run each is hours. That case is what the paragraph below
about the critical path is for, and the answer is to run the command it prints in
the background rather than to lower the check.

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

So: the executor does not run guards, and neither does the merge. After merging,
run the records the branch could have disturbed, in the background:

```bash
scripts/guards.sh --touched-by <the commit the branch left main at>
```

That reads the diff and measures every record those files could have made stale,
both ways round: the guarded file changed so the break may no longer apply, or a
record already names a test in a module that changed so its red set may have
grown. What it finds becomes a follow-up commit correcting the record.

**Know what it costs before starting it.** For plan 02-01 that is 63 records of
536, about 90 minutes. Narrowing the rule to modules that actually gained a test
only reaches 52, because one large shared file gained tests and many records name
a test in it, so there is no clever selection that makes this quick. The whole
sweep is 536 records and about 15 hours, which is an overnight job rather than an
impossible one since the thread setting halved it.

That setting is `WIXEN_TEST_THREADS`, it defaults to 4, and it applies to the
guard runs only. Measured on 24 logical cores over the 5,837-test library run on
its own: 2 threads 131s, 4 threads 88s, 8 threads 106s, 16 threads 164s, and the
harness default of one per core 196s. The suite is contended rather than
compute-bound. **It does not carry to `scripts/check.sh`**, which runs
`--all-targets`: there the test term falls from 197s to 111s and the whole gate
does not move, 335s against 353s, so the gate does not set it. Reading the
isolated figure as though it applied everywhere was the mistake, and timing the
gate rather than assuming is what caught it.

Running the records in parallel across git worktrees was measured and rejected:
two concurrent suites take 131s each against 88s alone, so the contention is on
something shared rather than on the processor and crosses process boundaries.
Five workers would buy about 1.8x for 43GB of disk and a five-minute build each.

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

Automated checks catch roughly half of accessibility defects. They do not replace testing with real
assistive technology. Structure present is not experience good.

**Windows has two accessibility channels and this project needs both right.** UI Automation is what
Narrator reads. MSAA, through `IAccessible`, is what NVDA reads for native controls, and it is the
only place `set_accessible_name` writes: for an edit box or a button, Windows supplies its own UI
Automation provider that shadows the MSAA object underneath. So a UI Automation scan reports the
system's name for those controls and never the one the code set. The accessibility workflow runs
Axe.Windows over UI Automation and `scripts/msaa-names.ps1` over MSAA, per window, and a name that
fails on either channel is a name somebody does not hear.

A control with a visible label beside it gets that label as its MSAA name even when nothing set one,
because Windows falls back to the nearest static text. That is a real name and it is really spoken,
so a clean run does not mean every name came from this code.

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
- **Schema changes are additive.** `MessageCache` opens existing user databases, so add tables with
  `CREATE TABLE IF NOT EXISTS` and columns with `ensure_column_exists`. Never drop or rename a column
  that shipped. The one exception taken so far is dropping a table that held secrets nothing read;
  if that case comes up again, say why in the commit.

### Versioning and releases

**Development happens on plain `0.x.y`.** Minor for feature work, patch for fixes. No suffix. `0.x`
already means unstable in SemVer, so a version does not need `-alpha` on top of it to say the same
thing twice, and it should not claim a testing programme that is not running.

**A prerelease suffix stages a release that is about to go to people.** When builds start going to
testers, cut `0.6.0-alpha.1`, then `-alpha.2`, then `0.6.0` when that round closes. That is what
prerelease identifiers are for. Twenty-five `0.1.0-alpha.N` versions were cut before this rule
existed, none of them tagged or published, because the version was being used as a build counter.

**"Alpha" as a state of the product belongs in the product, not in the number.** Nobody reads a
version number and learns that sending mail has never touched a real server. The first-run screen,
the Allowed Changes settings, the end of `--help` and `docs/ALPHA_TESTING.md` say it in sentences.

**Bump when the software changes, not when a build changes hands.** Several builds share a version,
so `scripts/build-installer.sh` appends the commit it built from: `0.5.0+g64c73dd`, in the file name,
in Apps and Features, in `--version` and in the first line of the log. After a `+` because that is
build metadata, which version ordering ignores. Nothing is appended at a tag, since that build is
the release. See `src/common/version.rs`.

A bug fix or a docs pass does not need a bump. A new feature, a schema change, or a behaviour change
does, in the same commit as the change rather than in a jump at release time.

`docs/changelog.md` is the record. Every user-visible change gets an entry under `[Unreleased]` in
the commit that makes it, and honest "Known limitations" notes belong there too. A feature list that
implies something works when it does not is worse than no entry.

Releases are cut deliberately, never as a side effect of a push. The mechanics of dispatching one,
and which levels publish as prereleases, are in the `cutting-a-release` skill.

<!-- END GUARDRAILS -->
