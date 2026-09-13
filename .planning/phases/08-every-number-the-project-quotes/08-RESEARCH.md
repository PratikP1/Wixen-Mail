# Phase 8: Every number the project quotes - Research

**Researched:** 2026-09-12
**Domain:** In-repo measurement and provenance. Nothing here came from a web search, no external
package is added by this phase, and no library recommendation is made.
**Confidence:** HIGH for every figure in the tables below; each carries the command that produced
it and the commit it was taken at. MEDIUM for the two long-job cost models, which multiply a
measured rate by a measured count and say so. LOW for nothing: where I could not settle a
question it is in "Open questions" rather than stated.

---

## How this document was taken, and how to read it

This phase is about numbers that go stale, so this document is one of the artefacts the phase has
to judge. Four rules were applied to it and you should hold it to them.

1. **Every figure carries the date it was taken and the command that took it.** No figure is
   given without both.
2. **No figure is restated in two places.** Each measured value appears in exactly one table row.
   Everything else refers to it by name. If you find the same number written twice in this file,
   that is a defect in this file.
3. **Nothing was quoted from another document.** Every number was taken again from the thing that
   decides it. Where a document's figure is reported, it is reported as *what that document
   claims*, in a column beside what the source says today.
4. **What was measured is separated from what was inferred.** Inferences are labelled.

**Conditions on everything below.** Taken on `main` at `a0b909e8`, 2026-09-12, working tree clean
(`git status --porcelain` empty), version `0.119.0` from `Cargo.toml`. The machine is the same
Windows 11 box the rest of the project's figures come from, 24 logical cores. Nothing was built
and no `cargo` command that compiles was run, because a build contends with the gate and because
several figures were available without one. Commands that need a build are named rather than run.

**Line numbers are given with the commit.** `.planning/REQUIREMENTS.md` cites
`docs/IMPLEMENTATION_STATUS.md:123` for a line that is at 164 today, and `wx_app.rs:9125` for a
constant at 9778. Those citations were correct on 2026-09-04 and are eight days old. A line
number without a commit is not a citation.

---

## What this supersedes

`.planning/phases/08-every-number-the-project-quotes/08-RESEARCH.md` dated 2026-09-06, taken from
`main` at `48ace28`. That document is good and this one keeps its best work rather than redoing
it: **its four-kind taxonomy is carried forward unchanged and attributed, and its list of
existing machinery to extend rather than rebuild is still right.**

What has changed in six days is the thing the phase is about. Between `48ace28` and `a0b909e8`
the repository took **431** commits (`git rev-list --count 48ace28..a0b909e8`, 2026-09-12), and:

> **This document got that number wrong on its first pass**, giving 197, which was a guess
> written beside a command nobody had run. It was caught by running the command that was already
> written next to it, before the file was reviewed. Recorded here rather than tidied away,
> because it is the exact defect this phase exists to end and it took four sentences of
> discipline to catch: **writing the command down is not the same as running it.** Phase 8 should
> assume every figure handed to it, including the ones in this file, is a guess until the command
> beside it has been run.

- Six of the figures that document quotes have moved.
- Two of the six questions it could not settle are now settled, one of them by a commit that
  landed four days later and one by work done in this session.
- A third figure for the guard sweep's cost has appeared in the tree, so there are now four.
- A class of counting error has appeared in `guards/guards.toml` that did not exist then.

That is the phase's own argument arriving on schedule. It is not a criticism of the earlier
reading; it is the reason the phase exists.

---

## Summary

**The project has already done phase 8 once, by hand, for five figures, and it worked.** Commit
`20c42110` of 2026-09-10 re-measured five numbers at a named commit, wrote down what each had
been and what it is, and said which two mattered and why. That commit is the model for this
whole phase. It is also the proof that doing it by hand does not stick: two of the five figures
it published were taken again in this session and one of them is wrong.

**The single highest-value thing this phase can build is not a set of numbers. It is a small
number of checks that compute, from the repository, the figures the documents currently assert
by hand.** The precedents are already in the tree, with companions, and they work. See "Which
numbers a check can hold instead of a person".

**The most alarming finding is that the counting method this project prescribes is itself
wrong.** `CLAUDE.md` records at length that counting guard records by grepping a file name
overcounts, and prescribes an awk snippet as the corrected method. That snippet silently
undercounts, because `guards/guards.toml` now contains four records written in TOML's inline
table form, which the snippet's line-oriented reader skips. The same question, asked of the same
commit, gives three different answers depending on who counts. Detail in the next section.

**The two long jobs are both larger than the roadmap allows, and only one of them has a rate
anybody has measured.** The guard sweep is about nineteen and a half hours at the rate the tree
itself records. The whole-tree mutation run has no current rate at all: the only one recorded
predates a change that halved the suite it depends on.

**Most of the measurements this phase needs already exist in the repository, dated and
conditioned, in commit messages.** Nobody has aggregated them. A merge-hash commit in this
project records the gate duration it paid for, and several record test counts and per-record
costs. Harvesting them costs one `git log` pass and produces a dated series rather than a single
figure. See "Numbers the tree already holds".

**Primary recommendation:** open the phase with the cheap counting tasks that resize everything
else, settle the four decisions at the end of this document before any long job starts, and
spend the bulk of the phase writing checks rather than taking measurements, because the
measurements will be stale before the phase closes and the checks will not.

---

## Phase Requirements

| ID | What it asks | Which findings here bear on it |
|----|--------------|-------------------------------|
| PERF-01 | Memory under 150 MB with 1,000 cached messages, measured | Table C row 1; "Requirement by requirement"; no instrument exists (Table E) |
| PERF-02 | Cold start under 2 seconds, measured | Table C row 2; the scan harness is the nearest existing instrument |
| PERF-03 | A mailbox of 100,000+ exercised; sort, filter, scroll numbers; a test that the virtual callback issues no query | Table A rows 12 to 14; "Requirement by requirement"; the filter path problem carried forward |
| PERF-04 | Idle memory under 100 MB, measured | Table C row 3; same missing instrument as PERF-01 |
| PERF-05 | Line coverage re-measured | Table A row 11; `cargo-llvm-cov` is installed |
| PERF-06 | Every document that quotes a test count quotes the same measurement; durations carry conditions | The whole of the inventory, the taxonomy, and "Which numbers a check can hold" |
| PERF-07 | One whole-tree mutation run with a real result | Table A rows 15 and 16; the rate problem; `cargo-mutants` is installed |

There is no `08-CONTEXT.md` in this phase directory (`ls .planning/phases/08-every-number-the-project-quotes/`, 2026-09-12), so there are no locked decisions from a discussion to honour. The
roadmap's six success criteria and the seven PERF requirements are the whole of the constraint.

---

## The taxonomy criterion 3 needs

**Carried forward from the 2026-09-06 reading, which got this right and should not be redone.**
Criterion 3 says every count carries its command and its date, and then says nothing may assert
that a written number equals what a tool reports today. Those two sentences are compatible only
if "count" is split first. Four kinds of number live in this project's documents and only the
first wants a command and a date.

| Kind | What it needs | Can a check assert it? |
|---|---|---|
| **A measurement of the tree** | The command, the date, and any setting that changes it | Only that the command and date are **present**, never that the number is current |
| **A target** | To be marked as a target, with where it came from | That it is not written as though it were a measurement |
| **A constant the code holds** | To agree with the code | **Yes, on every commit.** Both halves are in the repository |
| **A historical record** | To stay exactly as written | **Yes, that it is not silently updated.** Correcting it would falsify it |

