---
phase: 08-every-number-the-project-quotes
plan: 02
status: complete
subsystem: testing
tags: [measurements, provenance, guards, red-green, constants, house-style]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "`docs/development/measurements.md` as the one page a test count is quoted from, and the target this plan's four readings were added to"
provides:
  - "`tests/every_number_carries_its_command_and_its_date.rs`, 25 tests where 08-01 left 10: a provenance reading over every page a person believes, an agreement reading over the three pages that state the test count, a check that computes and prints the share of history before red/green, a reading that holds twelve prose figures to the constants they restate, and a companion for each shown red first"
  - "Thirteen paragraphs on three pages dated, none re-numbered; two pages quoting 7,697 tests from two new rows; four tree sites naming the fraction check; the roadmap's attachment line saying what the code does; the privacy page's update size a target"
  - "`guards/guards.toml`: six records, each measured on the target; census 598, 790 records"
affects: [08-06, 08-09]

actuals:
  tokens: 21900
  tasks: 3
  commits: 10

tech-stack:
  added: []
  patterns:
    - "A reading over pages with its patterns compiled once per process, because a document walk that compiles per paragraph put each companion past a minute"
    - "A ratio computed by an external process on every commit and printed with the day, with the sites that used to state it naming the check"
    - "A table of page phrase, unit and imported constant, so a renamed constant fails to compile and a reworded phrase fails to be found"
    - "A source-side guard record whose suite is an integration target, because a record has one suite and the library reddened nothing"

key-files:
  created: []
  modified:
    - tests/every_number_carries_its_command_and_its_date.rs
    - docs/development/measurements.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/integration-guide.md
    - docs/roadmap.md
    - docs/privacy.md
    - docs/changelog.md
    - CLAUDE.md
    - .cargo/mutants.toml
    - scripts/mutants.sh
    - src/presentation/accessibility/sound_scheme_import.rs
    - guards/guards.toml
    - .planning/WINDOWS.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

key-decisions:
  - "A past test count on the three test-count pages is written in a shape other than `N tests`, because the agreement reading cannot tell a past claim from a present one; the convention is stated in the reading's comment and the integration guide's one historical figure was reworded to fit it"
  - "The agreement reading takes a row as being about tests when its `what` begins with the word test, because a substring search took a guard-record row naming `tests_last_seen` for one"
  - "A target line is excused from both the date and the source, not the date alone: an aim has no day of being taken and no command that produced it"
  - "The source-side constants record names this target as its suite and no library-suite twin is written, because the whole library stayed green under the break and a record with an empty red list measures nothing"
  - "The two `.planning` validation records keep the two-absolutes sentence, because each is a record of what was believed on its day"

patterns-established:
  - "A red for a reading over text: write the reading in the form a first draft would take, commit it red naming what it sees, then show the companion for the case it misses going red before the reading is corrected"

requirements-completed: []

coverage:
  - id: D1
    description: "A count, percentage or duration on a page under docs/, in README.md or in CLAUDE.md sits beside a date and a command or named source, or names a target"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_figure_on_a_page_carries_its_date_and_its_source"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_provenance_reading_is_looking_at_the_pages_it_is_for"
        status: pass
    human_judgment: false
  - id: D2
    description: "The reading can see a bare count, a percentage without a date, a duration without a source, excuses a target line, and refuses an empty walk"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_provenance_reading_can_see_a_count_without_a_date"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_provenance_reading_can_see_a_percentage_without_a_date"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_provenance_reading_can_see_a_duration_without_a_source"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_provenance_reading_excuses_a_line_that_names_a_target"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_provenance_reading_refuses_an_empty_walk"
        status: pass
    human_judgment: false
  - id: D3
    description: "A test count stated on the three pages is the value of a row on the measurements page"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_three_pages_that_state_the_test_count_quote_one_row"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_agreement_reading_can_see_a_count_no_row_holds"
        status: pass
    human_judgment: false
  - id: D4
    description: "The share of history before red/green is computed and printed, and no site states it as two absolutes, wrapped or not"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_share_of_history_before_red_green_is_computed_and_printed"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_ratio_reading_can_see_the_one_line_form"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_ratio_reading_can_see_the_wrapped_form"
        status: pass
      - kind: other
        ref: "grep -Pzl 'commit 182 of\\s+(#\\s*)?344' over the four tree sites names nothing; over the two planning validation records names both"
        status: pass
    human_judgment: false
  - id: D5
    description: "Twelve prose figures equal the constants they restate, a moved figure is named, and a reworded phrase is refused"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_figure_that_restates_a_constant_agrees_with_it"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_constants_reading_can_see_a_figure_that_moved"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_the_constants_reading_refuses_a_phrase_that_has_gone"
        status: pass
    human_judgment: false

