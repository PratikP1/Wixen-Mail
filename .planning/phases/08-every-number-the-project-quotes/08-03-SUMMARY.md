---
phase: 08-every-number-the-project-quotes
plan: 03
status: complete
subsystem: testing
tags: [measurements, performance, cold-start, memory, webview2, harness, guards, red-green]

requires:
  - phase: 08-every-number-the-project-quotes
    plan: 01
    provides: "`docs/development/measurements.md` and the reading that refuses a row without its command, date and commit, which the four rows this plan wrote had to pass"
provides:
  - "`src/common/started.rs`: the process start instant taken as the first statement of `main`, and the line `the message list is usable: N rows, M ms after start`, said once per process and never for an empty list"
  - "`tests/the_numbers_the_targets_ask_for.rs`, 16 tests: two profile writers, the parsers, the row, a reading that holds the startup fill in place, 14 that run on every commit and two behind `#[ignore]` that start the release binary and read the working set of the process and its WebView2 tree"
  - "Four rows on `docs/development/measurements.md`: cold start 476 ms, memory with 1,000 cached messages 390 MB, idle at 120 s 391 MB, the empty-profile floor 390 MB, each with the runs behind it, the machine, the build and the command; the five definitions quoted once on the page"
  - "A fix: the window fills the module it opens on at startup. The folder tree came up empty and cached mail was not listed until a mail check finished or somebody switched modules away and back, on every profile since 2026-07-26"
  - "`guards/guards.toml`: five records, each measured; census 603, 795 records"
affects: [08-06, 08-09]

actuals:
  tokens: 20500
  tasks: 3
  commits: 13

tech-stack:
  added: []
  patterns:
    - "A once-per-process instrument as a small type holding a `OnceLock` and an `AtomicBool`, with one static for the process and one instance per test, so the once can be tested without sharing state across the test binary"
    - "A harness that starts the release binary from a test through `env!(\"CARGO_BIN_EXE_wixen-mail\")`, reads memory through PowerShell rather than a crate, walks `Win32_Process` by parent to any depth, and stops the whole tree with `taskkill /T` in a `Drop`"
    - "A reachability reading in the new target rather than a unit test in `wx_app.rs`, because the startup closure needs a window and the file is named by 48 records"

key-files:
  created:
    - src/common/started.rs
    - tests/the_numbers_the_targets_ask_for.rs
  modified:
    - src/common/mod.rs
    - src/main.rs
    - src/presentation/wx_app.rs
    - docs/development/measurements.md
    - docs/changelog.md
    - guards/guards.toml
    - Cargo.toml
    - Cargo.lock
    - .planning/WINDOWS.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

key-decisions:
  - "The profile writer sets `start_in_all_inboxes` as well as `told_about_the_alpha`, because with no folder chosen no list ever loads and there is no usable line to read; the plan's writer set only the alpha flag"
  - "The startup fill is fixed in this plan under Rule 1 rather than raised, because without it the plan's own definition of usable names a moment the program never reaches, and the fix is one call through the channel every other fill uses"
  - "The startup fill's test is a reading of the startup section of `run` in the new target, not a test in `wx_app.rs`, because the closure needs a window and the file is named by 48 records; a record couples `wx_app.rs` to the target so the reading runs on commits that could break it"
  - "The account write in the profile writer is taken one thread at a time, because keyring 4.1.5's `Entry::new` races its own lazy initialisation (ledger 374) and three tests write a profile at once; the keyring fix stays ledger 374's"
  - "Memory is read from the five series runs and the median taken over five, not the three the plan named, because the five were taken anyway and a median of five is the better number"

patterns-established:
  - "Before an instrument is built on a runtime moment, start the program on the default path and watch for the moment; a unit test that calls the loader proves the loader, not the path"

requirements-completed: []

