---
phase: 14-the-real-account-proofs
plan: 06
subsystem: proof-reading
status: complete
tags: [proof-reader, crash-log, privacy, REAL-01, REAL-02, answer-9]
requires: [14-05]
provides:
  - "scripts/read-a-proof.py: --profile, --since, --until, --folder, --message, and the two seams --mask-stdin and --openings"
  - "common::logging: CRASH_FILE, crash_entry(build, location, payload), append_to_the_crash_file(folder, entry)"
  - "presentation::page_window: refuse_unless_a_page(address, folder), say_why_the_start_stopped"
  - "tests/the_proof_reader_prints_no_private_mail.rs: a fixture profile in a temporary folder"
affects: [14-08, 14-09, 14-10, 14-11, 14-12]
tech-stack:
  added: []
  patterns: ["a script's privacy rule held by a Rust target that runs it over a fixture built with the real schema", "a writer that takes its folder, so a test hands it a temporary one"]
key-files:
  created:
    - scripts/read-a-proof.py
    - tests/the_proof_reader_prints_no_private_mail.rs
  modified:
    - src/common/logging.rs
    - src/main.rs
    - src/presentation/page_window.rs
    - tests/the_log_carries_what_a_report_needs.rs
    - scripts/check.sh
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "main.rs keeps one small write_to_the_crash_file, through the one writer, because log_crash had five callers besides the hook (the plan read one)"
  - "The crash entry drops its Time: SystemTime line; the writer's local stamp already gives it"
  - "A spoken sentence is cut where it withholds the larger part: everything after the first colon, or between Nothing was copied. and is still in"
  - "A number at or above 2^31 is read as one reserved here, since this program counts down from 4294967295 and a server counts up from 1"
  - "Warnings are tallied by their words before the first colon, replaced by a word count when those words hold a quote or an @"
metrics:
  duration: "about 2 hours 15 minutes, 2026-10-06"
  completed: 2026-10-06
actuals:
  # chars/4 over the lines added in src, tests, scripts, guards and docs, git diff main on the branch
  tokens: 21000
  tasks: 3
  commits: 7
---

# Phase 14 Plan 06: The proof reader and one crash writer Summary

`scripts/read-a-proof.py` reads a Wixen Mail profile read-only and prints the
build, the waiting queue, the proof messages and the log lines by template,
with no subject but a proof's and no address unmasked; a crash entry now names
the whole build through one writer; and the library's tests no longer write
into the tester's `crash.log`. Tested over a fixture profile only; the reader
has not been run on Pratik's profile, which 14-09 does first.

## What works now

- **A crash entry names its build.** The panic hook builds its entry with
  `logging::crash_entry(&version::current(), ...)`, so it reads
  `Wixen Mail v1.0.0-alpha.1+N.gHASH` on a build the installer script made,
  and writes it through `logging::append_to_the_crash_file`, which the
  binary's five starts that stop and the page window's two refusals share.