duration: 130min
completed: 2026-09-14
---

# Phase 8 Plan 02: Every count says when and how it was taken Summary

**Four readings in the target 08-01 created, each shown red first: a figure on any page a person believes carries its date and its source, the three test-count pages quote one row of the measurements page, the share of history before red/green is computed by `git rev-list` and printed rather than written, and twelve prose figures are held to the constants they restate. Thirteen paragraphs dated and none re-numbered; the roadmap's attachment line corrected from a 10 MB warning that never existed to what the code does at 25.**

Every count below is one this session took, with the command. Where the tree contradicted the plan, the tree won and the difference is named.

## Performance

- **Duration:** about 130 minutes, from the first read at about 20:10Z to the merge
- **Started:** 2026-09-14T20:10:00Z (approximate; the first commit is 20:52Z)
- **Completed:** 2026-09-14T22:10:00Z for the merge at `4ce4ad96`; the summary followed
- **Tasks:** 3 of 3
- **Files modified:** 13 on the branch, plus the three planning files on `main`

## Accomplishments

- `tests/every_number_carries_its_command_and_its_date.rs` holds 25 tests where 08-01 left 10, by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`. Four readings, each a function over text returning what it found, each with companions that splice a violation into the real page's own lines and require exactly that answer.
- The provenance reading walks 29 pages: `find docs -name '*.md'` gives 27 after leaving out `docs/changelog.md` and `docs/plans/`, plus `README.md` and `CLAUDE.md`. It refuses a walk of fewer than five pages and a walk that found no figure of any kind. Its first run named 13 figures on three pages, listed below with what changed in each.
- The agreement reading holds every `N tests`, `N unit tests` or `N integration tests` in a dated paragraph on `docs/IMPLEMENTATION_STATUS.md`, `docs/integration-guide.md` and `README.md` to a row on the measurements page whose `what` begins with the word test and whose value is a bare count. Its first run named 5,430 on the status page and 64 in the integration guide.
- The fraction check pins `18a02454`, requires it to be an ancestor of `HEAD`, counts with `git rev-list --count`, and prints one line. At the merge: `181 of 2043 commits, 8.9%, predate red/green as of 2026-09-14`. It asserts no value. It unwraps `#` and `//`, joins lines and collapses whitespace before refusing `commit N of M`, and names the file and the line the match starts on.
- The constants reading holds twelve pairs. A pair whose phrase is not on its page is a failure that says the page was reworded.
- Six guard records, each break applied by hand and measured on the target before it was written, and confirmed by `scripts/guards.sh --remeasure`. Census 192 + 598; 790 records by the TOML reader, from 784.
- `scripts/check.sh all` green on the branch at `f94dacca`: 7,702 passed, none failed, 55 result lines, 321 s. Green again on the merge at `4ce4ad96`, 7,702 passed. No keyring race either time.

## The thirteen paragraphs the provenance reading named, and what changed

The reading's own output on its first run, then the correction. Line numbers are the reading's, at `630ead8c` before and at `4ce4ad96` after. Kinds follow the research's taxonomy. No figure in any of them changed.