coverage:
  - id: D1
    description: "The application writes one usable line per process with rows and milliseconds, never for an empty list, from an instant taken as the first statement of main"
    requirement: PERF-02
    verification:
      - kind: unit
        ref: "src/common/started.rs#test_the_usable_line_is_held_byte_for_byte"
        status: pass
      - kind: unit
        ref: "src/common/started.rs#test_the_usable_line_is_said_once_per_start"
        status: pass
      - kind: unit
        ref: "src/common/started.rs#test_an_empty_load_is_not_usable_and_does_not_spend_the_once"
        status: pass
      - kind: other
        ref: "the release binary's whole log under WIXEN_MEASUREMENT_SHOW_LOG, one usable line per start, quoted below"
        status: pass
    human_judgment: false
  - id: D2
    description: "A profile of the defined shape is written, the release binary is started against it, the usable line is parsed, the working set of the process and its tree is read, and rows are printed in the page's shape"
    requirement: PERF-01
    verification:
      - kind: integration
        ref: "tests/the_numbers_the_targets_ask_for.rs#test_a_ten_row_profile_reads_back_ten_messages_and_ten_bodies"
        status: pass
      - kind: integration
        ref: "tests/the_numbers_the_targets_ask_for.rs#test_the_usable_line_the_library_writes_parses_back"
        status: pass
      - kind: integration
        ref: "tests/the_numbers_the_targets_ask_for.rs#test_memory_is_read_from_the_shape_powershell_prints"
        status: pass
      - kind: integration
        ref: "tests/the_numbers_the_targets_ask_for.rs#test_the_row_has_the_pages_columns_in_the_pages_order"
        status: pass
      - kind: other
        ref: "nine ignored runs against the release binary, every one printing its rows, quoted below"
        status: pass
    human_judgment: false
  - id: D3
    description: "Cold start, memory with 1,000 cached messages, idle memory and the empty-profile floor each have a row with date, commit, version, machine, command and conditions, and the page's reading accepts them"
    requirement: PERF-04
    verification:
      - kind: integration
        ref: "tests/every_number_carries_its_command_and_its_date.rs#test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit"
        status: pass
    human_judgment: false
  - id: D4
    description: "The window fills the module it opens on at startup"
    requirement: PERF-02
    verification:
      - kind: integration
        ref: "tests/the_numbers_the_targets_ask_for.rs#test_the_module_the_window_opens_on_is_filled_at_startup"
        status: pass
      - kind: other
        ref: "the log of every measured run: Mail loaded 4 update(s), Message list now holds 500 rows, the usable line"
        status: pass
    human_judgment: false

duration: 120min
completed: 2026-09-15
---

# Phase 8 Plan 03: The list says when it became usable Summary

**The application now says, once per start, when its message list first held a row and how many milliseconds that took from the first instruction of `main`; a harness builds a profile of exactly 1,000 cached messages, starts the release binary against it, reads that line and the working set of the process and its six WebView2 processes, and the three targets have their first numbers: cold start 476 ms, memory with 1,000 cached messages 390 MB of which the application is 57 MB, idle 391 MB of which the application is 56 MB. The first run found that nothing filled the mail module at startup, so the folder tree came up empty on every profile until a mail check or a module switch; that is fixed here.**

Every count below is one this session took, with the command. Where the tree contradicted the plan, the tree won and the difference is named.

## Performance

- **Duration:** about 120 minutes, from the first read at about 22:12Z on 2026-09-14 to the merge at 23:59Z; the summary followed
- **Started:** 2026-09-14T22:12:00Z (approximate; the first commit is 22:22Z)
- **Completed:** 2026-09-14T23:59:50Z for the merge at `e801a3cf`
- **Tasks:** 3 of 3
- **Files modified:** 11 on the branch, plus the three planning files on `main`

## The three numbers, with the runs behind them

Taken 2026-09-14 against the release binary at `9d5f15c5`, version 0.124.0, rebuilt from nothing with `cargo clean --release -p wixen-mail && cargo build --release` (73 s) before the series. AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, as `Get-CimInstance Win32_Processor` and `Win32_ComputerSystem` report it. Before every run `tasklist` was checked for `cargo.exe` and `rustc.exe` and found none but the one running the test; the test binary runs one ignored test, so no thread setting is in play. Every run is `cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture <test>`.

**Cold start to a usable list, 1,000 cached messages.** The first start after the build, on its own: **520 ms**. The series of five: 721, 476, 474, 483, 468 ms; **median 476 ms**. A sixth start after the series, taken to read the log with `WIXEN_MEASUREMENT_SHOW_LOG`: 472 ms. Every usable line said 500 rows. The target is 2 s; 476 ms is under it by a factor of four, and the after-build start by a factor of nearly four. The first-run outlier of 721 ms was the first start after the harness's own test binary was rebuilt; it is in the series because the definition says the five after the build are the series.

**Memory with 1,000 cached messages**, the application's peak working set to 60 s plus its WebView2 tree at 60 s. The five: 397, 390, 390, 393, 390 MB; **median 390 MB**. The after-build run: 396 MB. In every run the application process alone peaked at **57 MB**, working set 56 to 57 MB, private 18 to 19 MB. The tree was six `msedgewebview2.exe` processes every time, 333 to 340 MB at 60 s. The target is 150 MB: met by the application process alone, by 93 MB; missed by the sum, by 240 MB, and the whole of the miss is the WebView2 tree.

