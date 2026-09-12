---
phase: 07-installing-updating-and-what-is-stored
plan: 07
subsystem: infra
tags: [release, github-actions, authenticode, code-signing, smartscreen, guard-records]

requires:
  - phase: 07-02
    provides: "tests/installer.rs, the target this plan writes into, and the guard record carrying suite = \"installer\""
  - phase: 07-04
    provides: "the finding that a published tag carries a v, read off the glob rather than off any tag"
provides:
  - "A release that cannot produce a file it promised stops before it publishes anything, and names the file"
  - "A test holding every published glob against a name the build really writes, in two categories with the exception named"
  - "A test holding the release to a single trigger, so guardrail 7 is enforced rather than remembered"
  - "The certificate decision on the record with the three options that lost and which of their reasons could move"
  - "Roadmap criterion 1 corrected against Microsoft's page, re-fetched rather than quoted"
  - "Roadmap criterion 2 replaced in full per D-14, which plan 07-09 is measured against"
  - "SHIP-02's fourth [D] line for the release channel per D-15, and its first line replaced per D-16"
  - "A guard record is now read whatever kind of file its break lands on, so a record naming a workflow reaches the gate"
affects: [07-08, 07-09, any plan that adds a guard record whose break lands outside src]

actuals:
  tokens: 13016
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A published file list written once and read by both the step that checks it and the step that publishes it"
    - "Two categories for a published-file rule, built-then-published and published-as-it-stands, with the second naming exactly what it excuses and checked in both directions"

key-files:
  created: []
  modified:
    - .github/workflows/release.yml
    - tests/installer.rs
    - guards/guards.toml
    - scripts/check.sh
    - scripts/check.test.sh
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/WINDOWS.md

key-decisions:
  - "Two categories in the published-file rule rather than three globs, because a default category would silently absorb a fifth glob added later"
  - "The four globs live in one job-level env entry, so the checking step and the publishing step cannot drift apart"
  - "The tag-before-build ordering is recorded as a finding and not changed, because moving it changes when a release can happen"
  - "The guard-record coupling in check.sh was widened rather than duplicating the .iss special case in which-checks.sh, because a workflow reaches one suite rather than every target"

patterns-established:
  - "A test written over a rule that already holds has no red half, and its honest substitute is its own recorded break in guards/guards.toml"
  - "Where the shape of a produced string is decided by configuration, guard the configuration rather than the string"

requirements-completed: []

status: complete
---

# Phase 07 Plan 07: A release that stops rather than publishing what it has, and the certificate decision Summary

A release that cannot produce one of the four files it promises now stops and names it, proved by a
test that reads the workflow and the build script rather than by somebody having read both; and the
certificate decision is on the record with the three options that lost.

**Branch:** `a-release-that-cannot-produce-a-file-it-promised-says-so`, off `main` at `3e52ab85`,
**merged at `eab73a40`.**

## What landed

Five commits, each its own change.

| Commit | Kind | What |
|--------|------|------|
| `9694cd26` | RED | Seven tests over `.github/workflows/release.yml` in `tests/installer.rs` |
| `7cdf5fb5` | GREEN | The existence check, `fail_on_unmatched_files: true`, one published list, two guard records, three ledger entries |
| `91fe4c9b` | RED | Four cases over the guard-record coupling in `scripts/check.test.sh` |
| `a19234cc` | GREEN | The coupling reads a record whatever kind of file its break lands on |
| `3271db96` | docs | SHIP-01's decision block, criteria 1 and 2, SHIP-02's first and fourth `[D]` lines, one ledger entry |

### Task 1: a release that cannot keep a promise stops

`release.yml` publishes four globs. `fail_on_unmatched_files` was `false`, which the action's own
documentation calls the "Indicator of whether to fail if any of the `files` globs match nothing", so
a release that could produce three of the four published three, went green, and left the fourth to be
found by somebody trying to download it.

**Two nets now, and the order between them is the point.** A step called "Check that every promised
file exists" runs after "Prepare the portable download" and before "Publish GitHub release assets".
It reads the promised list, checks each pattern matches a file, and fails naming every pattern that
matched nothing. `fail_on_unmatched_files` is `true` as the second net, and it is not a substitute
for the first: it fires inside the publishing step, by which time the tag is on the remote and the
release exists, so a missing asset caught only there is already a half-published release.

