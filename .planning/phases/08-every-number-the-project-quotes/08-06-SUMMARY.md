---
phase: 08-every-number-the-project-quotes
plan: 06
status: complete
subsystem: testing
tags: [measurements, guards, provenance, documents, counting, audit]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "the rate, count and product rows on `docs/development/measurements.md` that every retired sweep figure now points at, and the `timed:` line the runner prints"
  - phase: 08-every-number-the-project-quotes
    plan: 02
    provides: "the provenance reading over `CLAUDE.md` and the status page, which read every paragraph this plan rewrote on the commit that rewrote it"
  - phase: 08-every-number-the-project-quotes
    plan: 05
    provides: "the coverage rows and ledger 452, which is what PERF-05's `[S]` line was dated and added to from"
provides:
  - "`CLAUDE.md` prescribes the TOML parser for counting guard records, says what the awk it used to prescribe missed and by how much, and states the sweep's cost as a rate times a count on the measurements page rather than as a figure"
  - "Every gate, suite, sweep and mutation-run duration in `CLAUDE.md` and the status page either carries its own date and names the row to read now, or is gone with its old figure kept as the figure of its date"
  - "The four comments that gave the sweep a figure, plus the fifth in `scripts/guards.py` that ledger 443 named, point at the same row; the three undated advisory acceptances in `.cargo/audit.toml` say since when they have stood"
  - "`.planning/REQUIREMENTS.md`, `.planning/PROJECT.md` and `.planning/STATE.md` state no undated figure about the tree in a sentence written as current; PERF-05's `[S]` line is dated and added to, not deleted"
affects: [08-07, 08-09]

actuals:
  tokens: 10566
  tasks: 3
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A retired figure is kept as the figure of its date in the sentence that replaces it, and the sentence names the row to read now; nothing is re-numbered"
    - "A comment about a cost that is a product of two moving terms names the page the terms live on and gives no product"

key-files:
  created: []
  modified:
    - CLAUDE.md
    - docs/IMPLEMENTATION_STATUS.md
    - docs/changelog.md
    - tests/house_style.rs
    - guards/guards.toml
    - scripts/guards.sh
    - scripts/guards.py
    - .cargo/audit.toml
    - .planning/REQUIREMENTS.md
    - .planning/PROJECT.md
    - .planning/STATE.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md

key-decisions:
  - "The awk finding is written at the size the checker corrected it to, and re-measured today rather than copied: level with the parser for every file no inline-table record names, 0 against 1 for `wx_send_later.rs`, with `wx_app.rs` at 50 on both sides where the plan's table said 48"
  - "The test count taken today, 7,750, stays in `REQUIREMENTS.md` as that file's own measurement and is not added to the page, because the page refuses a second row with one `what` and one date and 08-02's 7,697 row is dated today"
  - "Three dated quotations of a retired phrase were reworded to keep the figure without the phrase, because the plan's single-line greps for absence and its rule to keep the old wording could not both be met; `CLAUDE.md`, whose criterion admitted a dated sentence, quotes all four phrases as written"
  - "`scripts/guards.py` was edited though the plan's file list did not name it, because ledger 443 had assigned it to this plan and task 2's done criterion is that no comment in the tree states the sweep's cost as a figure"
  - "`CLAUDE.md:286`'s 'two targets that build a live window' was checked against the documents-only list and left: `checkbox_labels` and `manager_delete_stays_open` build one, and `house_style` and `wired` only name the toolkit in strings"

patterns-established:
  - "Before editing a comment in a file guard records break, list every record's `before` text in that file with the parser and check none sits on a line the edit touches"

requirements-completed: []