**Idle memory at 120 s, 1,000 cached messages**, the application's working set plus its tree. The five at 120 s: 398, 391, 390, 393, 391 MB; **median 391 MB**. At 60 s: 397, 389, 389, 392, 390 MB, median 390 MB, so idle growth over the minute is about 1 MB and all of it is in the tree; the application process sat at 56 to 57 MB working set at both readings, unchanged. The target is 100 MB: met by the application process alone, by 44 MB; missed by the sum, by 291 MB, again the tree.

**The empty-profile floor**, no account and no mail, at 120 s. Three runs: 389, 390, 391 MB; **median 390 MB**. The application alone 54 MB working set, 55 MB peak, 17 MB private in every run; the tree 334 to 336 MB. So 1,000 cached messages cost the application process about 3 MB over an empty profile and cost the tree nothing measurable.

Whether a target is met against the sum or against the process is 08-09's judgement; the page gives both and says so.

## How the WebView2 tree was found, and what it weighed

`Get-CimInstance Win32_Process` lists every process with its parent; the harness walks `ParentProcessId` from the application's id to any depth (`descendants_of`, tested on a literal three-level tree) and reads each with `Get-Process -Id N | Select-Object WorkingSet64,PeakWorkingSet64,PrivateMemorySize64 | Format-List`. On this machine the tree is always six `msedgewebview2.exe` processes: in the sixth run, at 60 s, 122, 73, 56, 41, 22 and 17 MB, 333 MB together. It weighs the same on the empty profile as on the 1,000-message one, so it is the cost of having a preview pane at all, not of the mail. `Child::kill` would not touch them, so `Started`'s `Drop` runs `taskkill /PID <id> /T /F`, and `tasklist` after every batch found no `wixen-mail.exe` left.

## What the application did with the refusable account

Nothing. The whole log of a measured run, under `WIXEN_MEASUREMENT_SHOW_LOG`, with the announcement lines left out:

```
Logging initialized at level: Info
Starting Wixen Mail v0.124.0
Started read only: nothing will be changed at any server
0 reminders loaded
Screen reader bridge ready; UI Automation clients listening: true
Starting wxdragon event loop
wxdragon on_init callback entered
Message list created, setting up WebView
WebView2 Edge backend available: true
WebView widget created
UI setup complete, showing main frame
Main frame shown, entering event loop
Mail loaded 4 update(s) for account perf-measurement
First timer tick, event loop is running
Message list now holds 500 rows
the message list is usable: 500 rows, 472 ms after start
```

Exactly one usable line. No connection was attempted: the timer's own comment says nothing in this program checks mail on a schedule, a sync starts because somebody asked or a folder watch fired, and neither happens on a fresh start. The log of every one of the nine measured runs held no WARN or ERROR line. Ledger 446 records that what the application says when a connection is refused was therefore read by nobody here. No dialog opened in any run; a run that had shown one would have stayed short of the usable line and been reported as producing nothing.

## What the tree contradicted in the plan

1. **Nothing filled the mail module at startup, so the plan's "first `MessagesLoaded` after startup" named a moment the program never reached.** The first release run waited 60 s and got nothing; the log ended at "First timer tick". Every `load_module_data` call is inside a switch, and `do_switch_module` returns early for the module already on screen; the only other tree redraws are a finished mail sync and commands. The real profile's own log of 2026-09-14 17:30 shows the same shape, and `git log -S` shows no startup fill has existed since `ae8c48f3` of 2026-07-26 added module loading. `test_opening_mail_loads_its_folders` proved the loader works when called and stayed green for fifty days while nothing called it at startup. Fixed under Rule 1: one call after the frame is shown, of whichever module the window opens on, with the account the state holds. Red first as a reading of the startup section of `run` in the new target (`7b49768b`), green as the fix (`c2054e61`), and the changelog has a Fixed entry.
2. **A fresh profile opens with no folder chosen, so a list never loads unless `start_in_all_inboxes` is on.** The plan's writer set only `told_about_the_alpha`. The writer sets both and the harness's header says why. So the usable line reports **500 rows, not 1,000**: All Inboxes lists its first page, `ALL_INBOXES_LIMIT`. The definition of "1,000 cached messages" is about the cache and holds; the page says what the list holds and why.
3. **The gate reads an integration target's test names bare.** The first attempt at the harness's red commit named `the_numbers_the_targets_ask_for::test_...` in its trailers, the shape `CLAUDE.md` gives for shell suites, and the gate reported every one as named and never run. 08-02's trailers were bare; recommitted bare, nothing else changed.
4. **Ledger 374's keyring race reached this target.** The second attempt at the startup-fill red commit was refused because `test_the_profile_has_the_shape_the_definition_gives` failed with "No default store has been set": three tests write a profile at once and `save_account` reaches the credential store even with an empty password. The account write is now taken one thread at a time in the writer; six runs since, none refused. The keyring fix stays ledger 374's.
5. **`wx_app.rs:17086` was the handler's line on 2026-09-14 and still was.** `tell_the_list_how_many` at `:19121`, the sample-mailbox load at `:5281`, `ask_about_the_alpha_once` at `:26405`: all where the plan said.