**The four globs are written once**, in a job-level `RELEASE_ASSETS` entry that both steps read. Two
copies of a list agree until somebody edits one of them, which is the drift the whole change is
about. The test follows the indirection rather than assuming it: `files:` either opens a block of its
own or names an `env` entry that does, and the reading takes whichever it is.

**Seven tests, of which one had a red half.** That is worth stating plainly rather than leaving to be
noticed:

| Test | Red half |
|------|----------|
| `test_a_release_that_cannot_produce_a_promised_file_stops_before_it_publishes` | Yes, named in `9694cd26` |
| `test_every_file_the_release_promises_is_one_it_really_produces` | No, the names already agreed |
| `test_a_release_still_happens_only_when_somebody_asks_for_one` | No, the trigger was already right |
| `test_nothing_here_moves_the_published_tag_away_from_the_shape_a_glob_expects` | No, no such configuration exists |
| `test_the_reading_can_tell_a_promise_the_release_keeps_from_one_it_does_not` | Companion, fixtures |
| `test_the_reading_can_see_a_release_that_publishes_before_it_checks` | Companion, fixtures |
| `test_the_reading_can_see_a_release_trigger_that_was_widened` | Companion, fixtures |

Three of those are guards over rules that were already true, so they have no red half by
construction. The honest substitute is a recorded break, and two of the three have one in
`guards/guards.toml`. The third is measured by hand below, because the break that would redden it
cannot be expressed as a text substitution.

**The two categories, and why not three globs.** The plan's acceptance criterion offered a choice:
hold every glob to a built name, which is impossible because `docs/changelog.md` is a tracked file
nothing builds, or say the criterion covers three globs. **Neither was taken.** The rule has two
categories, built-then-published and published-as-it-stands, and the second is a list of exact paths
holding one entry. It is checked in both directions: a path excused there that nothing publishes
fails as loudly as a glob nothing builds, because an exception that outlives the thing it excused is
how the rule rots without anybody seeing it. The weaker option was rejected for the reason the
criterion itself gives, that a fifth glob added later joins whichever category is the default.

**The tag shape, and where it is really decided.** `dist/wixen-mail-$tag.exe` is written and
`dist/wixen-mail-v*.exe` is published, so the two agree only while the tag begins with `v`. No tag
has ever existed, so the shape comes from cargo-release's defaults, whose `tag-name` is
`{{prefix}}v{{version}}` with an empty `tag-prefix` at a repository root. Rather than write that
assumption into a comment and assert under it, the test **guards the configuration that would break
it**: it fails if `release.toml` or `.cargo/release.toml` appears, or if `Cargo.toml` grows a
`[package.metadata.release]` section. That is where the shape of the string is decided, and it is the
half a reader would not think to check. The assumption itself stays unverified until a release is
really cut, which the ledger carries.

### Task 2: the certificate decision, written down

`SHIP-01` now carries the decision as a decision. Azure Artifact Signing, taken by Pratik on
2026-09-06, about $9.99 a month, publisher name Pratik Patel, no hardware token, and the United
States or Canada residence requirement met, asked and confirmed the same day. The existing evidence
block is left where it was, because it is what the requirement was judged against.

The three that lost, each with what it lost on, and **which of those reasons could move**: the OV
certificate is a price and prices move, so the comparison is worth reading again if an OV certificate
plus a cloud HSM falls under $120 a year; SignPath is a judgement about whose name goes where users
look, and that will not move; not signing lost on Smart App Control and on reputation never starting
to accrue while nothing is signed.

**What the decision commits the project to**, so 07-08 is not estimated as "sign two files": four
distinct binaries, three inside the installer and the setup itself, plus the uninstaller whose
`SignedUninstaller` prompting behaviour is recorded as unverified and would hang a CI job rather than
fail it. The portable copy and the zip are not two more jobs: both are made from
`target/release/wixen-mail.exe` after the build runs, so the portable copy inherits the signature
already in those bytes, and a zip is a container rather than a signable format.

**What it does not buy**, which is the part everybody gets wrong: no certificate available to this
project removes the SmartScreen warning on a first download, and `PrivilegesRequired=lowest` means the
"Unknown publisher" line a signature really does fix is not shown at all on a per-user install, which
is the common case.

