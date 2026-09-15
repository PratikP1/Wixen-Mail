---
phase: 08-every-number-the-project-quotes
plan: 04
status: complete
subsystem: testing
tags: [measurements, performance, message-list, virtual-list, sort, filter, scroll, guards, red-green]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "`docs/development/measurements.md` and the reading that refuses a row without its command, date and commit, which the eighteen rows this plan wrote had to pass"
  - phase: 08-every-number-the-project-quotes
    plan: 03
    provides: "the harness shape for a measurement behind `#[ignore]` that refuses a debug build and prints rows, and the definitions section on the page"
provides:
  - "`src/presentation/virtual_rows.rs`: `text_for` over the view and both row slices, the visible columns, the row and column as wxWidgets hands them, the date settings and the moment of the paint; the message list's closure reduced to lock, borrow, call"
  - "`tests/the_list_reads_only_memory.rs`, 6 tests: a reading that holds the function and both closures to naming no database, the message list's closure to calling `text_for(`, and the window to exactly two registrations, with three companions over the real files"
  - "`src/presentation/sample_mailbox.rs` and `src/presentation/mail_sort.rs`: the generator and the sort, moved out of `wx_app.rs` where tests outside the wx layer and `cargo mutants` can reach them; the Help menu path unchanged"
  - "`tests/the_list_at_two_hundred_thousand_rows.rs`, 6 tests: the harness that writes 200,000 rows into a `tempfile` cache and times the listing, the filter, the seven sorts and the paint with no window; a format-holding half at 2,000 rows on every commit"
  - "Eighteen rows on `docs/development/measurements.md`, every value under a second, with the three takes behind each median and what was and was not timed"
  - "`guards/guards.toml`: three records added and two re-pointed, each measured; census 606, 798 records"
affects: [08-08, 08-09]

actuals:
  tokens: 24000
  tasks: 3
  commits: 12

tech-stack:
  added: []
  patterns:
    - "A paint callback whose whole body is one call to a function over slices and copies, with a source reading in `tests/` holding the closure to that call and both to naming no database, because the type stops a connection being passed and the reading stops one being opened"
    - "A scale harness that turns the product's own generator rows into `IncomingMessage` through one conversion, so the cache rows and the in-memory rows are the same rows by construction"
    - "A source-reading test that follows an extracted call into the new module rather than being deleted, so a file named by many guard records keeps its test count"

key-files:
  created:
    - src/presentation/virtual_rows.rs
    - src/presentation/sample_mailbox.rs
    - src/presentation/mail_sort.rs
    - tests/the_list_reads_only_memory.rs
    - tests/the_list_at_two_hundred_thousand_rows.rs
  modified:
    - src/presentation/mod.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/development/measurements.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

key-decisions:
  - "`text_for` takes the view and both row slices in one `Listed` struct, six parameters in all, because the plan's eight-parameter signature trips clippy's `too_many_arguments` under `-D warnings`"
  - "`text_for` takes `row: i64` and `column: i32`, the types wxWidgets hands the callback, and converts inside, so a negative index reads as past the end rather than wrapping; the plan wrote `u32` for both"
  - "The reading in `wx_app.rs` that looked for the conversation cell inside the closure follows the call into `virtual_rows.rs` under its own name rather than being deleted, so the window keeps 199 tests and the 49 records naming it are not flagged"
  - "The generator's tests pin what it does, one day read a minute a row and wrapping after 1,440, not the descending dates its comment claimed; the comment is corrected and the code is unchanged"
  - "The harness saves no account row, because `save_account` reaches the credential store and the folder's `account_id` is all the listing and the search read"
  - "The filter is timed at the search box's limit of 500 and once with no limit, so the page says both what the box costs and what every match costs"
  - "The harness record breaks the row's column count, because the batch-size break the plan offered first reddens nothing: chunks of any size write every row"

patterns-established:
  - "Before moving code, grep the tree's tests for the moved identifiers as text: a source-reading test pins the shape rather than the behaviour and goes red after every behaviour test is green"

requirements-completed: [PERF-03]