## Task commits

Branch `the-list-says-when-it-became-usable` from `main` at `5d19e9cd`.

1. **Task 1, red:** `16dbfbf4` test(08-03), five tests named; the sixth, the unmarked case, passed against a stub answering nothing, which is the case it is for.
2. **Task 1, green:** `76469bf7` feat(08-03), the module, the mark above the hook, the handler's line.
3. **Task 1, records:** `59237bc2` test(08-03), two records, census 600.
4. **Task 2, red:** `584acda1` test(08-03), eleven tests named; the twelfth, a log without the line parsing to nothing, the same shape as above.
5. **Task 2, green:** `844f45d5` feat(08-03), the writers, the harness, the parsers, the row.
6. **Task 2, the finding, red:** `7b49768b` test(08-03), the reading of the startup section, and the account write serialised.
7. **Task 2, the fix:** `c2054e61` fix(08-03), the startup fill.
8. **Task 2, refinements:** `236c9708` feat(08-03), the command backticked and the log's complaints printed with a test for the reading.
9. **Task 2, records:** `9d5f15c5` test(08-03), three records, census 603.
10. **Task 3, refinement:** `e4c4c6e8` feat(08-03), the tree named per process and the whole log on request.
11. **Task 3, the page:** `44c444b7` feat(08-03), four rows, the definitions, the changelog, version 0.125.0.
12. **Task 3, ledger:** `3d749871` docs(08-03), three entries.
13. **Merge:** `e801a3cf`, gate green on the merge.

**Plan metadata:** the commit carrying this summary, `STATE.md` and `ROADMAP.md`.

## What the gate selected per file

From the hook's own output on each commit. `src/common/started.rs` and `src/common/mod.rs`: `common` and `common::started`. `src/main.rs`: `main`, the filter that matches nothing, the known hole; what reached the one line there is the harness reading the usable line off the release binary nine times, which is what the plan said would reach it. `src/presentation/wx_app.rs`: `presentation::wx_app`, 199 tests, plus the seven targets the records couple to it; on the fix commit the new target did not run, because no record said to, and the third record of `9d5f15c5` now couples it: `check.sh --suites-for guards/guards.toml src/presentation/wx_app.rs` prints `the_numbers_the_targets_ask_for`, and for `src/common/started.rs` prints the same. `tests/the_numbers_the_targets_ask_for.rs`: the target and the tree guards. `guards/guards.toml`: the tree guards alone. `docs/development/measurements.md`, `docs/changelog.md`, `Cargo.toml` and `Cargo.lock` rode together and the gate ran the tree guards; the `.planning/WINDOWS.md` commit answered `docs_only` and ran the document-reading targets.

`scripts/check.sh all` on the branch at `3d749871`: 7,722 passed, none failed, 3 ignored (the two measurement tests and the registration test), 56 result lines, 315 s, exit read directly from a redirect. On the merge at `e801a3cf`: 7,722 passed, none failed. No keyring race in either.

## Deviations from Plan

**1. [Rule 1, bug] The window did not fill the module it opens on at startup.** Found during task 2's first release run; described above. Files: `src/presentation/wx_app.rs`, `tests/the_numbers_the_targets_ask_for.rs`; commits `7b49768b` and `c2054e61`. The changelog's Fixed entry says what it did on every profile since 2026-07-26.

**2. [Rule 3, blocking] `start_in_all_inboxes` in the profile's settings.** Without it the tree fills and nothing is chosen, which is the setting's own promise, and no list loads. Files: the target; commit `844f45d5`. The usable line therefore reports the first page of 500.

**3. [Rule 3, blocking] The account write serialised in the writer.** Ledger 374's race, one run in three. Commit `7b49768b`.

**4. The startup fill's test lives in the new target as a source reading, not in `wx_app.rs`.** The closure needs a window and the file is named by 48 records; the harness sees the fill end to end, the reading holds it on every commit that could break it through the record.