**Microsoft's page was re-fetched rather than quoted.** Read 2026-09-12 at
`https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation`. It still
says "EV certificates no longer bypass SmartScreen. Years ago, signing files with an Extended
Validation (EV) code signing certificate would result in positive SmartScreen reputation by default,
but this behavior no longer exists", and its table still gives "Valid Certificate (OV/EV)" the same
first-download outcome, a warning until reputation accumulates. Its `updated_at` metadata is
2026-08-17, the same date the research recorded, so **the page has not moved since**. Its `ms.date`
is 2026-05-04, which is a different field and is the one a reader would meet on the page; worth
knowing which of the two the research's date came from.

**Criterion 1**, replaced parenthetical only. The word diff confirms the first sentence and the
closing clause are byte-identical:

```
  1. The published installer ... stated, not promised: [-only an EV-]{+no+} certificate
     [-carries reputation from-]{+available to this project removes+} the {+warning on a+} first
     download, {+since EV certificates no longer bypass SmartScreen and only publishing through the
     Microsoft Store avoids it. ...+} and [-while-]{+Smart App Control on Windows 11 no longer
     blocking the file. While+} the warning remains, `docs/installing.md` keeps the walkthrough ...
```

**Criterion 2**, replaced whole, per D-14. Old and new side by side:

> **Old.** The application can tell the user a newer version exists, as a deliberate action or an
> explicit setting, never as a silent background fetch, and declining leaves the current version
> working.

> **New.** The application can tell the user a newer version exists and can apply it, and nothing is
> fetched that the user did not ask for. Asking is one of two things: choosing the update item in the
> Help menu, which works whatever the setting says, or choosing a release channel in the one update
> setting, which starts off and says where it is chosen that choosing a channel means installers are
> downloaded automatically. With a channel chosen, the check, the download and the verification
> happen unattended; running the installer never does, and the user is asked first. A downloaded
> installer is verified before anybody is asked about it: the Authenticode signature must be valid
> and the signer must be this project's own publisher name, and a file that fails either check is
> refused and deleted rather than warned about and offered anyway. Where a signature cannot be
> checked at all, nothing is downloaded and nothing is run, and the reason is said. Declining, and a
> handover that fails, both leave the current version working.

Both halves of the verification are required and each has its own clause: "the Authenticode signature
must be valid **and** the signer must be this project's own publisher name". The refusal is a refusal
rather than a warning: "refused and deleted rather than warned about and offered anyway".

**How the new wording tells consented from silent.** It does not ask whether a question was asked at
fetch time. It asks where the consent was given: at the Help menu item, or at the setting, "which
starts off and says where it is chosen that choosing a channel means installers are downloaded
automatically". So an unattended fetch is consented to because somebody turned it on at a control
that said what it would do, and what is forbidden is a fetch nobody chose rather than a fetch nobody
watched.

**This plan wrote criterion 2 and plan 07-09 is measured against it. 07-09 was not read while writing
it.** The wording was taken whole from `PLANS-README.md`, which is what stops it being adjusted
toward whatever 07-09 currently plans to build.

**SHIP-02.** The first `[D]` line said the check is "never a silent background fetch", which reads
false under D-16, and is replaced with the `PLANS-README.md` wording. The fourth `[D]` line for the
release channel is added per D-15. The other two, about applying being the user's decision and about
the comparison ignoring `+build`, were read and left alone because both are still exactly true; the
diff of the SHIP-02 block shows exactly one `[D]` line removed.

Neither SHIP-01 nor SHIP-02 is marked complete and neither row in the requirement-to-phase table
moved.

## Deviations from the plan

### 1. [Rule 3] The gate could not read the guard records this plan added

**Found during:** immediately after the task 1 GREEN commit, by reading which targets the hook
selected rather than only its verdict.

**Issue.** `scripts/check.sh`'s record-to-suite mapping opened with a filter discarding every changed
path except `src/*.rs`. The reason written beside it was that `--lib` was never going to reach a file
that is not Rust source. That is true about `--lib`, and the function does not answer with a `--lib`
filter, it answers with a `--test` target. So the filter discarded exactly the records the mapping
exists for. The two records this plan added naming `.github/workflows/release.yml` sat in the registry
looking like coverage, and the commit that changed the workflow ran the four tree-reading guards and
nothing that reads the workflow.

This is the success criterion "your tests actually run on your own commits", and without the fix it
would have been false for every future commit to `release.yml`.