coverage:
  - id: D1
    description: "The text the message list paints for a row comes from a function whose inputs are slices and copies, the closure only locks, borrows and calls it, and a reading holds both to naming no database with companions that plant a violation and see it named"
    requirement: PERF-03
    verification:
      - kind: unit
        ref: "src/presentation/virtual_rows.rs#test_a_message_row_gives_its_cell_text"
        status: pass
      - kind: unit
        ref: "src/presentation/virtual_rows.rs#test_a_row_past_the_end_is_the_placeholder_in_both_views"
        status: pass
      - kind: integration
        ref: "tests/the_list_reads_only_memory.rs#test_the_message_list_paints_from_memory_and_its_closure_only_calls_the_function"
        status: pass
      - kind: integration
        ref: "tests/the_list_reads_only_memory.rs#test_a_function_that_opened_the_cache_would_be_named"
        status: pass
      - kind: integration
        ref: "tests/the_list_reads_only_memory.rs#test_a_closure_doing_its_own_work_would_be_named"
        status: pass
      - kind: integration
        ref: "tests/the_list_reads_only_memory.rs#test_a_third_registration_would_be_counted"
        status: pass
    human_judgment: false
  - id: D2
    description: "The generator and the sort live in modules a test outside the wx layer can reach, the two sort records followed the sort, and the Help menu path that loads the sample is unchanged"
    requirement: PERF-03
    verification:
      - kind: unit
        ref: "src/presentation/sample_mailbox.rs#test_the_count_asked_for_is_the_count_returned"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#test_each_way_of_sorting_the_list_puts_it_in_that_order"
        status: pass
      - kind: integration
        ref: "tests/the_list_at_two_hundred_thousand_rows.rs#test_the_help_menu_loads_the_sample_through_the_generator"
        status: pass
      - kind: other
        ref: "the release binary at 92155231's tree, the Help menu's command posted to the main window, the log quoted below"
        status: pass
    human_judgment: false
  - id: D3
    description: "The sort, the filter and the scroll each have a number over 200,000 rows on the page, taken by a harness in the tree with no window, and each row says what was timed and what was not"
    requirement: PERF-03
    verification:
      - kind: integration
        ref: "tests/the_list_at_two_hundred_thousand_rows.rs#test_every_row_the_measurement_prints_has_the_pages_shape_and_names_what_it_timed"
        status: pass
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit"
        status: pass
      - kind: other
        ref: "two release runs of the ignored measurement, 18 rows each, quoted below"
        status: pass
    human_judgment: false
  - id: D4
    description: "No criterion claims a real provider mailbox was used; every row's conditions say synthetic"
    requirement: PERF-03
    verification:
      - kind: integration
        ref: "tests/the_list_at_two_hundred_thousand_rows.rs#test_every_row_the_measurement_prints_has_the_pages_shape_and_names_what_it_timed"
        status: pass
    human_judgment: false

duration: 100min
completed: 2026-09-15
---

# Phase 8 Plan 04: The list paints from memory and says how fast Summary

**What the message list paints for a row now comes from `virtual_rows::text_for`, a function whose inputs are slices and copies, so it cannot reach the connection the program holds; the closure wxWidgets calls does nothing but lock, borrow and call it; and a reading on every commit holds both to naming no database, shown red first with three companions that plant a violation and see it named. The generator and the sort have modules of their own, and a harness in the tree times the list over 200,000 rows with no window: the listing 351 ms, a filter 78 to 110 ms at the search box's limit, the seven sorts 61 to 260 ms, one page's paint 0.09 ms. Every value is under a second; every row says the rows are synthetic and what it did not time.**

Every count below is one this session took, with the command. Where the tree contradicted the plan, the tree won and the difference is named.

## Performance

- **Duration:** about 100 minutes, from the first read at about 00:05Z on 2026-09-15 to the merge at 01:44Z; the summary followed
- **Started:** 2026-09-15T00:05:00Z (approximate; the first commit is 00:33Z)
- **Completed:** 2026-09-15T01:44:00Z for the merge at `6d08c94e`
- **Tasks:** 3 of 3
- **Files modified:** 11 on the branch, plus the three planning files on `main`

## The numbers, with the runs behind them