coverage:
  - id: D1
    description: "CLAUDE.md prescribes the parser, not the awk, and says what the awk missed and when"
    requirement: PERF-06
    verification:
      - kind: other
        ref: "grep -n \"^awk '\" CLAUDE.md finds nothing; grep -c tomllib CLAUDE.md is 1; the one-liner as written prints 77 for contacts_sync.rs and 798 for len(g)"
        status: pass
    human_judgment: false
  - id: D2
    description: "No page, comment or planning document states the guard sweep's cost as a figure; each names the row"
    requirement: PERF-06
    verification:
      - kind: other
        ref: "grep -Pzo 'an hour\\s+#? ?or two|eighteen\\s+hours' tests/house_style.rs guards/guards.toml scripts/guards.sh prints nothing; CLAUDE.md holds '20 hours' only in the sentence naming the four retired figures"
        status: pass
    human_judgment: false
  - id: D3
    description: "Every figure the provenance reading can see on CLAUDE.md and the status page is dated and sourced"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_figure_on_a_page_carries_its_date_and_its_source"
        status: pass
    human_judgment: false
  - id: D4
    description: "No comment edit landed on a guard record's locator, and no test count moved"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/house_style.rs#test_every_guard_record_still_names_one_place_in_the_tree"
        status: pass
      - kind: integration
        ref: "tests/house_style.rs#test_every_guard_record_says_how_many_tests_the_files_it_names_held"
        status: pass
    human_judgment: false
  - id: D5
    description: "The three planning files agree with themselves and hold no em dash"
    requirement: PERF-06
    verification:
      - kind: integration
        ref: "tests/the_planning_files_agree_with_themselves.rs, 16 passed"
        status: pass
      - kind: integration
        ref: "tests/house_style.rs#test_no_dashes_that_should_be_punctuation"
        status: pass
    human_judgment: false

duration: 40min
completed: 2026-09-14
---

# Phase 8 Plan 06: Four figures for one sweep become one row Summary

**`CLAUDE.md` counts guard records with the TOML parser and says the awk it prescribed for eight days skipped a record spelled on one line; the four undated figures the tree gave for one guard sweep, and a fifth the ledger had found, each point at the rate-times-count rows on the measurements page with the old figure kept as its day's; the gate, suite and mutation-run durations on the two pages people trust carry their dates and name the row to read; the three planning documents state their figures with a date and a command, and PERF-05's transport attribution is dated and added to rather than deleted. Documents and comments only: no test, record or setting changed value.**

Every count below is one this session took, with the command. Where the tree contradicted the plan, the tree won and the difference is named.

## Performance