**Fix.** Four cases in `scripts/check.test.sh` first, two of them red. The filter now excludes a
changed `tests/*.rs` only, because that already runs its own target a few lines up and would be asked
for twice. Verified end to end afterwards:

```
$ bash scripts/check.sh --suites-for guards/guards.toml .github/workflows/release.yml
installer
$ bash scripts/check.sh --suites-for guards/guards.toml installer/Wixen-Mail-Setup.iss
installer
$ bash scripts/check.sh --suites-for guards/guards.toml .github/workflows/ci.yml
(nothing)
```

**Two existing cases pinned the old rule and were corrected in place**, not deleted, with the wrong
reason quoted above them. Changing a test to make a change pass is the shape that should always be
argued for out loud, so the argument is written where the cases are. The third case in that group, a
record whose break lands on a test file, is unchanged and still right.

**Commits:** `91fe4c9b` and `a19234cc`.

### 2. [Rule 1] SHIP-01 carried a `[D]` line saying the decision was unmade

Not in the plan's list of lines to change. It read "This is blocked on a certificate decision that is
Pratik's. Until it is made, the requirement stays open". The decision was made on 2026-09-06, so the
line stated something false in the same requirement whose whole point this plan was to correct. It
now says the certificate is chosen and the account is not created, with the old text quoted. What the
requirement waits on is named: a subscription, an identity check, a payment and a role grant, none of
which is a repository operation.

### 3. [Rule 1] The `docs/ALPHA_TESTING.md` citation was three line numbers stale

The evidence cited `:146`. `grep -n 'The installer is not signed' docs/ALPHA_TESTING.md` answers
`342`. The sentence has carried three different numbers in a week as the file grew from about 160
lines to 365, so it is now cited by its words rather than by a line, which is what the requirements
audit of 2026-09-04 asked for.

## What is still false in the plan after the correction pass

**One acceptance criterion contradicts the premise that was corrected to fix it.** Task 2's
acceptance criteria say "The evidence says the decision commits the project to six artefacts plus the
uninstaller". Premise 7 was corrected on 2026-09-11 to say that six does not follow from its own
evidence, to write the honest description instead, and explicitly not to replace six with another
single number. The correction reached the premise and not the criterion that quotes it.

The honest version was written, per the premise. A reader following the criteria mechanically would
have restored the defect the correction existed to remove. The cause looks structural rather than
careless: a correction pass reads for wrong claims, and an acceptance criterion is phrased as an
instruction, so it does not trip claim-checking attention even when it repeats the claim word for
word.

**One verification line was already corrected and is worth confirming as correct.** The plan says
task 1's file list answers `affected` and task 2's answers `docs_only`. Measured on both commits:

```
$ bash scripts/which-checks.sh <branch> .github/workflows/release.yml guards/guards.toml .planning/WINDOWS.md
affected
$ bash scripts/which-checks.sh <branch> .planning/REQUIREMENTS.md .planning/ROADMAP.md .planning/WINDOWS.md
docs_only
```

**The premise about `tests/installer.rs`'s guard cost was right.** Re-measured by parsing
`tests_last_seen` rather than by grepping the file name: **1 record** named it, as the premise
predicted 07-02 would leave. The parse:

```
$ awk '/^\[\[guard\]\]/ {if(n>0) c++; n=0}
       /^tests_last_seen/ {b=1; next} b && /^\]/ {b=0; next}
       b && /file *=/ && /installer/ {n++}
       END {if(n>0) c++; print c+0}' guards/guards.toml
1
```

**The correction about `.planning` being unreadable by any test was right to make.** Both
`cargo test --test house_style` and `cargo test --test the_planning_files_agree_with_themselves` read
the two files task 2 writes, and both were run before committing rather than after. The rules they
hold the new wording to are the em-dash rule, the banned-word rule, and the requirement that a ledger
entry appear in both the markdown table and the JSON block with the same status and description. All
three passed. Had the plan's original sentence been written into this summary it would have been a
falsehood.

## The certificate decision was answered, not dropped

The plan originally ended in a blocking checkpoint asking which certificate to use. That checkpoint
was **answered** by decision 6 of `.planning/decisions-2026-09-06.md`, not removed, which is why this
plan ran start to finish without Pratik. A reader of the plan history should not read a dropped
checkpoint as a dropped question.

## Nothing here changes when a release can happen