Taken 2026-09-14 local against the release binary at `5cf04528`, version 0.125.0, built by `cargo build --release` (72 s) on the branch; AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, as `Get-CimInstance Win32_Processor` and `Win32_ComputerSystem` report it. `tasklist` was checked for `cargo.exe` and `rustc.exe` before each run and found none; the test binary runs one ignored test, so `WIXEN_TEST_THREADS` is unset and not in play. The command: `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture`, 123 s wall each run, 48 s inside the test. Two runs were taken: the first at `cd0f199b`, the second at `5cf04528` after a one-line wording fix to the row's conditions; the second is on the page and the first is beside it here. Each timing is three takes and the median is the value, except the cold listing, which is one take by definition.

| What | Median at `5cf04528` | The three takes | At `cd0f199b` |
|---|---|---|---|
| Listing 200,000 rows, cold | 351.44 ms | one take | 334.31 ms |
| Listing 200,000 rows, warm | 370.53 ms | 370.18, 370.53, 379.10 | 375.32 ms |
| Filter `quarterly`, one subject in five, All Folders, limit 500 | 78.24 ms | 78.24, 77.85, 80.13 | 78.19 ms |
| Filter `quarterly`, Subject Only, limit 500 | 78.91 ms | 80.90, 78.91, 77.75 | 78.90 ms |
| Filter `zebra`, in nothing, All Folders | 0.16 ms | 0.53, 0.16, 0.11 | 0.20 ms |
| Filter `zebra`, Subject Only | 0.13 ms | 0.16, 0.13, 0.09 | 0.11 ms |
| Filter `grace`, a sender in four, All Folders, limit 500 | 108.10 ms | 109.82, 108.10, 108.03 | 110.45 ms |
| Filter `grace`, Sender Only, limit 500 | 109.67 ms | 109.35, 111.34, 109.67 | 110.02 ms |
| Filter `quarterly`, every match, 40,000 rows | 192.55 ms | 192.55, 192.55, 195.29 | 192.35 ms |
| Sort Date (Newest First) | 126.58 ms | 126.58, 130.29, 125.24 | 121.99 ms |
| Sort Date (Oldest First) | 112.50 ms | 112.50, 113.62, 107.69 | 112.74 ms |
| Sort Sender (A-Z) | 259.50 ms | 257.55, 273.12, 259.50 | 262.98 ms |
| Sort Sender (Z-A) | 235.80 ms | 235.80, 237.86, 235.36 | 238.50 ms |
| Sort Subject (A-Z) | 224.42 ms | 223.46, 224.42, 225.88 | 222.51 ms |
| Sort Subject (Z-A) | 223.50 ms | 223.50, 218.02, 224.69 | 223.14 ms |
| Sort Unread First | 61.10 ms | 60.22, 61.10, 67.60 | 61.94 ms |
| Page paint, `text_for` over 40 rows and 6 columns | 0.09 ms | 0.38, 0.09, 0.09 | 0.12 ms |
| Full pass, `text_for` over 200,000 rows, one column | 28.94 ms | 30.80, 28.94, 24.13 | 24.04 ms |

Three things worth reading off that table. The warm listing is slower than the cold one in both runs, by 20 to 40 ms; "cold" here is the first read on a fresh connection over a file the same process had just written, so the operating system's cache was warm either way and the difference is noise around 350 ms, not a cache effect. The filters at the box's limit of 500 return their 500 rows in about the time it takes SQLite to find the first 500 of 40,000 or 50,000 matches, and the word in nothing answers in a tenth of a millisecond, so the index is doing the work; every match of the one-in-five word, 40,000 rows, costs 193 ms. The sender and subject sorts cost about twice the date sorts, which is the lowercase copy per comparison premise 7 named; at 260 ms for the worst order the `sort_by_cached_key` fix is not owed, and the changelog says so.

**Whether PERF-03's numbers meet a target:** the requirement sets none. Its `[S]` line says a multi-second freeze on a header click is an accessibility failure; the worst sort is 260 ms and runs off the interface thread, so no header click freezes for the sort itself. What the list control does with 200,000 rows handed back is not timed, and the page and ledger 449 say so.

## What the Help menu did with the moved generator

