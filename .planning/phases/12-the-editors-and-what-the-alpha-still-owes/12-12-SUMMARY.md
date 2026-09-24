---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 12
subsystem: the alpha page, the guide, the listening page, the changelog, the backlog, the phase's closing read, the ledger, the full gate
tags: [closing-read, listening-lines, full-gate, found-23, carddav]
status: complete

requires:
  - phase: 12-the-editors-and-what-the-alpha-still-owes
    provides: "fifteen plans merged, 12-11 last at 3791bc9d; the requirement lines each plan ticked; the ledger entries each opened"
provides:
  - "docs/ALPHA_TESTING.md: a short-version paragraph on what phase 12 changed, ten known-missing entries, three new items in what would help most"
  - "docs/manual-accessibility-pass.md: items 84 to 95, one per ledger entry the phase opened for an ear, an account or a mailbox; the count line and phase 11's paragraph corrected by dating"
  - "docs/changelog.md: 11-11.1's 'the NVDA case has not yet run' corrected by dating"
  - "docs/development/requirements-backlog.md: a CardDAV row saying what is built, and the CalDAV row's 'CardDAV not built' corrected by dating"
  - ".planning/REQUIREMENTS.md: the closing read's paragraph, LIST-19's replaced test name, the FOUND-20 and FOUND-22 rows"
  - ".planning/ROADMAP.md: a dated closing sentence on each of the twelve criteria, the plan list, the row at 16/16"
  - ".planning/STATE.md and .planning/WINDOWS.md: plan 16 of 16, ledger 609"
affects: [phase 13, which is planned when phase 12 closes]

actuals:
  # chars/4 over the added lines of this commit, 32,961 characters, leaving out three lines
  # the diff counts whole though a sentence was added to each: STATE's stopped_at (150,864),
  # its Current focus (19,689) and the roadmap's row (16,002); about 2,000 characters were added to them
  tokens: 8700
  tasks: 3
  commits: 2

tech-stack:
  added: []
  patterns:
    - "Where a phase's later summaries carry no coverage block, the closing read's one pass reads the requirement lines themselves: every backticked test name looked for as fn <name>, every backticked path looked for on disk"

key-files:
  created:
    - .planning/phases/12-the-editors-and-what-the-alpha-still-owes/12-12-SUMMARY.md
  modified:
    - docs/ALPHA_TESTING.md
    - docs/manual-accessibility-pass.md
    - docs/changelog.md
    - docs/development/requirements-backlog.md
    - docs/development/measurements.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/WINDOWS.md

key-decisions:
  - "Fourteen requirements read, not eleven: FOUND-21, FOUND-22 and FOUND-23 came with the three inserted plans"
  - "The listening page gains twelve items, not thirteen: ledger 588, the two pages answering, is Pratik's site and not a tester's ear, account or mailbox"
  - "The gate's result goes in a second documents commit, because the one documents commit the brief asks for comes before a run whose result it cannot hold"

metrics:
  duration: "about 2 hours on 2026-09-24"
  completed: 2026-09-24
---

# Phase 12 Plan 12: The pages, the listening lines and the closing read Summary

**Phase 12 is closed: the full gate was green on its first run, the pages describe the program the phase built, and every tick stands on a name in the tree.** `scripts/check.sh all` at `566116d3` exited 0 after 516 s with 8,985 tests passed. All fourteen of the phase's requirements stand, FOUND-23's ticked on that run.

## For a person

What changed in this round over the build the tester has: every number in Settings and the account editor is a spin control whose typing field is named; the contact editor has prefix, middle name and suffix, fills a name and its parts from each other, has a birthday as a date and reads a phone number by its country; event and reminder times move in blocks; signatures are per account with one default; labels have an order the menu and the keys share; About names its owners and links to the two pages; Send Feedback sends a report you have read; a link can open in a separate window; the status bar reads to one shape. Issues closed by the phase's plans: #35, #40, #41, #43, #48, #64, #71, #73, #75, #78 and #80, each read closed by `gh issue view` on 2026-09-24; #65, the licence, stays open on the ten decisions that are Pratik's. What only a person can settle is items 84 to 95 on the listening page, a report reaching each mailbox, five name parts through a real account, the two pages answering (ledger 588) and the push of `main` (ledger 609). Next: phase 13, the sixth of the seven groups, planned once this phase closes; Pratik confirms the order inside it first.

## The one-pass check

Command: `python one_pass.py` in the session scratchpad, read-only, over the five coverage blocks and the fourteen requirement blocks, at `3791bc9d`.

```
PART 1: refs 83, path#name 39, found 39, not found 0, commands or readings 44
PART 2: test names 30, not found 1; paths 33, not found 2
  NAME NOT FOUND ('LIST-19', 'test_the_page_process_names_its_profile_before_it_builds_a_browser')
  PATH NOT FOUND ('FOUND-20', 'tests/a-link-opens-where-the-setting-says.test.js')
  PATH NOT FOUND ('ALPHA-03', 'docs/plans/20260920-pro-licence.md')
ONE PASS COMPLETE
```

The name was replaced by 12-02 in `a1774d9e` (its deviation 9) with `page_window::tests::test_the_profile_name_is_set_before_the_browser_is_built` and `test_the_page_process_is_the_only_one_that_renames_itself`; LIST-19's line says so. The first path is under `nvda-tests/`; the second is the planned name of `docs/plans/20260924-pro-licence.md`, which ALPHA-03's own line records. Only 12-01 to 12-03.1 carry a coverage block; 12-03.2 to 12-11 wrote shorter summaries under the brief rules of 2026-09-23, so part 2 is what stands in for their blocks.

## The requirements