- **Duration:** about 40 minutes to the merge, from the first read at about 02:45Z to the merge at 03:19Z; the summary followed
- **Started:** 2026-09-15T02:45:00Z, which is the evening of 2026-09-14 on the machine, and every date written says 2026-09-14
- **Completed:** 2026-09-15T03:19:00Z for the merge at `53b9300f`
- **Tasks:** 3 of 3
- **Files modified:** 11 on the branch, plus the summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md` on `main`

## Accomplishments

- `CLAUDE.md`'s "Count records, not mentions" paragraph prescribes the parser. `grep -n "^awk '" CLAUDE.md` finds nothing; `grep -c tomllib CLAUDE.md` is 1; the one-liner as written prints 77 for `contacts_sync.rs` and 798 for `len(g)` at `3accd6e1`.
- The sweep's cost is stated once, as a rate times a count on `docs/development/measurements.md`; `CLAUDE.md` names the four figures the tree used to give and dates each; `tests/house_style.rs`, `guards/guards.toml` (two sites), `scripts/guards.sh` and `scripts/guards.py` point at the same rows.
- Nine `CLAUDE.md` paragraphs and one status-page paragraph rewritten; every retired figure kept as the figure of its date. The provenance reading passed on each commit: 25 passed.
- Three advisory acceptances dated from `git log -S`, and both exit conditions checked against `Cargo.lock` and `cargo tree` today rather than restated.
- `REQUIREMENTS.md`'s seven PERF evidence lines re-taken at `7da68e78`; `PROJECT.md`'s six figures re-taken with their 2026-08-29 values kept; `STATE.md`'s one present-tense past count dated.
- `scripts/check.sh all` green on the branch at `ee40346c` on its first run, 7,746 passed and none failed over 58 result lines, 320 s, no keyring race; green again on the merge at `53b9300f`, 7,746 passed.

## Every sentence changed, before and after

Line numbers are those at `3accd6e1` before and `53b9300f` after. "Kept" means the old figure is still in the sentence as the figure of its date.

### `CLAUDE.md`

| Before | After | What changed |
|---|---|---|
| `:266` "the three targets that read documents" | `:266` "the targets that read documents, which are listed in `scripts/check.sh`'s documents-only branch with the reason for each (three when this sentence was written on 2026-08-31, nine on 2026-09-14)" | a count read as current; dated by `git log -S`, 4b817ed5 of 2026-08-31; nine counted from the script's branch |
| `:296` "Measured warm on 2026-08-30: the whole gate is 311 seconds, of which the test suite is 239 and the release build 56, so the quick pair is 15 seconds and the slow two are everything else." | `:298` the same in the past tense, then "Those are that day's figures and the shape is what to keep. What the gate costs now is on `docs/development/measurements.md`: the full-gate row is a band harvested from the commit bodies, 275 to 654 seconds across the branches that recorded one on 2026-09-14, and the row for `cargo test --all-targets` read 104 seconds the same day with every target already built, which is the suite's term taken on its own rather than inside a gate run, so read the two against each other with that difference in mind." | kept; pointed at the band row and the suite row |
| `:390` "A whole-tree run is about two days, so it is used scoped" | `:401` "A whole-tree run costs a rate times a count, and the count is a row on `docs/development/measurements.md`: `cargo mutants --list` answered 12,335 mutants over 247 files on 2026-09-14. The rate is not written anywhere yet on purpose; it is measured on one shard, under the suite shape the run will use, before any whole run is scheduled, and the product goes on that page beside the count. This sentence said a whole-tree run is "about two days" from 2026-07-29 until 2026-09-14, with no date, no machine, no thread setting and no record of which count or rate it was built on, so nothing in it could be re-taken. So it is used scoped" | kept as a dated quotation; the sentence's first date from `git log -S`, 3f7ebd09 |
| `:464` "Parse the `tests_last_seen` blocks:" and the awk at `:467-470` | `:485` "Read the file with the format's own parser and count the records whose `tests_last_seen` names the file:" and the `tomllib` one-liner, then "Print `len(g)` instead of the sum for the number of records. The rows on `docs/development/measurements.md` are taken with these two commands and nowhere else." | the awk replaced |
| none | `:495` a new paragraph, "Until 2026-09-14 this paragraph prescribed an awk in place of the parser, and the awk miscounted." It says how the awk read the file, that four records are spelled inline, that the awk consumes such a line before looking for the file, and gives the measured table: level for `wx_app.rs` (50 and 50), `tests/house_style.rs` (21 and 21) and `contacts_sync.rs` (77 and 77) and for every file no inline record names; 0 against 1 for `wx_send_later.rs`. "The miss is small today and the shape is the point: a line reader for a structured file is right for the spelling the file happens to use and silently wrong for the others the format allows, and it hands back a plausible number rather than an error. It arrived here as the correction to a known miscount, which is the part worth remembering." | the finding at the checker's size, re-measured today |
| `:546` "At that size, on 2026-09-06, the sweep was about **20 hours**, not the 15 this used to say. The figure moved for two reasons at once: 52 more records, and a library that had grown to roughly 6,000 test functions, which put the per-record cost nearer 119 seconds than the 112 the old number assumed. Both terms go on moving, so multiply the count you take today by a rate you measure today; the product row on `docs/development/measurements.md` is where the latest pair is." and `:553` "**The rate halved on 2026-09-09 and the twenty hours above now overstates it.** A record costs one rebuild plus one whole-library run, and the library run stopped rebuilding the database schema once per test that opens a cache. Measured that day at 8 threads [...] 120.3s and 120.2s at `02ddbcb`, 64.8s and 63.8s with the schema template. Do not read a new total off that halving. The rebuild term was never measured beside the run term, so the instruction is the one above, unchanged: take the count today and measure the rate today." | `:586` two paragraphs. "**The sweep's cost is a rate times a count, and both terms live on `docs/development/measurements.md`, not here.**" names the count row, the rate row read off the `timed:` line, and the product row as the only place the total is written; "Until 2026-09-14 the tree gave four figures for this one job, none of them dated and none of them the same: "roughly 15 hours" in the roadmap, "about 20 hours" in this paragraph, "eighteen hours" in `tests/house_style.rs` and "an hour or two" in `guards/guards.toml` and `scripts/guards.sh`. Each was right on its day at that day's count and rate. The 20 hours here was 617 records at about 119 seconds on 2026-09-06, and it was already wrong three days later." Then "What moved it:" keeps the 2026-09-09 halving with its four figures word for word, adds "The rebuild term then rose on its own, from 29 s on 2026-09-10 to 44 s on 2026-09-14, which the rate row records", and corrects "was never measured beside the run term" to "it has been since 2026-09-10, and the runner has printed both apart since 2026-09-14." | kept; two paragraphs into two; the false "never measured" sentence corrected with its date |
| `:566` "63 records of the 536 that existed then, about 90 minutes, for plan 02-01 on 2026-08-31" | `:615` "63 records of the 536 that existed on 2026-08-31, about 90 minutes, for plan 02-01 that day" | the 536 dated in its own clause |
| `:613` "there the test term falls from 197s to 111s and the whole gate does not move, 335s against 353s." | `:663` "measured 2026-08-31, the test term fell from 197s to 111s and the whole gate did not move, 335s against 353s. Those are that day's figures; the same run read 104 seconds on 2026-09-14 with every target already built, the row on `docs/development/measurements.md`, which is the one to quote." | kept; dated from `git log -S`, 03d07951; pointed |
| `:619` "Running the records in parallel across git worktrees was measured and rejected: two concurrent suites take 131s each against 88s alone [...] Five workers would buy about 1.8x for 43GB of disk and a five-minute build each." | `:671` "was measured on 2026-08-31 with `cargo test --lib` and rejected: two concurrent suites took 131s each against 88s alone [...] Five workers would have bought about 1.8x for 43GB of disk and a five-minute build each, on that day's suite; nothing has re-taken it since the schema change above." | kept; dated from `git log -S`, ffa6f83c |

Left as they were, on purpose: every paragraph 08-02 dated in its own sentence (`:424`, `:461`, `:511`, `:566`, `:581`, `:609`, `:640`); the three thread-curve paragraphs, each dated; `:286`'s "two targets that build a live window", checked and true of the documents-only list.

### `docs/IMPLEMENTATION_STATUS.md`

| Before | After |
|---|---|
| `:228` "Run it with `scripts/mutants.sh <dir>`. A whole-tree run is about two days, so it is used scoped. The table below is from 2026-07-26." | `:228` "Run it with `scripts/mutants.sh <dir>`. A whole-tree run costs a rate times a count. The count is a row on `docs/development/measurements.md`: `cargo mutants --list` answered 12,335 mutants over 247 files on 2026-09-14. The rate is measured on one shard, under the suite shape the run will use, before any whole run is scheduled, and the product is written on that page beside the count. This paragraph put a whole-tree run at two days from 2026-07-29 until 2026-09-14, with no date, no machine and no thread setting, so nothing in it could be re-taken. So it is used scoped. The table below is from 2026-07-26." |

The test-count paragraph at `:171` already quotes the row and is a guard record's locator; not touched.

### The comments

| File | Before | After |
|---|---|---|
| `tests/house_style.rs:5365` | "A full run of the record is 683 builds and 683 suite runs, which is eighteen hours at the 95 seconds a record measured on 2026-09-10, and somebody told only that a record may be stale has no cheaper option than that." | "A full run of the record is one build and one suite run per record, and its cost is a rate times a count with both terms on `docs/development/measurements.md`, where the product is a row of its own; it is hours, and somebody told only that a record may be stale has no cheaper option than that. [...] Until 2026-09-14 this comment gave the product as a figure, 683 records at the 95 seconds a record measured on 2026-09-10, and it was that day's." |
| `guards/guards.toml:40` | "because a full run is an hour or two and nothing forces one." | "because a full run costs a build and a suite run per record, a rate times a count that docs/development/measurements.md holds as a product row and this file does not, and nothing forces one. [...] This comment put a full run at one or two hours from the day it was written, 2026-08-12, at about 192 records, until 2026-09-14." |
| `guards/guards.toml:3563` | "and a full run is an hour or two that nothing forces." | "and a full run is hours that nothing forces; how many is the product row on docs/development/measurements.md, not a figure here." |
| `scripts/guards.sh:36` | "an hour or two for all of them rather than seconds," | "hours for all of them rather than seconds, [...] How many hours is a rate times a count, and both terms are rows on docs/development/measurements.md with the product beside them; the runner prints the rate after every record as a `timed:` line. This comment put the whole run at one or two hours from 2026-08-08, when it was written, until 2026-09-14." |
| `scripts/guards.py:721` | "The arithmetic looked good: the rebuild a break forces is 23 seconds and the library is 89, so filtering would take a 220-record sweep from 6.8 hours to about 88 minutes." | "The arithmetic looked good on 2026-09-02, when this was measured: the rebuild a break forced was 23 seconds and the library was 89, so filtering would have taken that day's 220-record sweep from 6.8 hours to about 88 minutes. Both terms have moved since and the runner now prints today's pair after every record; the rate row on `docs/development/measurements.md` is where the current pair is, and the argument below does not depend on the figures." |

The dates: `guards.toml:40` from e5fdc293 of 2026-08-12; `guards.sh:36` from 115bacf3 of 2026-08-08; `house_style.rs` from 20c42110 of 2026-09-10; `guards.py` from c5cf0a37 of 2026-09-02; each by `git log -S` on the phrase.

### `.cargo/audit.toml`

The four accepted advisories and when each entered the list, by `git log --format='%h %ad' --date=short -S'"<id>"' -- .cargo/audit.toml | tail -1`:

| Advisory | Commit | Date | What was added |
|---|---|---|---|
| RUSTSEC-2026-0194 | 062cab91 | 2026-07-26 | "Accepted on 2026-07-26, the day this file was written; on 2026-09-14 Cargo.lock still resolves wxdragon-macros 0.9.17 over quick-xml 0.38.4, so the condition has not been met." |
| RUSTSEC-2026-0195 | 062cab91 | 2026-07-26 | shares the entry above |
| RUSTSEC-2024-0436 | 062cab91 | 2026-07-26 | "Accepted on 2026-07-26 with the two above. `cargo tree -i paste` on 2026-09-14 names two roots, wxdragon 0.9.17 and the boa crates behind a dev-dependency, so it clears when both stop pulling paste in, and neither has since it was accepted." |
| RUSTSEC-2023-0071 | 0096e925 | 2026-09-13 | nothing; its entry already says "Decided on 2026-09-13" |

`scripts/audit.sh` afterwards: "All 4 advisory(ies) this project accepts are still reported."

### `.planning/REQUIREMENTS.md`

Each PERF evidence line keeps its 2026-09-04 text and gains a paragraph opening "Re-read 2026-09-14 at `7da68e78`" or "Re-taken 2026-09-14 at `7da68e78`":

| Requirement | Figure or claim of 2026-09-04 | Today, with the command |
|---|---|---|
| PERF-01 | "nothing in `src/` reads resident memory, so no target below has a number attached" | `tests/the_numbers_the_targets_ask_for.rs` and the rows on the page, dated 2026-09-14 |
| PERF-02 | "Nothing in `src/` times process start against a usable list" | `src/common/started.rs` and the usable line; the cold-start row |
| PERF-03 | `SAMPLE_MAILBOX_SIZE` at `wx_app.rs:9125`, `sample_mailbox` at `9137`, the callback at `:1101`, the comment at `:1093`; "Nothing in the tree records a sort, filter or scroll timing"; "no test asserts it" | names in `src/presentation/sample_mailbox.rs` and `virtual_rows.rs`, `grep -n ID_LOAD_SCALE_SAMPLE src/presentation/wx_app.rs`; `tests/the_list_at_two_hundred_thousand_rows.rs` and eighteen rows; `tests/the_list_reads_only_memory.rs` and two records. `grep -n "wx_app.rs:[0-9]"` over the PERF block finds nothing |
| PERF-04 | "no measurement anywhere" | the idle row and the empty-profile floor |
| PERF-05 | 60.4% "still recorded that way at `docs/IMPLEMENTATION_STATUS.md:201`"; 1,195 of 1,373 commits since | 83.34% at `55464a5e`, the page row, found by `grep -n 83.34 docs/IMPLEMENTATION_STATUS.md`; `git rev-list --count --since="2026-07-26" HEAD` 1,830 of `git rev-list --count HEAD` 2,077, 88% |
| PERF-05 `[S]` | "low coverage in `service/protocols`, `service/oauth` and the provider clients is the network transport that has never met a live account" | kept, then "**That was true of the transport on 2026-07-26 and is not the reason on 2026-09-14.**" with the three areas' figures, everything outside `src/presentation/` at 95.53%, the 27 window files at 26.88% holding 73% of the missed lines, "a description of where the missed lines are and not an attribution", ledgers 452 and 453 named, and "The transport's own reason still stands for the transport" |
| PERF-06 | `--lib` 6,079 and `--all-targets` 6,271; `IMPLEMENTATION_STATUS.md:123`, `changelog.md:1276`, `integration-guide.md:5`; "about two days" at `:156` and `CLAUDE.md:323`; "about 15 hours" at `CLAUDE.md:465` | 7,264 and 7,750 by the same commands; the three pages quote the 7,697 row and `test_the_three_pages_that_state_the_test_count_quote_one_row` holds them; each citation replaced by the grep that finds it; both durations gone from the pages since 2026-09-14 with the row each points at |
| PERF-07 | 565 records by `grep -c "^\[\[guard\]\]"`; `CLAUDE.md:474` says 564; the 2026-08-05 run "at `CLAUDE.md:544`"; "192 hand-verified" has no counterpart | 798 by the parser, and the grep answers the same 798 today because every record opens with `[[guard]]`; the census 192 + 606 held by `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`; `grep -n "Count records" CLAUDE.md` and `grep -n 2026-08-05 CLAUDE.md`; 12,335 mutants |

No box was ticked. The 7,750 taken today is not on the page: 08-02's 7,697 row is dated today, the page refuses a second row with one `what` and one date, and the evidence line says so.

### `.planning/PROJECT.md`

Every figure re-taken at `7da68e78` with its command, the 2026-08-29 figure kept in parentheses:

| Figure | 2026-08-29 | 2026-09-14 | Command |
|---|---|---|---|
| lines under `src/` | 259,723 | 361,450 over 290 files | `find src -name '*.rs' \| xargs wc -l \| tail -1`; `find src -name '*.rs' \| wc -l` |
| tests | 5,430 | 7,750 every target, 7,264 library | `cargo test --all-targets -- --list` summed; `cargo test --lib -- --list \| tail -1` |
| guard records | 501 | 798 | the parser |
| line coverage | 60.4% on 2026-07-26 | 83.34% at `55464a5e` | `cargo llvm-cov --lib --summary-only` |
| `caldav.rs` | 8,149 | 8,337 | `wc -l src/service/caldav.rs` |
| `*_sync.rs` | "over 38,000 across five" | 36,328 across eight, named | `find src -name '*_sync.rs' \| xargs wc -l \| tail -1` |
| `wx_app.rs` | 19,659 | 30,030 | `wc -l src/presentation/wx_app.rs` |

The 259,723 at `:123`, inside the sentence about `CONCERNS.md`, is dated to 2026-08-29 with today's figure beside it; the 8,149 at `:57` the same.

### `.planning/STATE.md`

`:1135` "Version `0.112.0`, `guards/guards.toml` holds 720 records with the census reading 192 and 528, `.planning/WINDOWS.md` reaches 301, and nothing is pushed." became "held 720 records with the census reading 192 and 528 on 2026-09-11 at `3bc2651`, the day this section was written, `.planning/WINDOWS.md` reached 301, and nothing was pushed. The count on the day you read this is the record row on `docs/development/measurements.md`, by the parser it names." The plan put this sentence "in the part headed as history"; it is under "Phase 05.2, the phase before this one" above the history heading, so it was dated rather than left. Nothing under "History below this line" changed.

## What the awk finding became

The plan's first draft, the checker said, wrote a record count as the awk's output, which the awk never produces. The finding written is the checker's: the awk counts records naming a given file; it agrees with the parser for `wx_app.rs`, `house_style.rs` and `contacts_sync.rs` and for every file no inline-table record names, and undercounts by at most one for each file the four inline records name, which today is one file, `wx_send_later.rs`, 0 against 1. Re-measured at `3accd6e1` before writing, with the awk as it stood in `CLAUDE.md` and the file name substituted:

```
wx_app.rs              awk=50 reader=50
house_style.rs         awk=21 reader=21
contacts_sync.rs       awk=77 reader=77
wx_send_later.rs       awk=0  reader=1
attaching.rs           awk=2  reader=2
answered_meetings.rs   awk=2  reader=2
```

`wx_app.rs` is 50 where the plan's table said 48: 08-03 and 08-04 each landed a record naming it after the table was taken. The research's mechanism is stated in `CLAUDE.md` as the mechanism, and the observation that `attaching.rs` and `answered_meetings.rs` come out level is not restated there, because the plan marks it an inference from the code and not a measurement per record.

## What the tree contradicted in the plan

1. **The plan's file list left out `scripts/guards.py`, and ledger 443 had assigned it to this plan.** 08-01's summary ledgered the docstring of `run_the_whole_suite` as "a fifth sweep figure for 08-06 to point at the page". Task 2's done criterion is that no comment in the tree states the sweep's cost as a figure, so it was corrected here and 443 closed. Deviation 1.
2. **Three acceptance criteria could not be met beside rule 1.** `grep "about two days" docs/IMPLEMENTATION_STATUS.md` and `grep "an hour or two|eighteen hours"` over the three source files must find nothing, and rule 1 says every replaced figure keeps the old figure as the figure of its date. A dated quotation satisfies the rule and fails the grep. The three sentences say "put a whole-tree run at two days" and "put a full run at one or two hours" rather than quoting the words; `CLAUDE.md`, whose criterion admitted a dated sentence, quotes all four phrases as written. The first draft of the `guards.toml:40` sentence quoted the phrase across a line break, which the single-line grep did not see; that was reworded rather than left, because passing a grep by wrapping is the miss the phase README warns about. Deviation 2, ledger 455.
3. **`STATE.md`'s 720 is not in the history part.** It is at `:1135` under "Phase 05.2, the phase before this one", above "History below this line" at `:1628`, in a present-tense sentence; dated rather than left.
4. **`wx_app.rs` names 50 records, not the 48 in the plan's table**, as above; nothing else in the table moved.
5. **The status page's 2026-09-04 citation for PERF-06, `docs/IMPLEMENTATION_STATUS.md:123`, and for PERF-05, `:201`, are both the test-count and coverage paragraphs that 08-02 and 08-05 rewrote**; each replaced by the grep that finds the paragraph now.
6. **The 2026-09-04 PERF-07 line said `CLAUDE.md:474` gave 564 records "and is one behind"**; that sentence in `CLAUDE.md` has said 617 on 2026-09-06 at `485030f` since 08-02 dated it, and the research's table gives the same 617 at that commit by the parser, so the evidence line points at the paragraph by grep rather than restating a line number.

## Task commits

Branch `four-figures-for-one-sweep-become-one-row` from `main` at `3accd6e1`.

1. **Task 1:** `1dce536c` docs(08-06), `CLAUDE.md`, the status page, the changelog entry. The gate answered `docs_only`: formatting, clippy, the four shell suites, `--lib help_page::` and `what_the_scans_can_judge::`, and the seven document-reading targets, `every_number_carries_its_command_and_its_date` 25 passed among them.
2. **Task 2:** `7da68e78` docs(08-06), the five comments and the advisories. The gate answered `affected`: formatting, clippy, the shell suites, `house_style` (70 passed, the two named record checks among them) and the guards that read the whole tree; the rest of the suite and the release build did not run.
3. **Task 3:** `ee40346c` docs(08-06), the three planning files, `git diff --stat` 3 files, 138 insertions and 27 deletions. The gate answered `docs_only`, as task 1, with `the_planning_files_agree_with_themselves` 16 passed.
4. **Merge:** `53b9300f`, the whole gate on `main`, 7,746 passed and none failed.

**Plan metadata:** the commit carrying this summary, `STATE.md`, `ROADMAP.md` and `WINDOWS.md`.

Every commit went through `git commit` or `git merge` with the hook running; nothing was piped; no `.git/index.lock` was left. `scripts/check.sh all` on the branch was redirected to a file and its exit status read directly: 0, once, 320 s.

## Deviations from Plan

**1. `scripts/guards.py` edited outside the plan's file list.** Under contradiction 1. A docstring, not a doctest example; `python -m doctest scripts/guards.py` clean before and after, and `house_style`'s doctest runner green on the commit. Ledger 454.

**2. Three dated quotations reworded to keep the figure and drop the phrase.** Under contradiction 2. Rewording to satisfy a reading is what `CLAUDE.md` warns about; it is accepted because each sentence still gives the old figure and its dates, and it is recorded rather than absorbed. Ledger 455.

Otherwise the plan was executed as written. No package installed, `Cargo.toml` untouched, no version bump, no test added or removed anywhere, no record's `before`, `after` or `red` changed, no `.planning/phases/` or `.planning/intel/` file changed, no tracked file edited by anything but the editing tool, carriage returns measured with `tr -cd '\r' | wc -c` on every file before and after, none; no em dash, `test_no_dashes_that_should_be_punctuation` run before each commit.

## Guard records

None added, removed or re-measured. Before editing, every record whose `file` is one this plan touches was listed with its `before` text by the parser: nine break `tests/house_style.rs`, one breaks `CLAUDE.md` at the 01-02 writer-record sentence (`:425`, not touched) and one breaks `docs/IMPLEMENTATION_STATUS.md` at the test-count line (`:171`, not touched). None of the nine `house_style` locators is in the doc comment at `:5362-5371`, and the record reading the `guards.toml` header breaks `" records were swept that day."` at `:79`, which this plan did not touch. `test_every_guard_record_still_names_one_place_in_the_tree` passed on the task 2 commit and on the merge: "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out". `tests/house_style.rs` holds 70 tests before and after by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`, so `test_every_guard_record_says_how_many_tests_the_files_it_names_held` printed no remedy on any commit. 798 records by the parser before and after; census 192 + 606.