The release binary at the tree of `92155231`, started with `WIXEN_MAIL_DATA` at a scratch profile whose settings had `told_about_the_alpha` set, `--read-only`. Synthetic keystrokes from PowerShell reached nothing on two attempts and were refused with "Access is denied" on a third, so the menu item's own command, `WM_COMMAND` with `ID_LOAD_SCALE_SAMPLE`, which is `wxID_HIGHEST + 11` and is 6011, was posted to the main window, the routing a click on the item takes. The first guess of 6010 landed on Unpin Folder, which refused politely, and the log said which. At 6011:

```
Generating a sample mailbox of 200000 messages
Generated 200000 messages, sending to the list
Message list now holds 200000 rows
the message list is usable: 200000 rows, 8197 ms after start
Speaking: 200000 messages, 66667 unread topic=Some("messages")
```

The 8,197 ms is when the command was posted, not what the load cost. The handler's text is not edited by the move; `test_the_help_menu_loads_the_sample_through_the_generator` reads it on every commit and requires `sample_mailbox(SAMPLE_MAILBOX_SIZE)` and `UIUpdate::MessagesLoaded(` in the arm.

## What the tree contradicted in the plan

1. **The plan's `text_for` signature has eight parameters and clippy refuses it.** `too_many_arguments` fires above seven under `-D warnings`. The view and both row slices went into one `Listed` struct, six parameters; the reading still anchors on `text_for(`.
2. **The callback passes `row: i64, column: i32`, not `u32`.** The first green attempt did not compile. The function takes the types wxWidgets hands it and converts inside, so a negative index answers the placeholder or nothing rather than wrapping through `as usize`; a test says so and was written after the conversion in the same commit.
3. **A source reading in `wx_app.rs` pinned the old closure.** `test_the_paint_callback_asks_for_a_conversation_cell_when_rows_are_conversations` required `message_rows::conversation_cell_text(` in the shipping half of the window, and the gate refused the green commit on it. The plan's premise said no record breaks the callback, which is true, and no record is the only thing it checked. The reading now follows the call into `virtual_rows.rs` and requires both cell functions there, under the same name, so the window keeps 199 tests.
4. **The generator's dates do not descend.** The plan's behaviour said "dates descend" and the code's comment said "Descending so the newest is first". The formula is `(i / 60) % 24` hours and `i % 60` minutes on one day, which climbs a minute a row and repeats after 1,440 rows. The tests pin that, the comment says what the old one claimed, and the code is unchanged, because moving it is a refactor.
5. **Two registrations, as the plan said, and 49 records name `wx_app.rs`, not 48.** `python -c` over `tests_last_seen` answered 49 at `e9145821`, because 08-03's record names it; the prompt's 48 predates that record. 50 after this plan's closure record.
6. **The batch-size break for the harness record reddens nothing.** Chunks of 2,500 write every row as chunks of 5,000 do. The record breaks the row's column count instead, the plan's other candidate.
7. **The listing's "cold" is not cold.** The plan's word; the row and the page's definitions say what it is: the first read on a fresh connection over a file the process had just written.

## Task commits

Branch `the-list-paints-from-memory-and-says-how-fast` from `main` at `e9145821`.

1. **Task 1, red:** `6a2cd207` test(08-04), nine tests named: four in the module against a stub answering nothing, the reading and its four companions against a closure that did not yet call the function.
2. **Task 1, green:** `ba3b9df0` feat(08-04), the function, the closure, the i64/i32 conversion, and the window's reading following the call; refused once by the gate on that reading and recommitted.
3. **Task 1, records:** `55b556f1` test(08-04), two records, census 605.
4. **Task 2, red:** `14e92cdb` test(08-04), six tests named against a stub generator; the seventh holds the constant the stub already carried.
5. **Task 2, green:** `92155231` feat(08-04), the two modules, the two `use` lines, the two records re-pointed and re-measured, the comment corrected.
6. **Task 3, red:** `d2fb848b` test(08-04), two tests named against a writer that refuses and a measurement that prints nothing.
7. **Task 3, green:** `cd0f199b` feat(08-04), the writer, the measurement, the rows.
8. **Task 3, wording:** `5cf04528` feat(08-04), a row over one take says one take.
9. **Task 3, the page:** `1dbca9a5` docs(08-04), eighteen rows, the definitions section, the changelog entry.
10. **Task 3, record:** `3727bcc5` test(08-04), one record, census 606.
11. **Task 3, ledger:** `361bc629` docs(08-04), three entries.
12. **Merge:** `6d08c94e`, gate green on the merge.