Said in as many words, per guardrail 7 and threat T-07-30. The `on:` block is byte-identical: still
`workflow_dispatch` alone, with the same six levels. No token, no permission and no step that could
tag, publish or push was added. `permissions: contents: write` is unchanged. The full diff of
`.github/workflows/release.yml` adds one job-level `env` entry, one checking step, and changes
`files:` to read that entry and `fail_on_unmatched_files` from `false` to `true`.

**Every change here makes publishing stricter and none makes it easier.** A release that could
previously publish three of four files now fails instead. That is the only behaviour that moved.

And it is held by a test rather than by this paragraph:
`test_a_release_still_happens_only_when_somebody_asks_for_one` reads the `on:` block and refuses any
trigger but `workflow_dispatch`, with a companion proving the reading can see a widened one, and a
guard record whose break swaps the trigger.

## Guard records

Two added, both measured by hand with `cargo test --all-targets --no-fail-fast` at the default eight
threads on the tree the commit makes. `guards/guards.toml` goes from 729 records to **731**, and the
census at the top of the file moved from `192` and `537` to `192` and `539` in the same commit.

**1. "a published glob still matches a name the release really writes".** The break changes the glob
rather than removing it, which is the half-fix and the shape a real drift takes:

```toml
before = '        dist/Wixen-Mail-*-windows.zip'
after  = '        dist/Wixen-Mail-*-win.zip'
```

Full red list, and it is the whole of it: `test_every_file_the_release_promises_is_one_it_really_produces`.
7,077 passed in the library, 49 other targets green.

**2. "a release still happens only when somebody asks for one".** The break swaps
`  workflow_dispatch:` for `  push:`. Full red list:
`test_a_release_still_happens_only_when_somebody_asks_for_one`. 7,077 passed in the library, 50 other
targets green.

**3. The tag-shape guard has no record and is measured here instead.** The break that reddens
`test_nothing_here_moves_the_published_tag_away_from_the_shape_a_glob_expects` is the *appearance* of
a file that does not exist, and `guards.toml`'s mechanism is a text substitution inside an existing
file, so it cannot express one. Measured by hand on 2026-09-12 instead: writing
`release.toml` containing `tag-prefix = "wixen-"` turns that test red and leaves the other ten in the
target green; the file was removed immediately afterwards and `ls release.toml` confirmed it gone.
Recorded here rather than left as a guard nobody can break.

**The existing installer record was re-measured, not edited.** Adding seven tests to
`tests/installer.rs` turned `test_every_guard_record_says_how_many_tests_the_files_it_names_held` red,
which is the collision `CLAUDE.md` describes, so it was named in the RED commit's trailers alongside
the real failure. `scripts/guards.sh --remeasure "the icon a shortcut names is the one the installer
really put there"` then reported "all 2 tests named went red, and nothing else did" and wrote the
count from 4 to 11. The red list was still right, which was predicted and then measured rather than
assumed.

## One failure that is nobody's change

Three whole-tree runs were taken on 2026-09-12 for the guard measurements. **Two of them carried a
failure that has nothing to do with anything in this plan**, in
`tests/a_move_says_what_has_not_been_sent.rs`:

```
an account to look the folder up against: Security("Could not remove the password for acct in
the Windows credential store: Security error: Could not reach the credential store: No default
store has been set, so cannot search or create entries")
```

A **different test of that file** each time, and the file passes on its own on an unbroken tree. It
is the OS credential store being intermittently unreachable on this machine, it is not diagnosed, and
it did not appear in the final `scripts/check.sh all` run. Written down rather than filtered out:
reading a measurement run by grepping for the failure you expected is what stops it finding the one
you did not. Ledger **328**.

## Broken windows ledger

Four entries, 326 to 329, each present in both the markdown table and the JSON block.
`.planning/WINDOWS.md` goes from 325 entries to **329**, open from 303 to **307**.

| id | kind | What |
|----|------|------|
| 326 | unrun-verify | The release workflow has never run. `git tag` returns nothing and `git ls-remote --tags origin` returns nothing while `--heads` answers, so this is settled rather than a silent network failure. No published glob has ever been matched against a real `dist/` and the tag branch in `scripts/build-installer.sh` has never been taken. What goes wrong if the tag shape is not what the defaults say: `dist/wixen-mail-v*.exe` was published as a silent absence before this change and is a failed release after it, and the second is what was wanted |
| 327 | deviation | `cargo release` pushes the tag before anything is built, so any failure after that leaves a tag on the remote with no release behind it. The new check moves the failure earlier than publication but not earlier than the tag. Changing it means moving `cargo release` after the build, which changes when a tag exists, and guardrail 7 says that is a deliberate decision rather than something a change about asset names makes on the way past |
| 328 | deviation | The credential-store failure above |
| 329 | unrun-verify | Criterion 1 cannot close until an Azure Artifact Signing account exists. A dependency on something outside this repository rather than a defect in it |