**5. `complaints_in` and the tree naming were written before their test, and the test followed in the same commit.** Two small readings in a test target; said here rather than hidden.

**6. Memory medians over five runs rather than three.** The five series runs supplied the memory readings, and a median of five is the better number; the page says five.

Otherwise the plan was executed as written. No package installed; `Cargo.toml` touched by the bump alone, `Cargo.lock` by the same line; no `unwrap` or `expect` outside the test target; no tracked file edited by a script, and this executor's exception set was zero. `cargo fmt` ran before every commit.

## Guard records

Five added, each break applied by hand and measured before it was written, then all five confirmed by `scripts/guards.sh --remeasure` in one run of 256 s: "All 5 guards redden exactly the tests their records name". Nothing written back, so no count had moved.

| Record | File | Suite | Break | Measured |
|---|---|---|---|---|
| the usable line keeps the shape the harness parses | `src/common/started.rs` | library | "usable" to "ready" | 7,248 passed, exactly 2 failed, 79 s |
| the usable line is said once per process | `src/common/started.rs` | library | the once dropped | 7,249 passed, exactly 1 failed, 90 s |
| the measurement profile keeps the body size its definition gives | the target | the target | 2,048 to 1,024 | 13 passed, exactly 1 failed |
| the harness can still parse the line the application writes | `src/common/started.rs` | the target | the same reworded line | 13 passed, exactly 1 failed |
| the module the window opens on is filled at startup | `src/presentation/wx_app.rs` | the target | the fill replaced by a `let _` | 13 passed, exactly 1 failed; the library 7,250 passed, none failed, 71 s |

The last row is the reason the record exists: no unit test reaches the startup closure.

No test was added to `src/presentation/wx_app.rs`, `tests/house_style.rs` or `tests/wired.rs`: 199, 70 and 71 before and after, by `grep -cE '^\s*#\[(test|tokio::test)\]\s*$'`. The count check therefore never printed a remedy on any commit, and none was run; the plan's line about running the remedy for `wx_app.rs` assumed a test would land there, and none did.

Records: 790 before by the TOML reader, 795 after. Census 192 + 603.

## Ledger

`.planning/WINDOWS.md` 445 before, 448 after, both halves of each entry written by `gsd-tools windows append`, checked by `grep -c "| N |"` and `grep -c '"id": N'` for each, one and one; no backslash in the new text, checked over the diff's added lines:

- 446, unrun-verify, `tests/the_numbers_the_targets_ask_for.rs`: the refusable account was never dialled, so what the application does with a refusal was read by nobody.
- 447, unrun-verify, `docs/development/measurements.md`: idle with a live account is a different idle and no real account has ever been used.
- 448, deviation, `docs/development/measurements.md`: the after-build start depends on what the machine had read; a start after a reboot was not taken.

## Issues Encountered

Two commits were refused by the gate and recommitted, both described above under what the tree contradicted: the trailer shape, and the keyring race. `scripts/check.sh all` was redirected to a file and its exit status read directly, never piped. No `.git/index.lock` was left at any point; every commit went through `git commit`. `red-run.log` is gitignored and stayed out of every commit. Carriage returns were measured with `tr -cd '\r' | wc -c` on every document and source file touched: none.

## Known Stubs

None. The usable line is reached by the release binary on every start that lists mail; the harness is reached by hand and its parsers and writers on every commit; the four rows are on the page and the reading accepts them; every record is measured.

## Threat Flags

None. The harness writes only into a `tempfile` directory and starts the binary with `WIXEN_MAIL_DATA` pointed there and `--read-only`; the one thing it reaches outside the tree is the Windows credential store, where `save_account` with an empty password removes an entry named `perf-measurement` that does not exist, and ledger 374 already records that the integration seam does not exist. No network, no schema change, no new endpoint.

## Self-Check: PASSED

`src/common/started.rs`, `tests/the_numbers_the_targets_ask_for.rs` and the four new rows on `docs/development/measurements.md` exist on disk; the thirteen commits named above are in `git log --all`, checked by `grep` before this section was written. `main` is ahead of `origin/main` and nothing was pushed.

## Next Phase Readiness

08-04 starts from `main` at the metadata commit, version 0.125.0. 08-09 has the three numbers and the split it needs to judge each target: the application process meets all three, the sum with WebView2 misses two, and the page says which is which. 08-06's pass over documents should know that `docs/development/measurements.md` now holds the five definitions quoted from the harness's header and that the changelog's Fixed entry dates the startup defect to 2026-07-26.

---
*Phase: 08-every-number-the-project-quotes*
*Completed: 2026-09-15*