**Plan metadata:** the commit carrying this summary, `STATE.md` and `ROADMAP.md`.

## What the gate selected per file

From the hook's own output on each commit. `src/presentation/virtual_rows.rs`: `presentation::virtual_rows`. `src/presentation/mod.rs`: `presentation`, the whole layer's unit tests. `src/presentation/wx_app.rs`: `presentation::wx_app`, 199 tests, plus the eight targets the records couple to it, and from the closure record on, `the_list_reads_only_memory` as a ninth: on the green commit of task 1 the new target did not run, because no record said to, and it ran on task 2's green commit through `55b556f1`'s second record. `src/presentation/sample_mailbox.rs`: `presentation::sample_mailbox`; `src/presentation/mail_sort.rs`: `presentation::mail_sort`. `tests/the_list_reads_only_memory.rs` and `tests/the_list_at_two_hundred_thousand_rows.rs`: each its own target and the tree guards. `guards/guards.toml`: the tree guards alone. `docs/development/measurements.md` with `docs/changelog.md`, and `.planning/WINDOWS.md` on its own, each answered `docs_only` and ran the document-reading targets. `check.sh --suites-for guards/guards.toml src/presentation/virtual_rows.rs` prints `the_list_reads_only_memory`, and for `src/presentation/wx_app.rs` prints it ninth after the eight already coupled.

`scripts/check.sh all` on the branch at `361bc629`: 7,746 passed, none failed, 4 ignored, 58 result lines, 319 s, exit read directly from a redirect. On the merge at `6d08c94e`: 7,746 passed, none failed. No keyring race in either.

## Deviations from Plan

**1. [Rule 3, blocking] `Listed` in place of three parameters.** Contradiction 1 above. Files: `src/presentation/virtual_rows.rs`, `src/presentation/wx_app.rs`; commit `ba3b9df0`.

**2. [Rule 1, bug] `i64` and `i32` in place of `u32`.** Contradiction 2 above; the plan's signature would not have compiled against the callback. Commit `ba3b9df0`.

**3. [Rule 3, blocking] The window's reading follows the call.** Contradiction 3 above. Commit `ba3b9df0`.

**4. The generator's tests pin the ascending dates and its comment is corrected.** Contradiction 4 above. Commit `92155231`. The code is byte for byte what it was.

**5. The Help menu item was invoked by posting its command rather than by a keypress.** Synthetic keystrokes were refused; the posted `WM_COMMAND` is what a click sends. The log is quoted above.

**6. The harness saves no account row.** The plan said one folder of one account; the folder row carries the account id and nothing else reads an account row, and `save_account` reaches the credential store that ledger 374 races. The harness's header says so.

**7. Seven filter rows rather than five.** Three words under two answers each is six, and one more with no limit so the page says what every match costs and how many there are.

**8. The negative-index test and the median test followed their code in the same commit.** Two small tests in a module and a target; said here rather than hidden.

**9. The harness record's break is the column count, not the batch size.** Contradiction 6 above.

Otherwise the plan was executed as written. No package installed; `Cargo.toml` untouched; no version bump, because nothing a person meets changed; no `unwrap` or `expect` outside the test targets and test modules; no tracked file edited by a script, and this executor's exception set was zero. `cargo fmt` ran before every commit. Carriage returns were measured with `tr -cd '\r' | wc -c` on every source and document touched: none.

## Guard records

Three added, each break applied by hand and measured on its target before it was written, then confirmed by `scripts/guards.sh --remeasure`; two re-pointed and re-measured on the whole library.