| Before | Figure | Kind | What changed | After |
|---|---|---|---|---|
| `CLAUDE.md:415` | "5 tests" | narrative of a past run | gained "on 2026-08-30", the day 01-02's record was written (`01-02-SUMMARY.md` completed that day) | `CLAUDE.md:424` |
| `CLAUDE.md:453` | "77 records" | narrative of a past grep | gained "on 2026-09-06" and past tense; the commit that wrote it is `972b66f3` of that day | `CLAUDE.md:462` |
| `CLAUDE.md:501` | "sweep ... 90 minutes" | narrative with a date and no source | gained `scripts/guards.sh` scoped to the branch as the thing that ran | `CLAUDE.md:512` |
| `CLAUDE.md:503` | "220 records" | same paragraph as the row above | same edit | `CLAUDE.md:513` |
| `CLAUDE.md:536` | "sweep is about **20 hours**" | stale measurement | dated 2026-09-06, past tense, pointed at the product row on `docs/development/measurements.md`; the figure stays as that day's | `CLAUDE.md:546` |
| `CLAUDE.md:538` | "6,000 test functions" | same paragraph as the row above | same edit | `CLAUDE.md:548` |
| `CLAUDE.md:555` | "63 records" | narrative of plan 02-01 | gained "on 2026-08-31" (`02-01-SUMMARY.md` landed that day) and what was counted, records in `guards/guards.toml` naming a test in a changed module | `CLAUDE.md:566` |
| `CLAUDE.md:568` | "5,837 tests" | dated series with no command | gained `cargo test --lib` with `WIXEN_TEST_THREADS` | `CLAUDE.md:581` |
| `CLAUDE.md:595` | "111 tests" | "measured the same day" with the day unnamed | gained "2026-09-09", the day of the curve above it (`02ddbcb0`) | `CLAUDE.md:609` |
| `CLAUDE.md:626` | "3,500 tests" | narrative of the audio crash | gained "on 2026-09-03", the day `WIXEN_NO_AUDIO` was written (`506fc868`, `4f2285e5`) | `CLAUDE.md:640` |
| `docs/IMPLEMENTATION_STATUS.md:235` | "157 mutants" | dated record with no source | gained the progress table of `docs/plans/20260801-mutation-sweep.md`, whose first row is this run | `docs/IMPLEMENTATION_STATUS.md:243` |
| `docs/IMPLEMENTATION_STATUS.md:241` | "66 mutants" | dated record with no source | the commit went into backticks and the run gained `scripts/mutants.sh` | `docs/IMPLEMENTATION_STATUS.md:252` |
| `docs/integration-guide.md:5` | "64 tests" | dated narrative with no source, and a stale present tense | gained the measurements page and today's count; "64 tests" became "the count of tests at 64" for the reason under Deviations | `docs/integration-guide.md:5` |

Two the prototype in the plan named that the reading does not: `docs/architecture.md:400`, "coverage (target: 80%", excused by the word target, which is a record's break; and the prototype's own false match "06 records" in "PERF-06 records that", which the reading's `NOT_PART_OF_A_WORD` prefix drops.

## The test count, and the two rows behind it

Taken at `a42331bb` after the red commit, every target built first with `--no-run`:

```
cargo test --all-targets -- --list 2>&1 | grep -E '^[0-9]+ tests?,' | awk '{s+=$1} END{print s}'
-> 7697            (55 targets)
cargo test --all-targets -- --list 2>&1 | awk '/Running tests/{t=1;next} /Running/{t=0;next} t && /^[0-9]+ tests?,/{s+=$1} END{print s}'
-> 452             (53 targets under tests/)
cargo test --lib -- --list | tail -1
-> 7245 tests, 0 benchmarks
```

7,245 + 452 = 7,697; the binary's unit tests number none. The library row 08-01 wrote at `7b2482b1` still holds, so it was not re-taken as a new row, which the page would refuse for a second row with one `what` and one date. The status page now reads "7,697 tests: 7,245 unit tests and 452 integration tests" with the date, the commit and the command, and keeps "This paragraph said 5,430 (5,269 unit, 161 integration, counted 2026-08-29 the same way) until 2026-09-14". `grep -n "5,430" docs/IMPLEMENTATION_STATUS.md docs/integration-guide.md` finds each page's one sentence about what it used to say.

## The fraction, with its command

```
git merge-base --is-ancestor 18a02454 HEAD    -> exit 0
git rev-list --count 18a02454^                -> 181
git rev-list --count HEAD                     -> 2043 at 4ce4ad96 (2036 at the red commit, 2037 at the green)
cargo test --test every_number_carries_its_command_and_its_date test_the_share -- --nocapture
-> 181 of 2043 commits, 8.9%, predate red/green as of 2026-09-14
```

```
grep -Pzl 'commit 182 of\s+(#\s*)?344' CLAUDE.md docs/IMPLEMENTATION_STATUS.md .cargo/mutants.toml scripts/mutants.sh
-> nothing, exit 1
grep -Pzl 'commit 182 of\s+(#\s*)?344' .planning/phases/01-folders-and-conversations/01-VALIDATION.md .planning/phases/02.1-what-phase-1-found-on-its-way-past/02.1-VALIDATION.md
-> both names
```