- **The library's tests stay out of the tester's crash file.** The page
  window's refusal is decided and written by `refuse_unless_a_page(address,
  folder)`; `show` passes `default_log_dir()` and keeps `set_app_name` before
  `build(` in its own body; the unit test hands it a temporary folder and
  reads its three lines there.
- **The reader.** Standard library only. The database through
  `sqlite3.connect(<uri>?mode=ro, uri=True)`, columns read with
  `PRAGMA table_info` before each select, subjects selected only where they
  begin "Wixen proof"; the settings with `json`; every daily log the UTC
  window touches; `crash.log` with its local stamps turned to UTC. A
  `Speaking:` line is printed through one of six templates with its subject
  withheld, else as its length; addresses are masked by the feedback report's
  rule and quoted text, in double quotes or backticks, by its length. The
  three test refusals are dropped and counted.

Verify commands, counts taken on the day: `common::logging::` 12 (10 before),
`presentation::page_window::` 4, `--test the_log_carries_what_a_report_needs`
23 (21 before, not the 13 the plan read), `--test a_separate_window_is_its_own_process`
7, `--test the_planning_files_agree_with_themselves` 17 (before the docs),
`--test the_proof_reader_prints_no_private_mail` 12, `bash scripts/check.test.sh`
all cases pass, `--test house_style` 74, `--test the_words_that_say_nothing`
10; all passing. `python -m doctest scripts/read-a-proof.py` passes.

Acceptance greps: `CARGO_PKG_VERSION` in `src/main.rs` names line 368, the
erase note, only; `fn log_crash` and `fn crash_log` find nothing;
`default_log_dir` in `page_window.rs` names lines 393 and 436, both inside
`show`, and none in the tests module; `show(` in the tests module finds only
the string `"pub fn show("` in the profile-name reading, which reads the body
and calls nothing. `mode=ro` 2, `immutable` 0, `password` nothing;
`read-a-proof.py` sits on `what_no_suite_reads` at `scripts/check.sh:141`.

## The reader, for 14-09 to 14-12

```
usage: read-a-proof.py [-h] [--profile PROFILE] [--since SINCE]
                       [--until UNTIL] [--folder FOLDER] [--message MESSAGE]
                       [--mask-stdin] [--openings]

Read a proof back from a Wixen Mail profile, read-only, printing no mail.

options:
  -h, --help         show this help message and exit
  --profile PROFILE  the profile folder; WIXEN_MAIL_DATA, else
                     %LOCALAPPDATA%\wixen-mail
  --since SINCE      the window's start, UTC, such as 2026-10-05T23:50Z
  --until UNTIL      the window's end, UTC; now when not given
  --folder FOLDER    a folder to count, by name or path; may repeat
  --message MESSAGE  a message row to read; may repeat
  --mask-stdin       print the lines on standard input masked by the feedback
                     report's rule, and stop
  --openings         print every fixed text this reader matches, one per line,
                     and stop
```

A sitting's reading is `python scripts/read-a-proof.py --since <UTC start>
[--until <UTC end>] [--folder "Wixen proof"] [--message <row>]`. Sections, in
order: the build and the log level; log lines by template (replay, crossing,
refusal, sign-in, sync); spoken, opening with "A Speaking: line is what the
program handed the screen reader, not what NVDA said."; other warnings and
errors by shape; settings; accounts (provider, protocol, `use_oauth`);
the waiting queue; proof messages; the folders and messages asked for;
calendars by provider, events, contacts, tasks and sync state; the crash file.

## Commits

| Commit | What | Hook |
|---|---|---|
| `2aed4ab1` | test: failing cases for one crash writer naming the build | red, 156 s |
| `1b507e5d` | feat: one crash writer naming the build; seven records re-measured | affected, 315 s |
| `8b707911` | test: failing cases for a proof reader that prints no private mail | red, 134 s |
| `7b1b711f` | feat: `scripts/read-a-proof.py`; the script on `what_no_suite_reads` | affected, 279 s |
| `f251ffed` | test: four guard records, five re-measured | affected, 273 s (refused once at 276 s, below) |
| docs | changelog, ledger 828, REAL-01's and REAL-02's lines and rows, the four marks, this summary | see the report |

The first try at the records commit was refused at 276 s by
`the_planning_files_agree_with_themselves`, because the roadmap row and
`completed_plans` already said 6 and 255 on disk with no summary yet; it went
through once this summary was on disk.

`Fails-until-green:` lines. Task 1:
`common::logging::tests::test_a_crash_entry_names_the_build_it_came_from`,
`common::logging::tests::test_the_crash_writer_appends_a_stamped_entry_in_the_folder_it_is_given`,
`presentation::page_window::tests::test_an_address_that_is_not_a_page_opens_no_window`,
`test_the_panic_hook_names_the_build_a_crash_came_from`,
`test_the_reading_complains_when_the_panic_hook_names_the_crate_version` and
the count check, which the new cases made name seven records. Task 2: the
target's twelve cases, all failing on the script's absence.

The red stubs, which are not green code and were each replaced in `1b507e5d`:

```rust
pub fn crash_entry(_build: &str, _location: &str, _payload: &str) -> String {
    String::new()
}
pub fn append_to_the_crash_file(_folder: &Path, _entry: &str) -> std::io::Result<()> {
    Ok(())
}
fn refuse_unless_a_page(address: &str, _folder: &Path) -> Result<String, i32> {
    may_be_followed(address).ok_or(NOT_A_PAGE)
}
```

## Guard records

Four new, each break taken by hand before it was written down; the
arrived-since count 686 to 690; 1,487 records by the TOML reader.

| Record | Break | Red |
|---|---|---|
| the proof reader withholds a spoken subject that is not a proof's | `shown = subject` | the privacy case |
| the proof reader masks every address the feedback report masks | `redact` without `without_addresses` | the masking case, the by-template case, the privacy case |
| the page window writes its refusal into the folder it is given | the write taken out, a tuple left in its place | the page window's unit test, alone, over the whole library |
| the panic hook names the whole build in its crash entry | `env!("CARGO_PKG_VERSION")` back in the hook | the reading and its companion |

The page window's break takes the write out rather than pointing the refusal
at `default_log_dir()`, so no measurement writes into Pratik's crash file.
Re-measured, one call each time as the first run: seven after task 1
(`logging.rs`'s two and the log target's five that its two new cases flagged),
all agreeing; then the four new records and premise 9's "a separate window
opens a page and refuses everything that is not one", all five agreeing.
`test_every_guard_record_still_names_one_place_in_the_tree` passed before each
commit that touched code; premise 9's anchors (`page_window.rs`'s
`may_be_followed`, `logging.rs`'s two, `main.rs`'s three) were outside the
edited regions and still name one place.

## Ledger

- 828 opened and fixed (`deviation`): the page window's test wrote three lines
  into the real profile's `crash.log` on every library run, 1,515 of each from
  2026-09-22 to 2026-10-04 by research 3's count; the lines already there stay
  and the reader drops them. The entry also records the reading of the other
  profile-resolving sites in `src`: no test builds the main window, the window
  targets set `WIXEN_MAIL_DATA`, and one library test,
  `test_the_key_path_names_the_file_the_service_reads_and_writes`, creates the
  profile's root folder when it is missing and writes no file.
- Header: 703 open, 125 fixed, 828 in all.

## Deviations from Plan

1. **[Premise] `log_crash` had five callers besides the hook** (`main.rs`
   lines 176, 191, 198, 382 and 387: a start that cannot run, the window's run
   and start errors, the data folder unresolved or not made). The plan read
   only the hook. They now go through a small `write_to_the_crash_file` in
   `main.rs` that calls the one writer with `default_log_dir()`, and
   `fn log_crash` is gone.
2. **[Shape] The crash entry drops `Time: SystemTime { .. }`**, which printed
   the time as a count of intervals; the writer's stamp gives it in words. The
   changelog says so.
3. **[Shape] The fixture writes the account and the messages by plain SQL**
   after `MessageCache::new` lays the real schema down, since the message
   writer takes some thirty fields with no default; folders, calendars and the
   waiting copy go through `save_folder`, `save_calendar` and
   `keep_a_move_waiting`.
4. **[Shape] Two flags beyond the plan's five**: `--mask-stdin`, which the
   masking case feeds the same lines as `feedback_report::redact`, and
   `--openings`, which the companion holds to the shipped source. The
   companion reads the openings in `src` as text with test modules cut, not
   inside a log call: the six spoken templates live in the sentences
   `server_delete.rs` and `imap.rs` build, which reach the log through the one
   `Speaking:` call.
5. **[Shape] The crash case reads its own window**, an hour back from now,
   because the writer stamps the current local time; the log cases read the
   plan's 23:50Z to 00:10Z window.
6. **[Process] The ledger entry went into the documents commit**, not task
   1's, under the brief's rule of one documents commit before the merge.
7. **[Premise] Counts on the day**: the log target held 21 tests, not 13; the
   register held 1,483 records, not 1,437; `page_window.rs` and `logging.rs`
   line numbers held as the plan read them, apart from `main.rs`'s callers in
   deviation 1.
8. **[Process] The first records commit was refused** by the planning files
   target, as said under Commits.

## Known limits of the reader, said plainly

- A warning or error whose words before its first colon are a subject in plain
  words, with no quote or @, would print those words in the tally. No warning
  in `src` is written that way today; the shape rule cannot prove none will be.
- An Info line of a shape the reader has no template for is counted and not
  printed, so a new proof line needs a template before a sitting can read it.
- "Reserved here" is read as a number at or above 2^31.

## TDD Gate Compliance

Task 1: `2aed4ab1` red, then `1b507e5d`. Task 2: `8b707911` red, then
`7b1b711f`. Each red was accepted by `red-commit.sh` and each named case failed
for the reason it names, read before the green: the empty entry, no crash file
in the folder given, the hook naming `CARGO_PKG_VERSION`, and for task 2 the
script's absence. Task 2's expected values were checked by reasoning against
the fixture before the green (the reserved number 4294967295, `filed_here` 1 on
the copy, `ad***@example.com` by `mask_email`'s two characters, the 23:40 and
00:20 lines outside the window), and its green passed all twelve on the first
run; the two script records then showed the privacy and masking cases go red
when the rule is taken out.

## Threat Flags

None beyond the register. T-14-06-01: the six spoken templates withhold the
subject, an unknown spoken shape prints its length, quoted text and addresses
are masked, and the fixture's private subject, sender, the account's address,
its stored secret and the body are asserted absent from every output the
target asks for. T-14-06-02: the script names no secret column, no credential
store and no `oauth.toml`, read by a case. T-14-06-03: `mode=ro`, and the
database file's bytes are compared before and after. T-14-06-04: the refusal
writes to the folder it is given and the guard break takes the write out.
T-14-06-05: `--openings` held to `src`. T-14-06-06: the hook's reading and its
record. T-14-06-SC: nothing installed.

## Self-Check: PASSED

`2aed4ab1`, `1b507e5d`, `8b707911`, `7b1b711f` and `f251ffed` are on the
branch; `scripts/read-a-proof.py` and
`tests/the_proof_reader_prints_no_private_mail.rs` exist; every file in
key-files is modified on it.