| Record | File | Suite | Break | Measured |
|---|---|---|---|---|
| the function the list paints with cannot open the cache | `src/presentation/virtual_rows.rs` | `the_list_reads_only_memory` | a call to `MessageCache::new` with an empty path planted in `text_for` | 1 passed, exactly 5 failed; runner 16 s |
| the message list's closure calls the function rather than painting for itself | `src/presentation/wx_app.rs` | `the_list_reads_only_memory` | the old dispatch put back inside the closure | 1 passed, exactly 5 failed; runner 21 s |
| the rows the scale harness prints keep the page's six columns | `tests/the_list_at_two_hundred_thousand_rows.rs` | `the_list_at_two_hundred_thousand_rows` | two cells merged with a semicolon | 4 passed, exactly 1 failed; runner 5 s |
| each way of sorting the message list really puts it in that order | re-pointed to `src/presentation/mail_sort.rs` | library | unchanged | the one test named went red and nothing else, rebuild 35 s, run 47 s |
| sorting by unread brings the unread ones to the top | re-pointed to `src/presentation/mail_sort.rs` | library | unchanged | the one test named went red and nothing else, rebuild 39 s, run 47 s |

The library was not run under the first record's break, on purpose: the planted call would open a cache at an empty path from inside every test that paints a cell, and the record's suite is the target. The runner wrote `mail_sort.rs` at 0 tests into the two re-pointed records; the count check had printed exactly that remedy after the move and nothing else, and it was run and read before the commit.

No test was added to `src/presentation/wx_app.rs`: 199 before and after, by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`, checked after every task. No record naming it was re-measured; the two that named it for the sort now name `mail_sort.rs`. `tests/house_style.rs` 70 before and after.

Records: 795 before by the TOML reader, 798 after. Census 192 + 606. Records naming `src/presentation/wx_app.rs` in `tests_last_seen`: 49 before, 50 after.

## Ledger

`.planning/WINDOWS.md` 448 before, 451 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c "| N |"` and `grep -c '"id": N'` for each, one and one; no backslash and no em dash in the added lines:

- 449, unrun-verify, `tests/the_list_at_two_hundred_thousand_rows.rs`: the sort's apply is not timed; a number for the list control taking 200,000 rows back needs a window.
- 450, unrun-verify, the same file: a scroll's own paint is not timed; the page paint row is `text_for` alone.
- 451, todo, the same file: `THE_SEARCH_BOXES_LIMIT` copies the private `LIMIT` in `managers::search_messages`; if the box's limit moves, the harness times the old one.

No entry for the sort fix, because no order came near a second.

## Issues Encountered

One commit was refused by the gate and recommitted, described under contradiction 3. Three attempts to drive the Help menu by keystroke reached nothing; the first of them also left the first release instance running because `taskkill /IM` under the MSYS shell had its flag rewritten as a path, so the second start handed over to the first and the keys went to a first-run dialog; the processes were stopped through PowerShell and the command was posted instead. `scripts/check.sh all` was redirected to a file and its exit status read directly, never piped. No `.git/index.lock` was left at any point; every commit went through `git commit`.

## Known Stubs

None. `text_for` is reached by the message list's closure on every paint and by the harness; the generator by the Help menu and the harness; the sort by `apply_sort` and the harness; the eighteen rows are on the page and the reading accepts them; every record is measured.

## Threat Flags

None. The harness writes only into a `tempfile` directory through `MessageCache::new` and `upsert_messages` and reads it back; no account row, no credential store, no network, no schema change, no new endpoint. The reading opens two source files.

## Self-Check: PASSED

`src/presentation/virtual_rows.rs`, `src/presentation/sample_mailbox.rs`, `src/presentation/mail_sort.rs`, `tests/the_list_reads_only_memory.rs` and `tests/the_list_at_two_hundred_thousand_rows.rs` exist on disk; the twelve commits named above are in `git log`, checked by `grep` before this section was written. `main` is ahead of `origin/main` and nothing was pushed.

## Next Phase Readiness

08-05 starts from `main` at the metadata commit, version 0.125.0. 08-08's mutation run can now reach `sample_mailbox`, `mail_sort` and `virtual_rows`, none of which `cargo mutants` could see inside `wx_app.rs`. 08-09 has PERF-03's three lines answered: the list exercised at 200,000 synthetic rows with sort, filter and scroll numbers on the page; a test that the callback issues no query, in two halves; and no claim of a provider mailbox anywhere. 08-06's pass over documents should know the page has a new definitions section for the scale rows and that the changelog's entry names the two untimed costs.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-15*