## Verification

**`scripts/check.sh all` on the branch tip, all four green.** Not piped into anything; redirected to a
file and the exit status read from the script. **506 seconds**, inside the 419 to 654 the six landed
branches report. 7,077 tests in the library, 51 targets green, zero failures, then the release build.

Per commit, through the hook, nothing using `--no-verify`:

- `9694cd26` RED, accepted by `red-commit.sh`: "a red that is exactly the one this commit named",
  both named tests ran, both failed, nothing else failed.
- `91fe4c9b` RED, same, for the two shell-suite cases named `check::<description>`.
- The three others ran formatting, clippy, the shell suites and their scoped targets.

`cargo test --test installer`: 11 passed, 0 failed.

**No version bump and no changelog entry.** `CLAUDE.md` ties a bump and an entry to a user-visible
change, and nobody running the program can tell the difference. A release workflow that refuses to
publish an asset it could not produce is not something somebody using Wixen Mail meets, and inventing
an entry for it would be worse than none. The gate fix in `scripts/check.sh` is the same: that is the
gate, not the program.

**Task 2 is documentation and has no RED commit.** `CLAUDE.md` lists documentation as one of the
exceptions to test-first, and the exception is taken for that reason. What was considered: the two
document-reading targets do now read `.planning`, so a test that *reads* the new wording exists and
was run, but neither can assert that the certificate decision is correctly recorded, which is the
content of the task. There is no test shape that could, so no test was written.

**Owed after the merge and not blocking it:** `scripts/guards.sh --touched-by 3e52ab85`. Among files
a record names, this branch changes `tests/installer.rs` and `.github/workflows/release.yml`.

## What does not close

**Criterion 1 does not close, and it is no longer blocked on a decision.** Its first clause needs a
valid Authenticode signature with a timestamp countersignature on the published installer and the
executable inside it, verified against the published asset. That is 07-08's, and it waits on one
thing: the Azure Artifact Signing account existing. **That is a different state from where this plan
started**, where the certificate was an open question with four answers.

**What closes here** is the part of "a build reaches a user" that is about this repository rather than
about a certificate authority: the publishing path no longer hides a missing asset. And the decision
is on the record with the three options that lost, so the comparison does not get rebuilt the first
time the recurring cost is questioned.

**Nothing is signed and nothing pretends to be.** No certificate exists, no signing machinery was
written, and the narrower grep the corrected criterion asks for confirms every page still says so:

```
$ grep -rniE 'installer is( not)? signed|not yet code signed|build is unsigned' docs/ README.md
docs/ALPHA_TESTING.md:342:- **The installer is not signed**, so Windows will warn about it.
docs/BETA_RELEASE.md:9:Every alpha and beta build is unsigned, so everyone who downloads one meets
docs/installing.md:8:Wixen Mail is not yet code signed, so Windows does not recognise it.
```

**SHIP-02 does not close**, and 07-09 owns it. This plan wrote the standard 07-09 is measured
against and did not read 07-09.

## Self-Check: PASSED

Files claimed, checked on disk:

```
FOUND: .github/workflows/release.yml
FOUND: tests/installer.rs
FOUND: guards/guards.toml
FOUND: scripts/check.sh
FOUND: scripts/check.test.sh
FOUND: .planning/REQUIREMENTS.md
FOUND: .planning/ROADMAP.md
FOUND: .planning/WINDOWS.md
```

Commits claimed, checked in `git log`:

```
FOUND: 9694cd26  FOUND: 7cdf5fb5  FOUND: 91fe4c9b  FOUND: a19234cc  FOUND: 3271db96
```

Counts claimed, taken rather than carried forward: `guards/guards.toml` holds **731** records by
`grep -c '^\[\[guard\]\]'`, and the census reads 192 and 539, which sum to it.
`.planning/WINDOWS.md` frontmatter reads `total_count: 329` and `open_count: 307`, and the markdown
table holds 329 numbered rows.