## Ledger

`.planning/WINDOWS.md` 453 before, 455 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c "| 454 |\|| 455 |"` and `grep -c '"id": 454\|"id": 455'`, two and two; no backslash in the added lines, `grep -cF '\'` over the diff's added lines 0; no carriage return.

- 443 closed with `gsd-tools windows fixed 443`, both halves now `fixed`.
- 454, deviation, `scripts/guards.py`: edited outside the plan's file list on ledger 443's assignment.
- 455, deviation, `docs/IMPLEMENTATION_STATUS.md`: three dated quotations reworded to satisfy an absence grep that the plan's own rule 1 contradicted.
- 452 stays open: its evidence-line half is done here and its criterion-3 half is 08-09's, which the entry says.

## What was deliberately left, and why

Everything under `.planning/phases/` and `.planning/intel/`, including the three intel files that still say "roughly half of WCAG" and the two validation records carrying "commit 182 of 344": records of their day, by the README's Assumption 2. `.planning/ROADMAP.md`'s criterion 3, "Low coverage is attributed to the untested network transport", is 08-09's to revise under criterion 6 and is left as written; the evidence line it reads from now says why. `CLAUDE.md`'s three thread-curve paragraphs and every paragraph 08-02 dated in its own sentence.

## Issues Encountered

None that stopped anything. The gate passed on every commit through the hook; `scripts/check.sh all` passed on its first run, so ledger 374's keyring race was not met.

## Known Stubs

None. Every corrected sentence is on its page; the provenance reading accepts the two pages; the record checks accept the comments; the planning-file checks accept the three files.

## Threat Flags

None. No network endpoint, auth path, file access pattern or schema. T-08-SC: no package added; `cargo tree` was read, not changed.

## Self-Check: PASSED

The eleven modified files exist on disk and hold the text described, checked by `grep` for `tomllib` in `CLAUDE.md`, `12,335` in the status page, `2026-09-14` in each of the five comment sites and the advisory file, `7da68e78` in `REQUIREMENTS.md` and `PROJECT.md` and `3bc2651` in `STATE.md`'s dated sentence; the four commits `1dce536c`, `7da68e78`, `ee40346c` and `53b9300f` are in `git log --all`, checked by `grep` before this section was written. `main` is ahead of `origin/main` and nothing was pushed.

## Next Phase Readiness

08-07 starts from `main` at this plan's metadata commit with 798 records and the census unchanged; its checkpoint quotes the product row, and `CLAUDE.md` now sends a reader there rather than to a figure. 08-09 has PERF-05's `[S]` line dated and added to, criterion 3's wording still to revise, and the seven evidence lines re-taken at `7da68e78` to read the clauses against.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-14*