The dividing line is **whether the thing counted is in the repository**. That is the load-bearing
sentence of the whole phase and it is not mine.

**One addition this session.** There is a fifth kind, and it is the one that has caused the most
damage here: **a ratio written as two absolute numbers**. "Red/green started at commit 182 of
344" is not a measurement, a target, a constant or a record. Both numbers stay true forever while
the conclusion the reader draws from them inverts. Table D row 4 has the case. A ratio needs the
ratio written out, dated, or it needs to be a check.

---

## The headline finding: the prescribed counting command is wrong

**What the project says.** `CLAUDE.md` (the section "Count records, not mentions, and do not use
a ratio to convert between them") records that ten plans were misled by grepping a file name in
`guards/guards.toml`, that the grep-to-truth ratio is neither two nor stable, and prescribes an
awk snippet as the method. The research brief for this phase repeats that instruction as
authoritative.

**What is true.** The snippet is a line-oriented reader. It sets a flag on a line beginning
`tests_last_seen`, clears it on a line that is a bare `]`, and counts `file =` lines in between.
TOML also permits the block written inline on one line, and `guards/guards.toml` now holds four
of those, at lines 16533, 16566, 16616 and 16651 (`grep -n 'tests_last_seen = \[{'
guards/guards.toml`, 2026-09-12). On an inline record the flag is set, the line is consumed by
`next`, its entry is never counted, and the flag then stays set until some later bare `]`.

**Measured, this session, at four commits.** The quantity is "records whose `tests_last_seen`
names exactly one file", which `CLAUDE.md` and `tests/house_style.rs` both publish.

| Tree | Records | Published figure | Truth, by `tomllib` | The prescribed awk | Inline records present |
|---|---|---|---|---|---|
| `9d6543a`, 2026-09-02 | 548 | 471 (`tests/house_style.rs:5471`) | **439** | 439 | 0 |
| `485030f`, 2026-09-06 | 617 | 477 (`CLAUDE.md:427`) | **477** | not taken | 0 |
| `eda2719`, 2026-09-10 | 683 | 510 (`tests/house_style.rs:5467`) | **512** | 508 | 4 |
| `a0b909e8`, 2026-09-12 | 734 | none | **537** | 533 | 4 |

Command for the truth column, run against each commit's file:

```bash
python -c "
import tomllib,sys
g=tomllib.load(open(sys.argv[1],'rb'))['guard']
print(sum(1 for r in g if len(r.get('tests_last_seen',[]))==1))" guards/guards.toml
```

Three alternative readings of "names one file" were tried against all four trees (one entry in
the block; one distinct file in the block; one distinct file counting the record's own `file`
key). None reproduces 471 and none reproduces 510. The 477 at `485030f` reproduces exactly under
the first two.

**What this means for the phase.** This quantity has been published three times and been wrong
twice, at a total error of 32 and 2. It is computable in milliseconds from a file in the
repository. `tests/house_style.rs` already parses that file with `toml::from_str` in the adjacent
test, so the parser is in hand. **This is the cleanest worked example in the repository of a
number that should be a check rather than a sentence, and phase 8 should make it one and delete
the sentences.**

**And the general rule it argues for:** where the thing counted is a structured file, the
counting command must be the format's parser, not a line reader. A line reader for a structured
format is right for the spelling the file happens to use today and silently wrong for the others
the format allows, and it returns a plausible number rather than an error. That the wrong method
arrived as the correction to a known miscount is the part worth remembering.

---

## The inventory

Every number the project quotes, by kind. Each row gives where the claim lives, what it claims,
what the source says today, and what would hold it.

### Table A: measurements of the tree

| # | Where the claim lives | What it claims | Source today, 2026-09-12 at `a0b909e8` | Command that takes it | Holdable by a check? |
|---|---|---|---|---|---|
| 1 | `guards/guards.toml:79-80` census | 192 swept + 542 since | 734 records; the two add up | `grep -c '^\[\[guard\]\]' guards/guards.toml` | **Already held**, by `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it` |
| 2 | `guards/guards.toml:75` prose | "548 records" | 734 | as row 1 | **No, and must not be.** Historical record of 2026-09-02, see Table D |
| 3 | `CLAUDE.md:427` and `:518` | 617 records, 2026-09-06 at `485030f` | 734 | as row 1 | Yes, if the sentence becomes a reference to row 1 |
| 4 | `CLAUDE.md:539` | "the 536 that existed then" | 734 | as row 1 | Historical, but undated in the text |
| 5 | `.planning/REQUIREMENTS.md:1958` | 565 records, 2026-09-04 | 734 | as row 1 | Out of scope if `.planning` is excluded; see Decision 2 |
| 6 | `.planning/STATE.md:583` | 720 records, census 192 and 528 | 734, census 192 and 542 | as row 1 | Historical section of STATE |
| 7 | `.planning/PROJECT.md:133` | 501 records | 734 | as row 1 | See Decision 2 |
| 8 | `tests/house_style.rs:5428,5467,5471` | 683 records; 510 of them name one file; 77 name `contacts_sync.rs` | 734; 537; 77 | doc-comment figures, see headline section | **Yes**, and the parser is already in that file |
| 9 | `docs/IMPLEMENTATION_STATUS.md:164` | 5,430 tests, 5,269 unit and 161 integration, counted 2026-08-29 | 7,077 tests over `--all-targets`, recorded in commit `b8857bc8` on 2026-09-12; 7,544 test attribute lines in source | `cargo test --all-targets -- --list`; source proxy `grep -rcE '^\s*#\[(test\|tokio::test)\]\s*$' src/ tests/` | **Only that the command and the date are present.** PERF-06's fifth clause forbids asserting the value |
| 10 | `.planning/PROJECT.md:133` | 259,723 lines under `src/`; `caldav.rs` at 8,149; "over 38,000 lines across five `*_sync.rs` files" | 352,680 lines across 280 files; `caldav.rs` 8,337; **eight** `*_sync.rs` files totalling 36,328 | `find src -name '*.rs' \| xargs wc -l \| tail -1`; `find src -name '*_sync.rs' \| xargs wc -l` | Yes for all three |
| 11 | `docs/IMPLEMENTATION_STATUS.md:242` | line coverage 60.4%, measured 2026-07-26 | not re-measured; 1,679 of the repository's 1,911 commits postdate the reading | `cargo llvm-cov --lib --summary-only`; `git rev-list --count --since=2026-07-26 HEAD` | Command and date only |
| 12 | `.planning/REQUIREMENTS.md` PERF-03 evidence | `SAMPLE_MAILBOX_SIZE` 200,000 at `wx_app.rs:9125`; callback at `1101`; comment at `1093` | constant at `wx_app.rs:9778`; callback at `1205`; comment at `1196-1198`. Value 200,000 unchanged | `grep -n 'SAMPLE_MAILBOX_SIZE' src/presentation/wx_app.rs` | Line numbers: no. The constant's value: yes |
| 13 | `wx_app.rs:1196-1198` comment | the callback "reads what is already in memory and never touches the database" | true by reading; the closure captures `state` and `column_layout` and no cache handle | read the closure | **Yes**, and PERF-03 requires it |
| 14 | nothing | sort, filter and scroll timings over 200,000 rows | **nothing in the tree records one** | none exists | No. These are measurements to take |
| 15 | `docs/IMPLEMENTATION_STATUS.md:197` and `CLAUDE.md:366` | a whole-tree mutation run is "about two days" | undated, no conditions, and the rate behind it predates the 2026-09-09 suite change | `cargo mutants --list` for the count | No. Conditions can be required |
| 16 | `docs/plans/20260801-mutation-sweep.md:12-15` | 4,171 mutants; one every 26 seconds; "near thirty hours" | 2026-08-01 figures. The rate's denominator has since roughly halved and the mutable source has grown | as row 15 | Historical record |
| 17 | `.cargo/mutants.toml:37` | "the suite takes about 46 seconds, measured 2026-08-06", and `timeout_multiplier` and `minimum_test_timeout` are read against it | library runs recorded on 2026-09-11 in commits `c5c43c6d`, `c58bfc1a`, `7ad70ffe` are 63.86 s, 69.99 s and 57.65 s at eight threads; the `--all-targets` term is larger again | `cargo test --all-targets` prints it | The value, no. That the calibration was re-taken, yes, as a dated note |
| 18 | `tests/house_style.rs` self-reference in `.planning/writing-checks-widened.md:26` | "it stays at 64 [test functions] and none of the 649 guard records needs re-measuring" | 69 test functions; 734 records | `grep -cE '^\s*#\[(test\|tokio::test)\]\s*$' tests/house_style.rs` | See Decision 2 |

### Table B: constants the code holds, quoted in prose

All of these agree with the code today. None is held by a check. Both halves are in the
repository, so all of them are holdable, and the taxonomy says this is the kind that a check can
assert outright.

| Claim in prose | Where | Constant | Where |
|---|---|---|---|
| a single file kept up to 25 MB | `docs/privacy.md:66` | `LARGEST_ATTACHMENT_KEPT_BYTES` | `src/data/message_cache/attachment_content.rs:51` |
| all of them together up to 512 MB | `docs/privacy.md:67` | `ATTACHMENT_CACHE_BUDGET_BYTES` | `src/data/message_cache/attachment_content.rs:64` |
| a signed message larger than 25 MB is dropped | `docs/privacy.md:91` | `LARGEST_SIGNED_MESSAGE_KEPT_BYTES` | `src/data/message_cache/signed_original.rs:63` |
| these copies pass 128 MB | `docs/privacy.md:92` and `:94` | `SIGNED_ORIGINAL_BUDGET_BYTES` | `src/data/message_cache/signed_original.rs:75` |
| a message larger than 25 MB is not kept at all | `docs/privacy.md:114` | `LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES` | `src/data/message_cache/moves_in_flight.rs:64` |
| interrupted moves adding to more than 64 MB | `docs/privacy.md:115` | `MOVES_IN_FLIGHT_BUDGET_BYTES` | `src/data/message_cache/moves_in_flight.rs:77` |
| PNG, JPEG, GIF and WebP up to 2 MB | `docs/KEYBOARD_SHORTCUTS.md:843` | `MOST_ONE_PICTURE_MAY_BE` | `src/application/pictures.rs:116` |
| 20 MB zip, 5 MB per file, 50 MB total, 2 seconds | `docs/plans/20260823-earcon-sound-schemes.md:196-214` | `MAX_ZIP_BYTES`, `MAX_FILE_BYTES`, `MAX_TOTAL_BYTES`, `MAX_SOUND_DURATION` | `src/presentation/accessibility/sound_scheme_import.rs:27-41` |
| an attachment size warning over 10MB | `docs/roadmap.md:138` | `LIMIT_BYTES` is 25 MB | `src/application/attaching.rs:27` |

**The last row does not agree**, and it is the only one that does not. `docs/roadmap.md:138`
ticks "Attachment size warnings (>10MB)" while the shipped limit is 25 MB. Whether that is a
stale document or a different feature is for the planner to establish, not me; I record that the
two numbers differ.

**One number in this class has no constant behind it.** `docs/privacy.md:417` says the update
download "is roughly 12 MB today". "Today" is not a date, the size of a release artefact is not
in the repository, and nothing can check it. It belongs in the ledger or it needs a date.

### Table C: targets

Each of these is a target with no measurement attached. PERF-01, PERF-02 and PERF-04 exist to
attach one, and criterion 6 says each is then met or revised with the reason.

| # | Target | Where | What exists today |
|---|---|---|---|
| 1 | under 150 MB with 1,000 cached messages | `docs/development/requirements-backlog.md:81` | no instrument at all; see Table E |
| 2 | cold start under 2 seconds | `docs/roadmap.md:221` and `:253`, `docs/development/requirements-backlog.md:82` | no instrument; the accessibility workflow starts the release binary per window and is the nearest thing |
| 3 | low memory footprint, under 100 MB idle | `docs/roadmap.md:254` | as row 1 |
| 4 | high code coverage, target 80%+ | `docs/architecture.md:400` | last reading 60.4%, Table A row 11. Two different targets in the tree: this and the 95% at `docs/integration-guide.md:6`, which that page already describes as never met and keeps as a historical page |
| 5 | 100% keyboard accessible | `docs/roadmap.md:255` | not a number this phase can measure; belongs to the accessibility work |

### Table D: historical records, which must not be corrected

Correcting any of these would falsify a statement about a past tree. `tests/house_style.rs:3042`
already states the principle for the version guard. The check to write, if any, is that they are
**not silently updated**, not that they match today.

| # | The record | Where | Why it must stay |
|---|---|---|---|
| 1 | "548 records ... written on 2026-09-02 by `--recount-everything`" | `guards/guards.toml:74-75` | The file really held 548 records on 2026-09-02; confirmed this session at `9d6543a` |
| 2 | every dated row of the mutation progress table | `docs/plans/20260801-mutation-sweep.md:413-420` | Each row is a run that happened on the date beside it |
| 3 | "3,362 from 2026-08-09 is what a number without its command and its date turns into" | `docs/IMPLEMENTATION_STATUS.md:165-166` | The example is the point |
| 4 | "red/green started at commit 182 of 344" | `CLAUDE.md:349`, `docs/IMPLEMENTATION_STATUS.md:191`, `.cargo/mutants.toml:8`, `01-VALIDATION.md:45`, `02.1-VALIDATION.md:54` | **This one is different and needs a decision.** See below |

**Row 4 is the fossilised ratio, and it is the worst number in the repository.** Both figures
are true statements about the past: red/green did start at commit 182, and the repository did
hold 344 commits when that was written. The repository holds 1,911 today
(`git rev-list --count HEAD`, 2026-09-12, first commit `ca131605` on 2026-02-13). So the same
sentence that once said "53% of this project's history predates the practice" now says 9.5%, and
the conclusion it was written to support, that most tests here describe the code rather than
specify it, may now be false. It is quoted in five places including the configuration for the
mutation tool, and it is the stated justification for PERF-07. Nothing failed and nothing could
have: neither number moved.

I am not claiming the conclusion **is** false. Commits are not tests, and the fraction of tests
written before their code is a different quantity that nobody has measured. I am claiming the
number as written no longer supports what it is cited for, and that a phase about numbers cannot
leave it alone.

---

## Numbers the tree already holds, dated, that nobody has aggregated

**This is the cheapest useful thing in this document.** This project records, in the body of its
merge-hash commits, the gate duration each branch paid for, and in several commits the test count
and per-record costs beside it. That is a dated, conditioned time series sitting in `git log`,
and no document reads it.

One pass over the last 800 commits, 2026-09-12:

```bash
git log --format='@@@%ad %h%n%B' --date=short -800 | awk '
  /^@@@/ {hdr=substr($0,4); next}
  /[0-9][0-9]+ seconds/ { if (match($0, /[0-9]+ seconds/)) print hdr "  " substr($0, RSTART, RLENGTH) }'
```

What it yields for the full gate, `scripts/check.sh all`, all on 2026-09-12 and all four checks
green: 419, 463, 465, 471, 473, 506 and 654 seconds. Commit `44615216` had already noticed the
band and wrote it down as "419 to 654 the six landed branches report". `a0b909e8`, this
session's HEAD, records two runs of 473 and 465.

Three things follow.

**The gate's own recorded figure is out of date.** `CLAUDE.md:280` gives 311 seconds for the
whole gate, measured warm 2026-08-30. The band today is 1.3x to 2.1x that. Nobody re-measured it,
and nobody had to, because every branch was recording the real number in its own commit message.

**A single figure for the gate would be a wrong answer.** The spread within one day is 1.56x.
`CLAUDE.md` already warns about this for the documents-only path (36 seconds once, 2m56s another
time) and the warning applies to the full gate too. Whatever the phase publishes for gate cost
must be a band with a date and a note about what moves it, not a number.

**The per-record guard cost is settled and decomposed, and one sentence in `CLAUDE.md` has not
caught up.** Commit `20c42110`, 2026-09-10, measured at `eda2719`:

> 112 seconds a record -> 95, being 29s rebuild and 66s library run at 8 threads

`CLAUDE.md:527-529` says "The rebuild term was never measured beside the run term". It was, one
day after that paragraph was written. The 2026-09-06 research listed "the real per-record guard
cost on today's tree" as a question it could not settle; it is settled, at 95 seconds, with both
components named.

The same commit records the library run and shell-suite figures, and commits `c5c43c6d`,
`c58bfc1a` and `7ad70ffe` of 2026-09-11 record library-run wall times of 63.86 s, 69.99 s and
57.65 s over 6,905 to 6,993 tests at eight threads.

**Recommendation for the planner:** make harvesting this series a task, and make the harvested
series the thing the documents point at. A document that says "the gate costs between X and Y,
from the N branches that recorded it, most recent Z" is the only shape of durational claim that
does not perish, because the next branch extends it automatically.

---

## Which numbers a check can hold instead of a person

**The most valuable section, per the brief.** The repository does this well already and the
precedents are better than anything I would design.

### What already works, and the shape to copy

| Check | Where | What it holds | Companion that proves it can fail |
|---|---|---|---|
| `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it` | `tests/house_style.rs` | two numbers in a file header add to the records in that file | `test_the_sweep_header_check_can_tell_the_two_apart` |
| `test_no_changelog_list_is_introduced_by_a_count_that_disagrees_with_it` | `tests/house_style.rs` | a count stated in prose agrees with the bullets it introduces | `test_the_count_check_can_see_a_count_that_disagrees` |
| `test_every_guard_record_says_how_many_tests_the_files_it_names_held` | `tests/house_style.rs` | recorded per-file test counts against the tree, and prints the scoped remedy | `test_the_recorded_count_check_can_tell_a_drift_from_an_agreement`, plus two more |
| `test_every_test_a_guard_record_names_is_a_test_that_exists` | `tests/house_style.rs` | a renamed test cannot leave a record unmeasurable | its own siblings |
| six readings in `tests/the_planning_files_agree_with_themselves.rs` | that file | STATE frontmatter against its prose, plan and summary counts against disk, both halves of the ledger, the roadmap against disk | one companion per reading, six of them, each splicing a disagreement into the real file's own lines |
| `test_no_status_page_names_a_version_the_code_does_not_ship` | `tests/house_style.rs` | no page names a version the code does not ship | `test_the_version_reading_can_see_one_on_a_real_page`, which splices a version nobody ships into each page's real text |

**The rule every one of these follows: a reading guard ships with a companion that proves the
reading can see a violation.** `CLAUDE.md` explains why (a guard whose trigger is "a document
mentions X" is disarmed by the workaround it recommends) and the version guard is the live
example: neither page it reads names a version today, so it iterates over nothing, and only the
companion keeps it honest. Every check this phase writes needs one. **Budget for that: the
companion is most of the work.**

### What phase 8 could add, ranked by value against cost

| Candidate check | Holds | Cost | Why it is worth it |
|---|---|---|---|
| **Guard-record shape figures computed, not written** | the record count, how many name one file, how many name each hot file | low; `tests/house_style.rs` already parses the file with `toml::from_str` | The headline finding. Three publications, two wrong. Delete the sentences and have the failure message print the figures |
| **Prose figures in `docs/privacy.md` against their constants** | Table B, nine rows | medium; needs a mapping from a sentence to a constant | Both halves are in the repository, which the taxonomy says is exactly the checkable kind. Today nothing reads a single one |
| **A measurement carries a date and a command** | the *presence* of provenance beside a number, never its value | medium to high; needs the taxonomy encoded so a target, a constant and a record are not flagged | This is criterion 3 itself. Scope it to the four product pages first, not to the changelog |
| **A duration carries its conditions** | that a figure in seconds, minutes or hours is accompanied by a date and a machine or setting | medium | PERF-06's fourth clause, added 2026-09-04, asks for exactly this |
| **A stated ratio of two absolutes is banned or dated** | Table D row 4's failure mode | low | One pattern, five sites, and the class has no other detector |
| **The `Cargo.toml` version against pages that name one** | already exists and is vacuous | zero; it is written | Not a new check. Worth noting that it stays vacuous until some page names a version |

**What must not be checked:** any assertion that a written number equals what a tool reports
today. PERF-06's fifth clause records that this is what the requirement used to ask for and that
it was corrected on 2026-08-29 because it is false the next time anybody adds a test. The phase
must not reintroduce it.

---

## Numbers that must be quoted with their conditions

Each of these is a figure whose value depends on something the figure does not name. PERF-06's
fourth clause makes carrying the conditions a requirement.

| Figure | Conditions that change it | Evidence |
|---|---|---|
| Full gate duration | whether the previous commits forced a clippy rebuild; branch or `main`; what else is running | the 419 to 654 band on one day, above |
| Documents-only gate duration | the same, plus the 108-second shell-suite floor | `CLAUDE.md:253` gives 36 seconds and `:268` gives 2m56s for the same path |
| Library suite duration | `WIXEN_TEST_THREADS`; whether the schema template is in play | the 2026-09-09 and 2026-09-11 series |
| Per-guard-record cost | thread count; that it decomposes into a rebuild and a run, which moved for different reasons | `20c42110` |
| Any whole-sweep or whole-run total | it is a rate times a count and **both terms move**, sometimes in opposite directions | see the cost models below |
| Mutation timeout behaviour | calibrated against a suite figure of 2026-08-06 that has since grown several times over | `.cargo/mutants.toml:37-48` |
| `target/` disk cost | measured 258.5 GB total and 246.1 GB in `target/debug` over 229,412 files, 2026-09-12, with `Get-ChildItem -Recurse -File \| Measure-Object -Property Length -Sum` | relevant because both long jobs rebuild repeatedly |

---

## Cost models for the two long jobs

Both are a rate times a count. **Take the count today and measure the rate today** is the
project's own instruction and it is the right one; what follows is the best available starting
point, labelled.

### The guard sweep, roadmap criterion 5

- **Count, measured 2026-09-12:** 734 records.
- **Rate, measured 2026-09-10 at `eda2719` and recorded in commit `20c42110`:** 95 seconds a
  record, being 29 s rebuild plus 66 s library run at eight threads.
- **Product:** 69,730 seconds, **19 h 22 m**, plus triage.

That is an inference from two measurements, not a measurement. It is also the fourth figure the
tree gives for this job, and the phase should retire the other three rather than adding a fifth:

| Where | Figure | When it was right |
|---|---|---|
| `.planning/ROADMAP.md`, criterion 5 | "roughly 15 hours" | at a smaller count and the pre-halving rate |
| `CLAUDE.md:520` | "about 20 hours", then superseded two paragraphs later without a replacement | 2026-09-06, at 617 records and 119 s |
| `tests/house_style.rs:5429` | "eighteen hours" | 2026-09-10, at 683 records and 95 s |
| `guards/guards.toml:40` | "a full run is an hour or two" | a fossil from about 192 records; out by an order of magnitude |

**Three practical notes for the planner.** The job is unattended and restores each file after
each break, but it needs a quiet machine: `scripts/guards.sh:41` records that a commit hook
running the suite during one reported three guards green that go red on their own. A plain sweep
has no resume and writes nothing back, so its only artefact is the log. And `--remeasure` writes
counts back for records that agreed, which a plain sweep deliberately does not, so the two are
not interchangeable.

### The whole-tree mutation run, roadmap criterion 4 and PERF-07

- **Count:** unknown. `cargo mutants --list` parses rather than builds and answers in seconds.
  `cargo-mutants 27.1.0` is installed. **This should be the first task of the phase**, because
  everything about criterion 4's schedule depends on it.
- **Rate:** no current figure exists. The only measured rate, one mutant every 26 seconds, is
  from 2026-08-01 and its denominator is a suite run that has since roughly halved. The "about
  two days" in two documents is undated and has no conditions at all.
- **Product:** not given here, deliberately. Multiplying a 2026-08-01 rate by a count nobody has
  taken would be exactly the mistake this phase exists to end.

**One risk the planner must carry.** `.cargo/mutants.toml` sets `timeout_multiplier = 5.0` and
`minimum_test_timeout = 60`, and its own comment says both are read against a suite figure of
about 46 seconds measured 2026-08-06. The suite is several times that now. The 2026-08-05
whole-tree attempt already reported 81 spurious timeouts from a busy machine, and its comment
warns against lowering the multiplier to hide them. Re-deriving that calibration before the run
is cheaper than discovering it at hour forty.

---

## What cannot be measured on this machine at all

Each of these belongs in `.planning/WINDOWS.md` rather than in a plan. Verified this session
unless stated.

| What | Evidence |
|---|---|
| **Anything against a real mail account** | recorded across the ledger and the roadmap; no account has ever been used |
| **PERF-03's "a real mailbox of 100,000 messages"** | the requirement itself already says synthetic rows answer the list question and the provider question waits for a live account |
| **A OneNote tenant, a CalDAV or CardDAV server, Google or Microsoft Graph** | `.planning/STATE.md` records every request being answered by a loopback server the tests start |
| **A Linux or macOS build** | `.github/workflows/other-platforms.yml` is `workflow_dispatch` only and its header says nobody has ever compiled this crate off Windows |
| **A published release, or any signed artefact** | phase 7's summaries record that nothing is signed because there is no certificate, and that criterion 1 waits on an Azure account only Pratik can create |
| **The update download's real size** | `docs/privacy.md:417`'s "roughly 12 MB today" is about an artefact no release has produced |
| **Screen reader confirmation of anything** | the project's second guardrail; 320 of the 342 ledger entries are open and many are `unrun-verify` |

### One more that the brief did not anticipate

**CI has been red, and the local gate cannot see it.** Measured 2026-09-12 with `gh run list`:
the last three CI runs on `main`, all on 2026-09-10 at commit `1aba3a5b`, failed. Run
`34467657848` shows Clippy, Test Suite and Security Audit failing and four jobs passing. The
Clippy failure is two instances of "using `chunks_exact` with a constant chunk size", a lint the
local toolchain does not have: local is `rustc 1.97.1` and `clippy 0.1.97` of 2026-07-14, the
workflows use `dtolnay/rust-toolchain@stable` resolved at run time, and the repository pins no
toolchain (no `rust-toolchain.toml`, no `rust-toolchain`). Meanwhile `git rev-list --count
origin/main..HEAD` is **180**.

Three consequences for this phase.

1. `CLAUDE.md` says the local script runs "the same four checks CI runs". CI runs seven jobs.
   One of them, `cargo audit`, has no local counterpart at all and is among the failures. That is
   a documented claim that is not true, in the file whose whole argument is that documented
   claims need checks.
2. Every long job this phase schedules will run against a tree that CI has never judged.
3. `.cargo/audit.toml` accepts seven advisories, each with a written exit condition and none with
   a date: `RUSTSEC-2026-0194`, `-0195`, `-0098`, `-0099`, `-0104`, `RUSTSEC-2024-0436`,
   `RUSTSEC-2025-0134`. An exit condition nobody re-reads is the same kind of thing as an
   undated measurement, and the audit job failing means an eighth advisory has arrived that
   nobody has read.

I have not fixed any of this and it is not obviously phase 8's work. It is recorded because the
phase cannot honestly publish "the gate is green" without it.

---

## Requirement by requirement

### PERF-01, memory under 150 MB with 1,000 cached messages

**Evidence today:** no instrument exists. Verified 2026-09-12: no `benches/` directory, no
`criterion`, `divan` or `[[bench]]` in `Cargo.toml`, and `grep -rn 'sysinfo|GetProcessMemoryInfo|
PROCESS_MEMORY_COUNTERS|WorkingSet' src/ Cargo.toml` returns nothing. The 2026-09-04 evidence in
`.planning/REQUIREMENTS.md` is still accurate.

**What would have to be produced:** a repeatable procedure that builds a cache holding 1,000
messages, starts the release binary against it, and reads the working set, recorded with date,
machine and build. The mechanism for the controlled profile already exists and is used by the
accessibility workflow: `WIXEN_MAIL_DATA` (`src/common/paths.rs:30`) redirects the whole data
folder, and `.github/workflows/accessibility.yml:109` sets it to a scratch directory and starts
`target/release/wixen-mail.exe` with `Start-Process`. That is most of the harness already
written. Reading the working set is a PowerShell one-liner over the started process; no new
dependency is needed and none should be added.

**Open question the planner must settle:** what "1,000 cached messages" means concretely, since
the cache holds envelopes, bodies, attachments and signed originals under separate budgets
(Table B). A thousand envelopes and a thousand full bodies with attachments are different
measurements by a wide margin.

### PERF-02, cold start under 2 seconds

**Evidence today:** nothing in `src/` times process start against a usable list. The target is
correctly written as a target in all three places.

**What would have to be produced:** a measurement from process start to the message list being
usable, not to the window appearing. The requirement is explicit about that and it is the part
that makes this hard: "usable" is a state inside the application, so either the application logs
a timestamped line at that moment or the measurement is of something else. The application
already emits `tracing::info!` around the sample mailbox load (`wx_app.rs:5255-5262`), so the
pattern of timestamping a milestone in the log is established. `hyperfine` is **not** installed;
`Measure-Command` is available and adequate.

### PERF-03, exercised at scale

**Evidence today, all verified this session:**

- The 200,000 row generator ships and is reachable from the Help menu, not behind a build flag
  (`SAMPLE_MAILBOX_SIZE` and `ID_LOAD_SCALE_SAMPLE` per Table A row 12).
- No sort, filter or scroll number exists anywhere (Table A row 14).
- The virtual text callback does not touch the database, and no test says so (Table A row 13).

**Carried forward from the 2026-09-06 reading, and re-checked:** the sample mailbox is pushed
straight into the in-memory list through `UIUpdate::MessagesLoaded` and never enters SQLite, and
the mail list's filter is `MessageCache::search_messages`, a SQL query. So the sample mailbox
cannot produce a filter number. That is unchanged and is Decision 3.

**For the no-query test, a precedent exists and is better than inventing one.**
`test_no_query_a_folder_listing_runs_reads_message_text_or_a_table_it_may_not`
(`src/data/message_cache/messages.rs:7973`) builds a database holding only what a listing may
read, prepares every query the listing can run against it, and requires none to be refused. Its
companion `test_a_listing_that_read_the_columns_message_text_used_to_live_in_is_caught` proves
the reading can fail. That is the shape. For the callback the assertion is different in kind,
because the callback runs no query at all: the strongest version is that the closure captures no
connection, which is a type-level fact. Decision 4 is which shape to use, and the earlier
reading's recommendation to do both still looks right to me.

### PERF-04, idle memory under 100 MB

Same missing instrument as PERF-01, same harness, and the same open question in a milder form:
"idle, after startup, with a cache present" needs the cache defined.

### PERF-05, line coverage re-measured

**Evidence today:** `cargo-llvm-cov 0.8.7` is installed. The last reading is Table A row 11.
The requirement's own `[D]` line already says the low areas are to be attributed to the untested
network transport rather than treated as a number to raise, and roadmap criterion 3 repeats it.
This is the cheapest of the seven: one command, one number, one date, one attribution paragraph.

**One caution.** A coverage run recompiles with instrumentation, so it will not share
fingerprints with the ordinary build and will cost a full rebuild on a tree whose `target/` is
already 246 GB in debug artefacts alone.

### PERF-06, every document that quotes a test count quotes the same measurement

This is the requirement the rest of this document serves. Three things are worth stating.

**The three documents that agreed on 2026-08-29 still agree with each other** and are all behind
the tree together, which under the requirement's own third clause is a re-measure rather than a
failure. The sites are `docs/IMPLEMENTATION_STATUS.md:164`, `docs/changelog.md:3519-3523` and
`docs/integration-guide.md:5`.

**The split between unit and integration is preserved in all three** and the requirement says
keep it rather than add it.

**The structural count and the `--list` count are different quantities and must not be
conflated.** `grep` over test attributes counts what is written in source; `cargo test --list`
counts what compiles for this platform and feature set. Table A row 9 gives both for today and
they differ by several hundred. Whichever the phase publishes, it must say which.

### PERF-07, one whole-tree mutation run with a real result

**Evidence today:** `cargo-mutants 27.1.0` installed; `scripts/mutants.sh` refuses a partial run,
a run whose build failed before anything changed, and a run in which the suite was never once run
against a mutant; `.cargo/mutants.toml` excludes the wxWidgets layer, `main.rs` and vendored code
with the reasoning written down. The count and rate problems are in the cost model above.

**The justification problem.** PERF-07's argument for existing is Table D row 4, the fossilised
ratio. If the phase corrects that number, it should say plainly whether the requirement's
motivation survives the correction. My reading is that it does, for a different reason than the
one given: mutation testing found real dead code and whole families of untested behaviour in the
August sweeps regardless of when the tests were written. But that is a different sentence, and
writing it is part of criterion 6.

---

## Existing machinery to extend rather than build

| What | Where | Use it for |
|---|---|---|
| TOML parsing of `guards.toml` | `tests/house_style.rs`, `test_every_guard_record_says_how_many_tests_the_files_it_names_held` | every guard-shape figure |
| The companion pattern | `test_the_version_reading_can_see_one_on_a_real_page` and the six in `the_planning_files_agree_with_themselves.rs` | every new reading guard |
| `WIXEN_MAIL_DATA` plus `Start-Process` | `src/common/paths.rs:30`, `.github/workflows/accessibility.yml` | the memory and startup harnesses |
| `--scan-target` and `--scan-window` | `src/presentation/command_line.rs` | starting the app in a controlled state |
| `tracing::info!` milestones | `wx_app.rs` sample mailbox handler | timestamping "usable" for cold start |
| A stripped database and `prepare` | `src/data/message_cache/messages.rs:7973` | the no-query assertion |
| Gate durations in commit bodies | `git log` | the durational series |
| `scripts/guards.sh --remeasure` and `--touched-by` | those scripts | scoped record work during the phase |

---

## Don't hand-roll

| Problem | Do not build | Use instead | Why |
|---|---|---|---|
| Counting records in a structured file | a line-oriented reader | the format's parser, already linked into the test target | the headline finding: three answers to one question |
| Proving a reading guard can fail | a hand-written negative case with a literal string | splice the violation into the real file's real lines, as `test_the_version_reading_can_see_one_on_a_real_page` does | six recorded ways a guard goes vacuous, and a literal fixture catches none of them |
| Timing a suite | a stopwatch quoted once | harvest the existing series and extend it | a single figure is wrong within a day at this spread |
| Reading working set | a new dependency | the OS, through the already-started process | this project's dependency bar, and nothing here needs a crate |
| A whole-sweep total | a number | a rate and a count, each dated, with the product shown | both terms move and they have moved in opposite directions |

---

## Common pitfalls

**Do not write a check that compares a document's number to what a tool reports.** Criterion 3
and PERF-06's fifth clause both forbid it, and the requirement records that it was corrected on
2026-08-29 for exactly this reason.

**Do not treat a stale number as a defect.** The three documents behind on the test count are a
re-measure, not a failure.

**Do not correct a historical record.** Table D exists for this. Correcting a dated statement
about a past tree falsifies it.

**A census that asserts a floor weakens the guard beside it.** `CLAUDE.md` records the case: a
constant saying "at least 8 of these exist" stops being load-bearing at 9. If a new check counts
claim sites, re-measure every guard record that reads that census.

**Adding a test to a hot file costs re-measurements.** Measured 2026-09-12 with a TOML parse:
`src/application/contacts_sync.rs` is named by 77 records and `src/application/calendar.rs` by
72; 190 distinct files are named across 734 records carrying 1,899 red-list entries in total. At
95 seconds each, a test added to the first costs about two hours of scoped re-measurement. Where
a new test can honestly live in a smaller file, that is real money.

**Do not pipe `scripts/check.sh` into anything whose exit status you then read.** Recorded in
`CLAUDE.md`; it has already put an unformatted tree onto a commit.

**Nothing else may be building while `scripts/guards.sh` runs.**

**`cargo test` takes one `--lib`.** `CLAUDE.md` records that 55 plans told executors to pass
several and none could ever have run. Any verification command this phase writes must be one
`--lib` per invocation, joined with `&&`.

**Read a partial mutation run as partial.** The script refuses to summarise one now, but the
report is still written as it goes and reading it mid-run has already produced a wrong commit
message.

**A red commit here has three conditions at once** and this phase will produce them: every named
test ran, every named test failed, and nothing else failed. `test_every_guard_record_says_how_many
_tests_the_files_it_names_held` reddening on an added test is the known collision, and
`CLAUDE.md` says the answer is to name the count check among the failures rather than to split
the commit.

---

## What phase 6 could invalidate

**This section was measured twice in one session and the answer changed between them. Both
readings are kept, because that is the finding.**

**First reading, at `a0b909e8`, early in the session.** Phase 6 was not executing.
`.planning/phases/06-how-the-application-speaks/` held eight `*-PLAN.md` files and no
`*-SUMMARY.md`, the roadmap row read `0/8, Planned, not started`, and `.planning/STATE.md`
frontmatter had `current_phase: 7` with `status: executing`. Phase 7 was the one in flight, at
`07-09` with task 3 open. The eight phase 6 plans had been written earlier the same day in commit
`3e52ab85`.

**Second reading, about ninety minutes later, at the end of this session.** Phase 6 had started.
`dcbe5ee4 refactor(06-01): Event and Event::ALL come from one list` had landed on top of
`a0b909e8`, and a further 140-line change to `src/presentation/accessibility/feedback.rs` was
staged in the working tree by that executor. `.planning/STATE.md` still read `current_phase: 7`
and the roadmap row still read `0/8, Planned, not started`, so **the two documents that answer
"which phase is running" were both wrong at the moment I read them the second time**, and would
have been the only evidence available to anybody who did not look at `git log`.

Nothing here was rewritten to hide the first reading. A measurement that was correct when taken
and false ninety minutes later, in a document whose subject is measurements that go stale, is
worth more on the record than a tidy answer. It is also the strongest possible argument for the
central recommendation: the documents should not be asserting this at all when `git log` and the
phase directory can be read.

The exposure to phase 8 is unchanged by which reading is current:

1. **The WCAG coverage claim.** Phase 6's criterion 3 turns "roughly half of WCAG" into a list.
   That phrase occurs **28 times across 14 files** (`grep -rn 'half of WCAG'` over the tree
   excluding `target/`, 2026-09-12), including `CLAUDE.md`, `docs/principles.md`,
   `docs/IMPLEMENTATION_STATUS.md`, `docs/changelog.md`, `.github/workflows/accessibility.yml`,
   three `.planning/intel` and audit files, and four phase plan or research files. Phase 8's
   criterion 3 asks that the documents agree with each other. **If phase 6 corrects some of those
   28 and not others, phase 8 inherits the disagreement.** Whether the merged plan files count as
   documents to correct or as historical records is Decision 2.
2. **The two figures phase 6's research established.** Axe.Windows carries 155 rules across five
   standards, and WCAG 2.2 Level AA conformance is 55 success criteria rather than the commonly
   published 56, both counted 2026-09-12 and both recorded in `06-RESEARCH.md` with the reasoning
   and the earlier wrong answers (144 with a breakdown summing to 155; 173 from one reading; 141
   from another). Phase 8 should treat those as phase 6's to publish and should not re-derive
   them, but must check that whatever phase 6 writes carries its command and its date.
3. **File-level collisions.** Phase 6 will change `feedback.rs`, `config.rs`, `wx_settings.rs`,
   four date sites and the scan configuration. Any figure phase 8 pins about those files, in
   particular guard-record test counts, will need re-measuring after phase 6 merges.

---

## Assumptions log

| # | Claim | Where | Risk if wrong |
|---|---|---|---|
| A1 | The guard sweep is about 19 h 22 m | cost model | It is a rate of 2026-09-10 times a count of 2026-09-12, not a measurement. One record measured on the day settles it and costs two minutes |
| A2 | The mutation run's mutant count is unknown rather than estimable | cost model | I declined to extrapolate; the earlier research did and got 11,000 to 12,000. `cargo mutants --list` settles it in seconds |
| A3 | "Red/green started at commit 182 of 344" no longer supports its conclusion | Table D row 4 | The conclusion is about tests, and I measured commits. It may still be true for a reason the number does not give |
| A4 | The nine prose figures in Table B all agree with their constants | Table B | I matched by reading the sentence and the constant. A mapping written as a check may find a tenth I missed, or find that one of mine pairs the wrong sentence with the right constant |
| A5 | `docs/roadmap.md:138`'s 10MB warning and the shipped 25 MB limit are the same feature | Table B note | If they are two different things, there is no disagreement to fix |
| A6 | The 510 published at `eda2719` was measured rather than derived | headline section | Commit `20c42110` says "re-measured rather than scaled". If it was measured with the file's own reader, the 2-record gap is a bug in that reader and worth finding |

---

## Open questions

1. **Why the published one-file counts are wrong at two of three commits.** Settled: the true
   values are 439, 477 and 512. Not settled: what produced 471 and 510. The inline-record
   mechanism explains a 4-record undercount at `eda2719`, not a 32-record overcount at
   `9d6543a`, and not the 2 that separates 510 from 512.

2. **Whether the 2026-08-05 mutation run was whole-tree.** Carried forward unresolved from the
   2026-09-06 reading; the run's output is gone. It matters only for what shape of failure to
   expect, not for the plan.

3. **How many of the 734 records are stale.** The two prior sweeps disagree by a factor of
   twenty, 23 in 208 against 1 in 220, and which rate applies depends on how consistently the
   per-commit remedy was run through phases 3 to 7. I did not measure that. It is knowable
   cheaply: the remedy prints a command, and whether it was run is visible in the commits that
   followed.

4. **What "1,000 cached messages" and "idle with a cache present" mean concretely.** PERF-01 and
   PERF-04 both need this pinned before a number means anything, and neither requirement says.

5. **Whether the `cargo audit` failure is an eighth advisory or something else.** I read the job
   list, not the job log for that job.

---

## Decisions for Pratik

Listed with options and costs. I have not answered any of them.

**1. When does the guard sweep run, and does it get a resume flag first?**

Roughly nineteen and a half hours, unattended, needs a quiet machine, no resume, writes nothing
back, and its only artefact is the log. The options and their costs are set out in the
2026-09-06 research under the same heading and have not changed except for the total; they are
one overnight run, split by suite, split into named chunks through `--remeasure`, or a resume
flag first. The one new fact is that the total is now nineteen and a half hours rather than
twenty, and that the rate behind it is now a real decomposed measurement rather than an
extrapolation, so the estimate is firmer than it was.

**2. What counts as "the documentation" for criterion 3?**

Four candidate scopes, widening: the four product pages; all of `docs/`; `docs/` plus `CLAUDE.md`
and `README.md`; all of that plus `.planning/`.

The precedent is already written and it argues for excluding `.planning`.
`.planning/writing-checks-widened.md` records that when the writing checks were pointed at
`.planning`, the rules about *what the product claims* produced twelve findings and zero real
ones, because a planning tree is a different genre, and that the exclusion is by genre in one
place with the reasoning in a doc comment. It also names its cost plainly: a planning document
really could promise something false and nothing would say.

The cost of following that precedent here is larger than it was there, because the documents
carrying the most numbers are `.planning/REQUIREMENTS.md`, `.planning/PROJECT.md` and
`.planning/STATE.md`, and three of Table A's rows are in them. The cost of not following it is
that every merged plan and every audit becomes a document the check reads, and most of those are
historical records that must not be corrected.

**3. What answers criterion 2's filter number?**

Unchanged from the 2026-09-06 reading and still open: either write 200,000 rows into a `tempfile`
cache and time `search_messages`, which measures the real filter path and needs no window, or
drop the filter number and record why.

**4. Which shape of check asserts that the virtual callback issues no query?**

The type-level version is strongest and cannot be recorded in `guards/guards.toml`, because its
break is a compile error and the runner measures breaks by which tests go red. The
source-reading version is recordable and is the shape this project has most often watched go
vacuous.

**5. What happens to "red/green started at commit 182 of 344"?**

Five sites. Three options: leave it and date it as a historical statement, which is honest but
leaves the inference wrong in the reader's head; replace it with the ratio as of a date, which
is one sentence and needs re-taking; or make it a check that computes the fraction of history
predating red/green and prints it, which never goes stale and costs a companion. The third is
also the only one that would have caught it.

**6. Does phase 8 own the CI problem?**

CI has been red since 2026-09-10 and `main` is 180 commits ahead of what CI has seen, because the
toolchain is unpinned and the runner's clippy is newer. This is not a number and it is not in the
PERF requirements. It is in this document because a phase whose deliverable is trustworthy
figures cannot publish them from a tree nothing outside this machine has judged. Pinning the
toolchain is a small change; deciding whether it belongs here is yours.

---

## What in the research brief turned out wrong

Recorded because the brief asked, and because a brief is an artefact of the same kind this phase
judges.

1. **"The guard sweep is roughly eleven hours at the current record count and rate."** It is
   about nineteen and a half. The brief re-took the count correctly, 734, and inherited a rate
   nobody re-derived: eleven hours implies 54 seconds a record, which is below the 66-second
   library run alone and ignores the 29-second rebuild. This is the exact failure the brief warns
   about, one term re-measured and the other inherited, arriving in the warning itself.

2. **"Count guard records by parsing `tests_last_seen` blocks with the awk in `CLAUDE.md`, never
   by grepping a file name."** The grep half is right; the awk half is not. See the headline
   section.

3. **"Phase 6 is executing in parallel with you."** Wrong when I first measured it and right by
   the end of the session: phase 6 had eight plans and no summaries at `a0b909e8`, with phase 7
   in flight, and began executing about ninety minutes later at `dcbe5ee4`. Both readings are in
   "What phase 6 could invalidate" with their times. The instruction not to write into phase 6's
   directory was followed, and nothing under `src/`, `tests/`, `scripts/`, `guards/` or `docs/`
   was touched by me; the staged change to `feedback.rs` in the working tree is phase 6's
   executor's.

4. **Three figures the brief gave were correct and are confirmed:** `main` at `a0b909e8`, version
   `0.119.0`, 734 guard records. `.planning/WINDOWS.md` reaching entry 342 is also confirmed, and
   its frontmatter counts agree exactly with its own table: 342 total, 320 open, 22 fixed.

---

## Environment availability

| Dependency | Required by | Available | Version | Note |
|---|---|---|---|---|
| `cargo-mutants` | PERF-07 | yes | 27.1.0 | `cargo mutants --list` needs no build |
| `cargo-llvm-cov` | PERF-05 | yes | 0.8.7 | a coverage run costs a full instrumented rebuild |
| `python` with `tomllib` | any structured counting | yes | 3.14.3 | the authoritative reader for `guards.toml` outside the test target |
| `gh` | reading CI state | yes | authenticated | |
| `hyperfine` | PERF-02 | **no** | | `Measure-Command` is the fallback and is adequate |
| a memory profiler | PERF-01, PERF-04 | **no**, and none is wanted | | the OS answers it through the started process |
| a Linux or macOS machine | nothing in this phase | no | | recorded under what cannot be measured |
| disk headroom | both long jobs | `target/` already 258.5 GB | | worth checking before a sweep starts |

---

## Validation architecture

`.planning/config.json` sets `workflow.tdd_mode: true` and does not set
`workflow.nyquist_validation`, so validation is on.

| Property | Value |
|---|---|
| Framework | `cargo test`, with `#[cfg(test)] mod tests` beside the code and integration targets under `tests/` |
| Config | `Cargo.toml`; `.cargo/mutants.toml` for mutation; `WIXEN_TEST_THREADS` defaults to 8 |
| Quick run | one `--lib` path per invocation, joined with `&&` |
| Full suite | `bash scripts/check.sh all` |
| Gate | `commit-msg` hook via `git config core.hooksPath .githooks`; `scripts/which-checks.sh` decides scope |

**Requirements to tests.** Every check this phase writes is a source-reading guard plus a
companion, in `tests/house_style.rs` or a new integration target. A new integration target needs
a `guards/guards.toml` record, or `scripts/check.sh` cannot tell which commits could break it;
`CLAUDE.md` records two guards that ran on every commit except the ones that mattered for exactly
that reason.

**Wave 0 gaps.** None for infrastructure. The gap is a decision: whether the new checks live in
`tests/house_style.rs`, which already holds 69 test functions and is named by many guard records,
or in a new target. `CLAUDE.md` records that where a new test lives is a planning cost decision
once per-file test-count bookkeeping exists, and that the cheapest home can be much cheaper than
the obvious one. Measured this session: `tests/house_style.rs` holds 69 test functions, and a
test added to it flags every guard record whose `tests_last_seen` names it.

**One measurement this phase must take before it writes any test:** how many records name
`tests/house_style.rs`. It decides where the phase's own tests go.

---

## Sources

**Primary, HIGH confidence.** Everything above came from one of these, this session:

- The working tree at `a0b909e8`, read with `grep`, `sed`, `find`, `wc` and `python -c` with
  `tomllib`.
- Historical file contents at `9d6543a`, `485030f` and `eda2719` through `git show`.
- Commit bodies through `git log`, which is where the durational series lives.
- `gh run list` and `gh run view` for CI state.
- `command -v` and `--version` for installed tooling.
- PowerShell `Get-ChildItem -Recurse -File | Measure-Object -Property Length -Sum` for disk.

**Carried forward with attribution:** the four-kind taxonomy and the "existing machinery" list
from `08-RESEARCH.md` of 2026-09-06, and the two accessibility figures from `06-RESEARCH.md` of
2026-09-12, which are phase 6's to publish.

**Nothing came from a web search, and no external source was consulted.** This phase adds no
package, so there is no package legitimacy audit to run.

---

## Metadata

**Confidence breakdown:**

- The inventory and every figure in it: HIGH. Each was taken from the thing that decides it, at a
  named commit, with the command written down.
- The two cost models: MEDIUM, and each says which term is measured and which is inherited.
- The claim that the fossilised ratio no longer supports its conclusion: MEDIUM, and A3 says why.
- What cannot be measured here: HIGH, verified by search over the whole tree or by the absence
  of any workflow run.

**Research date:** 2026-09-12. Every figure was taken on `main` at `a0b909e8`, version `0.119.0`,
with the working tree clean. **The tree moved before this file was finished:** `dcbe5ee4` landed
during the session and a change to `src/presentation/accessibility/feedback.rs` was staged by
phase 6's executor. No figure above was re-taken at the newer commit, so read every one of them
as measured at `a0b909e8` and not as a description of the tree you are looking at.

**Valid until:** the figures in Table A are good for days, not weeks. This repository took 440
commits in the seven days to 2026-09-12 (`git rev-list --count --since=2026-09-05 HEAD`), about
63 a day, so expect the guard record count and the test count to have moved before this phase's
first plan is written. **Take them again rather than
quoting this file.** That instruction is the whole point of the phase, and it applies to the
phase's own research first.