| Requirement | Plan, merge | Read |
|---|---|---|
| FOUND-20 | 12-01 `e29c514b`, 12-03.1 `8f57f80e` | Stands. Handler and readings found; NVDA run 35876075636 green; ear 578 |
| FOUND-21 | 12-02.1 `a503ce77` | Stands. `[S]` waits for the next sweep |
| FOUND-22 | 12-03.1 `8f57f80e` | Stands. Runner green; row corrected, 576 fixed |
| FOUND-23 | 12-03.2 `b3cc380a`, 12-12 | Stands. Last `[D]` line met by this plan's full gate; 585 fixed |
| LIST-11 | 12-03 `a9ce329d` | Stands. Ear 574 |
| LIST-19 | 11-11.1 `8340e5e6`, 12-02 `70f5435a` | Stands, one name corrected. Ear 568 |
| ALPHA-01 | 12-04 `2e00c42b` | Stands. Ear 587, site 588 |
| ALPHA-02 | 12-05 `6562e753` | Stands. Ear 589, mailboxes 590 |
| ALPHA-03 | 12-11 `3791bc9d` | Stands. Decisions 608 |
| EDIT-01 | 12-06 `01dfbe28`, 12-06.1 `8098b4b1` | Stands. Scan run 35976411245; ear 592 |
| EDIT-02 | 12-07 `dc768391` | Stands. Ear 596, account 597 |
| EDIT-03 | 12-08 `ab3e14c3` | Stands; a task has no time. Ear 603 |
| EDIT-04 | 12-09 `ae04f292` | Stands. Ear 605 |
| EDIT-05 | 12-10 `015035f7` | Stands. Ear 607 |

`grep -c '^- \[x\] \*\*\(ALPHA\|EDIT\)-' .planning/REQUIREMENTS.md` answers 8 and the unticked form 0. The coverage count, `grep -c '^- \[[ x]\] \*\*[A-Z]\+-[0-9]\+\*\*'`, answers 122, with 122 traceability rows, unchanged.

## The four marks

The roadmap's plan list has 16 ticked lines of 16 and the row reads 16/16; `STATE.md` has `current_plan: 16`, `Current Plan: 16`, `Total Plans in Phase: 16`, and `completed_plans: 172` of `total_plans: 172`, from `ls .planning/phases/*/*-SUMMARY.md | wc -l` and the same over `*-PLAN.md` once this file exists. The requirement marks: FOUND-23's last `[D]` line, its box and its row, in the second commit, after the gate. The phase line is ticked and the row reads Complete from the same commit.

## The ledger

`head -7 .planning/WINDOWS.md` at the end: 550 open, 59 fixed, 609 in all, agreeing with the table and the JSON by `the_planning_files_agree_with_themselves`. One entry opened, 609, the push of `main`; one fixed, 585, the full gate.

## The pages

- `docs/manual-accessibility-pass.md`: the count of section A's items, 95, is 83 plus the twelve entries 568, 574, 578, 587, 589, 590, 592, 596, 597, 603, 605 and 607.
- `docs/ALPHA_TESTING.md`: `grep -c -i 'send feedback'` and `grep -c 'separate'` each at least 1.
- The user guide was read against the tree for the sentences the plans wrote: About's controls and letters, the Help items, the spin-control ranges, the contact editor, times, signatures, labels and the status sentences it quotes. None was false, so the guide is unchanged.
- The changelog: one sentence falsified by a later plan, 11-11.1's "the NVDA case has not yet run", corrected by dating.
- The requirements backlog, carried from 12-11: CardDAV is built and never met a server. The row names the screen, the client's four requests, the sync through the contacts merge on every contacts sync, and the credential store.

## The full gate

`git rev-list HEAD..main --count` read 0 immediately before the run. Then, with nothing else building:

```
scripts/check.sh all > <scratchpad>/12-12-full-gate.log 2>&1; echo "exit=$?"
exit=0
check.sh: all passed after 516 s: start 1 s, rustfmt 3 s, clippy 29 s, the scripts that decide what runs 103 s, security advisories 6 s, tests 289 s, release build 85 s
```

At `566116d3`, 19:50:21Z to 19:58:57Z on 2026-09-24, warm, NVDA running for the tester. Summed over its 111 `test result:` lines: 8,985 passed, 0 failed, 12 ignored, against 8,693 on 12-03's gate the day before. The audit: all 5 accepted advisories still reported, nothing outside `.cargo/audit.toml`. Green on the first run, so no red and green pair was needed. The run covers five of CI's seven jobs: no debug build, no setup executable, no search handler checks, and no NVDA case or accessibility scan, which is ledger 609. The row is on `docs/development/measurements.md`.

## Deviations from Plan

1. **Fourteen requirements, not eleven.** The plan named eleven; the three inserted plans brought FOUND-21, FOUND-22 and FOUND-23, and all fourteen were read.
2. **The one-pass check read the requirement lines as well as the coverage blocks**, because nine of the fourteen plans' summaries carry no coverage block.
3. **The gate's result is a second documents commit.** The brief asks for one documents commit before the gate; a measurements row, ledger 585 closed and FOUND-23 ticked can only be written after the run. The second commit touches documents only, so the code the gate read is the code that merges.
4. **The FOUND-20 and FOUND-22 traceability rows corrected** for ledger 567 and 576, which were fixed on 2026-09-23 while the rows still named them as waiting.

## Threat model

T-12-40: every page sentence written was checked against the tree with a command. T-12-41: no tick written on a name not in the tree; the one pass above. T-12-SC: nothing installed.

## Self-Check: PASSED

Every file under key-files exists; the commits are named in the report that closes this plan.