The two comment sites now read, in `.cargo/mutants.toml`: "Red/green started at 18a02454 on 2026-07-26. How much of the history predates it is computed and printed on every commit by test_the_share_of_history_before_red_green_is_computed_and_printed in tests/every_number_carries_its_command_and_its_date.rs: 8.9% on 2026-09-14. This comment used to give the commit's position and the count on the day it was written, two absolutes that stayed true while the share they implied fell from 53%." And in `scripts/mutants.sh` the same, with "wrapped across two lines so that a line grep never saw it" added, because that is what that site did. The two planning validation records, `01-VALIDATION.md:45` and `02.1-VALIDATION.md:54`, are not edited: each records what its phase believed on its day, and correcting it would falsify the record.

**The wrapped companion's red, quoted.** The red commit's reading was written line by line on purpose. With the four sites corrected and the join not yet written, one test was red: `test_the_ratio_reading_can_see_the_wrapped_form`, "one site was planted and the reading named []". The join turned it green. At the red commit itself the line reading named three sites and missed `scripts/mutants.sh`, exactly as the plan's premise 3 said a line grep would.

## The twelve pairs

| Page | Phrase before the figure | Figure | Constant |
|---|---|---|---|
| `docs/privacy.md` | A single file is kept up to | 25 MB | `LARGEST_ATTACHMENT_KEPT_BYTES` |
| `docs/privacy.md` | all of them together up to | 512 MB | `ATTACHMENT_CACHE_BUDGET_BYTES` |
| `docs/privacy.md` | dropped when a signed message is larger than | 25 MB | `LARGEST_SIGNED_MESSAGE_KEPT_BYTES` |
| `docs/privacy.md` | when the space these copies use passes | 128 MB | `SIGNED_ORIGINAL_BUDGET_BYTES` |
| `docs/privacy.md` | A message larger than | 25 MB | `LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES` |
| `docs/privacy.md` | interrupted moves add up to more than | 64 MB | `MOVES_IN_FLIGHT_BUDGET_BYTES` |
| `docs/KEYBOARD_SHORTCUTS.md` | PNG, JPEG, GIF and WebP up to | 2 MB | `MOST_ONE_PICTURE_MAY_BE` |
| `docs/plans/20260823-earcon-sound-schemes.md` | a working number to start from: | 20 MB | `MAX_ZIP_BYTES` |
| `docs/plans/20260823-earcon-sound-schemes.md` | extracted-size cap (a working number: | 5 MB | `MAX_FILE_BYTES` |
| `docs/plans/20260823-earcon-sound-schemes.md` | a total cap across the whole pack ( | 50 MB | `MAX_TOTAL_BYTES` |
| `docs/plans/20260823-earcon-sound-schemes.md` | a rule and not a suggestion.** A working number: | 2 seconds | `MAX_SOUND_DURATION` |
| `docs/roadmap.md` | once encoded for sending, come to more than | 25 MB | `attaching::LIMIT_BYTES` |

At the red commit the twelfth pair anchored on the old wording, "Attachment size warnings (>", and the reading answered `docs/roadmap.md: "Attachment size warnings (>10 MB" restates attaching::LIMIT_BYTES, which is 25 MB`. The line now says: "A warning when the attachments, once encoded for sending, come to more than 25 MB, spoken as each file is added. The limit is `LIMIT_BYTES` in `src/application/attaching.rs`, and a check holds this line to it. This line said "Attachment size warnings (>10MB)" until 2026-09-14; there was never a warning at 10 MB." That is what `attaching::over_the_limit` does, reached from `wx_compose.rs:1527` through `choose_all` and `what_to_say`: an announcement marked trouble, once per batch, saying most providers refuse past 25 MB.

The four sound-scheme constants are `pub` with a doc line each saying the reading is why. Nothing else changed in that file; the gate ran `presentation::accessibility::sound_scheme_import` on that commit and it was green.

`docs/privacy.md:417` said the update download "is roughly 12 MB today". No release has been published, so the sentence now says the size has not been measured, that about 12 MB is the target, and that the measured size will go here with its date once there is a release. Ledger 444.

## What the tree contradicted in the plan

1. **A guard record has one suite, so the source-side constants record could not name both the library's tests and this target's.** The plan's task 3 said the break on `attachment_content.rs` "also reddens whatever unit tests pin that constant; the record names all of them or it is short". Under that break, `cargo test --lib -- --test-threads=8` was 7,244 passed and none failed in 73 s: the tests beside the constant are written relative to it (`LARGEST_ATTACHMENT_KEPT_BYTES + 1`) and stay green at any value. So the record names this target as its suite, its three reddening tests, and says in its comment what the library did and why there is no library-suite twin. This target's reading is the only thing in the tree that holds the number.
2. **`check.sh --suites-for guards/guards.toml src/data/message_cache/attachment_content.rs` prints nothing**, as 08-01 found and ledger 442 records: the target is in `guards_that_read_the_whole_tree`, so the coupling function drops it. The coupling holds another way, visible in every hook log on the branch: each scoped run ended with "the guards that read the whole tree", which includes this target, and the commit that made the constants `pub` selected `presentation::accessibility::sound_scheme_import` beside it.
3. **The prototype's count was 13, not 16**, because 08-01 had corrected three paragraphs since the plan was written and the prototype matched "PERF-06 records" as a count. The reading's own first run also found 13, two of them second figures in paragraphs the prototype counted once.
4. **The agreement reading as specified would have flagged the integration guide's historical "64 tests".** See Deviations.
5. **The plan's premise 5 said `LARGEST_ATTACHMENT_KEPT_BYTES` is at `attachment_content.rs:51`.** It is. All seven `pub` line numbers matched.

## Task commits

Branch `every-count-says-when-and-how` from `main` at `630ead8c`.

1. **Task 1, red:** `a42331bb` test(08-02), seven tests and the count check named in trailers, red against thirteen figures and two stale counts.
2. **Task 1, green:** `1df4286f` feat(08-02), the pages, two rows, the regex cells, the changelog entry, 08-01's record re-measured to 19.
3. **Task 1, records:** `957a2d31` test(08-02), three records, census 595.
4. **Task 2, red:** `3231e4b2` test(08-02), the fraction check read line by line, three tests and the count check named.
5. **Task 2, green:** `ace8f482` feat(08-02), the join, the four sites, four records re-measured to 22.
6. **Task 2, record:** `2233056a` test(08-02), the wrapped break, census 596.
7. **Task 3, red:** `7845ee17` test(08-02), the constants reading and the four `pub`s, three tests and the count check named.
8. **Task 3, green:** `32a35c41` feat(08-02), the roadmap line, the privacy sentence, ledger 444, five records re-measured to 25.
9. **Task 3, records:** `f94dacca` test(08-02), two records, census 598.
10. **Merge:** `4ce4ad96`, gate green on the merge.

**Plan metadata:** the commit carrying this summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md`.

What the gate selected per commit, from the hook's own log: every commit that touched the target answered `affected` and ran `every_number_carries_its_command_and_its_date` plus the tree guards; the commit touching `src/presentation/accessibility/sound_scheme_import.rs` ran that module's tests as well; the three record-only commits ran the tree guards alone, since `guards/guards.toml` maps to no target; `.cargo/mutants.toml` and `scripts/mutants.sh` are read by nothing but this target's fraction check and the tree guards, which is what the fraction check exists for. `CLAUDE.md`, the four pages under `docs/` and the changelog rode with the target in every commit that touched them, so no commit answered `docs_only`.

## Deviations from Plan

**1. [Rule 2, missing critical] A past count on the three test-count pages is written in a shape the agreement reading does not match.** The plan's reading takes every `N tests` in a dated paragraph on the three pages. The integration guide's "counted 64 tests against the 5,430 that run today" is a record of what an old version said, and no row will hold 64. The reading cannot tell a past claim from a present one, so the convention is that on those three pages a present count is written `N tests` and a past one another way; it is stated in the reading's section comment, the guide now reads "put the count of tests at 64", and ledger 445 records that the convention lives in the check rather than where a page author meets it. This is rewording to satisfy a reading, which `CLAUDE.md` warns about, and it is accepted because the companion still proves the reading sees a violation and the sentence's meaning is unchanged.

**2. The regex cells.** The first green run took 113 s, each companion past a minute, because every paragraph compiled three patterns. `OnceLock` cells compile each once; the run is 0.2 s. Not in the plan; a check that costs a minute per commit is a check people route around.

**3. The agreement rows are those whose `what` begins with the word test.** The plan said "whose `what` mentions tests"; a substring search took "Guard records naming exactly one file in `tests_last_seen`" for a test-count row. The page's own naming carries the distinction and the reading's comment says so.

**4. A target line is excused from the source as well as the date.** The plan's wording excused it from the date; `docs/architecture.md:400` has no backticked token either, and a target has no command that produced it. The excuse is the word, and a record proves it is the only thing excusing that line.

Otherwise the plan was executed as written. No package installed, `Cargo.toml` untouched, no version bump, no `unwrap` or `expect` outside the test target, no tracked file edited by a script.

## Guard records

Six added, each measured by hand on the target and confirmed under `--remeasure`, and 08-01's record re-measured three times as the file grew: 19, 22, 25.

| Record | Break | Red | Predicted | Measured |
|---|---|---|---|---|
| a figure on a page cannot lose the date it was taken | the day off `CLAUDE.md:416` | 5 | 5 | 14 passed, 5 failed |
| the word target is the only thing excusing a figure from its date | "target" to "aim" at `docs/architecture.md:400` | 5 | 5 | 14 passed, 5 failed |
| a test count on a page is the value of a row on the measurements page | 7,697 to 7,698 at `docs/IMPLEMENTATION_STATUS.md:171` | 2 | 2 | 17 passed, 2 failed |
| no site states the share of history before red/green as two absolutes | the wrapped form back into `scripts/mutants.sh:9` | 3 | 3 | 19 passed, 3 failed |
| a page cannot restate a constant with a figure the code does not hold | 25 to 26 MB at `docs/privacy.md:66` | 3 | 3 | 22 passed, 3 failed |
| a constant cannot move away from the page that restates it | 25 to 26 MiB at `attachment_content.rs:51` | 3 | 3 | 22 passed, 3 failed; library 7,244 passed, 0 failed |

Every prediction matched, which is the companion pattern working: each companion asserts the real pages are clean before it splices, so a break in a page reddens the reading and every companion of that reading.

No test was added to `tests/house_style.rs` or `tests/wired.rs`: 70 and 71 before, 70 and 71 after, by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`. The count check fired on every red commit and was named in its trailers with the reason; its remedy was run and read after each green, six records in the last run, every one still exactly its own red.

Records: 784 before by the TOML reader, 790 after. Census 192 + 598.

## Ledger

`.planning/WINDOWS.md` 443 before, 445 after, both halves of each entry written by `gsd-tools windows append`, no backslash in either, checked by `grep -n "| 444 |"` and `grep -c '"id": 444'` and the same for 445:

- 444, todo, `docs/privacy.md:417`: the update download size is a target and not a measurement until a release exists to measure.
- 445, deviation, `docs/integration-guide.md:5`: the agreement reading's convention for a past count lives in the reading's comment.

## Issues Encountered

None that stopped anything. The gate passed on every commit through the hook, never piped; `check.sh all` was redirected to a file and its exit status read directly, green on the branch and on the merge. The first heredoc this session, used only to write a scratch script outside the tree, had its backslashes stripped; the scratch file was written with the editing tool instead and no tracked file was touched by anything but the editing tool and `cargo fmt`.

## Known Stubs

None. Every reading runs on every commit through `check.sh`'s two lists, which 08-01's self-check holds; every page correction is on the page; every record is measured.

## Threat Flags

None. The fraction check runs `git rev-list` and `git merge-base` read-only against the repository it is in; no network, no write, no schema.

## Self-Check: PASSED

The target exists on disk and holds 25 tests; the ten commits named above are in `git log --all`, checked by `grep` before this section was written. `main` is ahead of `origin/main` and nothing was pushed.

## Next Phase Readiness

08-03 starts from `main` at the metadata commit. Every count a later plan quotes on a page now has a check behind its date and its source, so 08-06's pass over `CLAUDE.md`'s remaining gate and sweep figures will meet the provenance reading and must date and point rather than restate. 08-09 rewrites PERF-07's justification and can quote the fraction line rather than a number.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-14*
