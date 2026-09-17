---
phase: 09-what-the-first-day-of-testing-found
plan: 10
subsystem: importing mail, the File menu, the shortcuts page, the user guide, four pages that describe the Outlook reader, guards, the phase's closing read
tags: [import, folder, DirDialog, thunderbird, wired, guards, changelog, user-guide, comparison, privacy, closing-read]

requires:
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-08: the Outlook data file reader reached from the picker at 06fdc9b7, which is the build the four pages date their correction to; the pattern of a wired.rs reading with a struct of yes-or-no answers and a companion that splices each out in memory"
  - phase: 04-writing-and-reading-a-message-in-full
    provides: "service::mailbox_archive::opened and a_folder_looked_over, the folder branch that had no picker; application::import_tree::what_was_chosen answering AnArchive for a folder"
provides:
  - "wx_app::import_a_folder_of_messages: File, Import a Folder of Messages, a DirDialog, the folder handed to the same worker Import Mailbox hands a file to"
  - "wx_app::an_account_to_import_into, refuse_to_import and mail_brought_in_from: the readiness check, the refusal and the worker start shared by the two pickers"
  - "tests/wired.rs: WhatTheFolderImportDoes, the reading that holds the item, its arm, its DirDialog and its hand-over, with a companion that takes each half out; the older import reading re-pointed at the shared worker start"
  - "docs/KEYBOARD_SHORTCUTS.md: rows for Import Mailbox, Import a Folder of Messages, Export Mailbox and Import PGP Private Key, none of which was on the page"
  - "docs/USER_GUIDE.md: Import and Export, the three commands, what each takes and leaves, why Imported, the unproven .pst sentence and the Thunderbird sentence"
  - "docs/comparison.md, docs/privacy.md, .planning/intel/built-and-left.md, .planning/codebase/INTEGRATIONS.md: a dated sentence beside the one issue 53 named"
  - "docs/changelog.md: the folder entry for #53 point 3 with the Thunderbird limitation and the gathered points 4 to 6; the older import entry dated"
  - "guards/guards.toml: one record measured, 17 re-measured and read; 868 records, census 802 + 66"
  - "The phase's closing read: FOUND-02 to FOUND-07 and FOUND-10 to FOUND-12 ticked clause by clause, FOUND-08 and FOUND-09 left open on their CI clauses, the roadmap's nine criteria read the same way, phase 9 complete in the progress table"
affects: [the tester, who has the first folder of saved mail and the first Thunderbird profile this will meet; whoever pushes main next, which closes FOUND-08's walk clause and FOUND-09's transcript clause or reopens them; the next planner, who starts from the README's groups 3 to 7]

actuals:
  tokens: 24000
  tasks: 2
  commits: 6

tech-stack:
  added: []
  patterns:
    - "Two pickers for one worker share three functions, the readiness check, the refusal and the worker start, and the reading that holds the worker's shape follows the file picker into the shared function rather than reading one picker's body; a second picker with its own worker would be a second place for the reload-after-counts rule to go wrong"
    - "A page that promised a command that did not exist is corrected by a dated sentence beside the promise saying from which build it was true, never by deleting the promise, so a reader of the older build finds out why it did not work"

key-files:
  created: []
  modified:
    - src/presentation/wx_app.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/comparison.md
    - docs/privacy.md
    - .planning/intel/built-and-left.md
    - .planning/codebase/INTEGRATIONS.md

key-decisions:
  - "The item's letter is O (Import a F&older of Messages), not the plan's F: Fetch Missing Message Text has Alt+F on the File menu, and tests/wired.rs's menu-letter check would have refused the plan's spelling at the commit"
  - "The import handler was split rather than duplicated: an_account_to_import_into asks for the cache and the account and says why not, refuse_to_import words a refusal on both channels, and mail_brought_in_from says the opening sentence and starts the worker; import_a_mailbox and import_a_folder_of_messages are each a picker and a hand-over"
  - "The shortcuts page gained four rows, not one: Import Mailbox had no row to sit beside, and Export Mailbox and Import PGP Private Key were missing with it"
  - "The Thunderbird layout is said, not recognised: what a profile's mail directory becomes was read off the walk (one folder per mailbox file, .msf refused and counted, .sbd nesting one level out) and written in the changelog, the guide and ledger 510, and no profile has been read"
  - "FOUND-08 and FOUND-09 stay open, on one clause each, because both wait for a run on CI that a push of main triggers and nothing has been pushed; the phase is marked complete in the progress table because every summary is complete, and the two clauses are named where they wait"

patterns-established:
  - "A check's statement of its own blind spot is about that check and not the suite: the dialog collision test says it cannot see menus, and the menu collision test sits beside it; the green commit said no check read menus and a fix commit corrected it after the hook listed the test it ran"

requirements-completed: [FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06, FOUND-07, FOUND-10, FOUND-11, FOUND-12]

coverage:
  - id: D3
    description: "A folder of saved messages can be chosen through a directory picker on its own File item and goes down the directory branch mailbox_archive::opened already has; the changelog's promise is true with a dated sentence; the changelog says a Thunderbird profile folder is not recognised as such"
    requirement: FOUND-11
    verification:
      - kind: integration
        ref: "tests/wired.rs#test_a_folder_of_messages_can_be_chosen_and_goes_to_the_import_worker"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_the_reading_of_the_folder_import_can_see_the_hand_over_taken_out"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_importing_mail_reads_the_list_back_before_it_finishes, re-pointed at mail_brought_in_from"
        status: pass
      - kind: other
        ref: "grep -c 'DirDialog' src/presentation/wx_app.rs: 2 before, 3 after; grep -c 'Import a Folder of Messages' docs/KEYBOARD_SHORTCUTS.md: 1; grep -c 'Corrected on 2026-09-17' docs/changelog.md: 1; the Known limitations line names .sbd and .msf"
        status: pass
    human_judgment: false
  - id: D4
    description: "The four documents describe what is reachable, dated where they change, and the user guide names the three File commands"
    requirement: FOUND-11
    verification:
      - kind: integration
        ref: "cargo test --test house_style, 74 passed; cargo test --test the_words_that_say_nothing, 9 passed"
        status: pass
      - kind: other
        ref: "grep -c 'Import Mailbox' docs/USER_GUIDE.md: 1; the section names Import Mailbox, Import a Folder of Messages and Export Mailbox and the Thunderbird sentence; the four pairs quoted below"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether a real folder of somebody's saved mail imports as they expect, and what a Thunderbird profile becomes in practice"
    requirement: FOUND-11
    verification: []
    human_judgment: true
    rationale: "Ledger 509 and 510. Nobody has run the item by hand and no Thunderbird profile has been read; the sentences describing one were read off the walk"

duration: 42m to the merge, about 1h05m with the closing read
completed: 2026-09-17
status: complete
---

# Phase 9 Plan 10: A folder of saved messages can be chosen, and the pages say what is reachable Summary

**File, Import a Folder of Messages is a second item beside Import Mailbox that opens a folder
picker, which a file picker cannot be, and hands the folder to the same worker a chosen file goes
to, so the folder branch `mailbox_archive::opened` has carried since it was written is reached for
the first time and the changelog's "a folder you point it at" is true, with a dated sentence saying
from which build. The two pickers share the readiness check, the refusal and the worker start, and
`tests/wired.rs` holds the item to its arm, its `DirDialog` and its hand-over with a companion that
takes each half out. What a Thunderbird profile folder becomes here is said in the changelog, the
guide and the ledger rather than recognised. The four pages that said Outlook data files were read
each keep their sentence and gain a dated one saying the reader was reachable by nothing until
09-08's merge and has met no real file; the user guide gains Import and Export, naming the three
commands; the shortcuts page gains the four File rows it never had. Then the phase's closing read:
nine of the twelve `FOUND` requirements ticked clause by clause with the plan and test that closed
each, FOUND-08 and FOUND-09 left open on the one clause each that waits for CI, the roadmap's nine
criteria read the same way, and phase 9 complete in the progress table.** #53 advanced with the merge
commit, points 3 and 7; the issue stays open for points 4 to 6. Nothing pushed.

## Performance

- **Duration:** 42 min from the branch to the merge, of which about 5.5 minutes were the count
  check's remedy over 17 records, 14 s the one new record's measurement, and 13 minutes the two
  whole gates (390 s on the branch, 385 s on `main`'s hook at the merge); about 1 h 05 min with
  the summary and the closing read
- **Started:** 2026-09-17T06:34:53Z (first commit 06:39Z)
- **Merged:** 2026-09-17T07:10:22Z at `8eba6a38`, the gate green at 07:16:47Z
- **Tasks:** 2
- **Files modified:** 10 (0 created)

## What landed

| Where | Before | Now |
|---|---|---|
| The File menu | Import Mailbox, Export Mailbox, Import PGP Private Key | Import a F&older of Messages between the first two, "Read every saved message and mailbox file in a folder in, keeping the folders inside it" |
| `menu_ids!` | `ID_IMPORT_MESSAGES` | `ID_IMPORT_A_FOLDER_OF_MESSAGES` after it, with a comment saying why a folder wants its own item |
| The handler arm | one | `_ if id == ID_IMPORT_A_FOLDER_OF_MESSAGES => import_a_folder_of_messages(...)` |
| `import_a_mailbox` | the cache check, the account check, the `FileDialog`, the opening sentence and the worker, in one function | `an_account_to_import_into`, the `FileDialog`, `mail_brought_in_from` |
| `import_a_folder_of_messages` | nowhere | `an_account_to_import_into`, `DirDialog::builder(frame, "Import a folder of saved messages", "")`, `mail_brought_in_from` |
| `mail_brought_in_from` | the tail of `import_a_mailbox` | its own function: the opening sentence, the worker, `fill_folders_from`, the counts said, then the tree read back |
| `tests/wired.rs` | 75 tests by the count check's count | 77: `WhatTheFolderImportDoes` with four answers, its reading, and the companion that splices the hand-over out and swaps the folder picker for a file picker; `test_importing_mail_reads_the_list_back_before_it_finishes` follows Import Mailbox into `mail_brought_in_from` and reads that |
| `docs/KEYBOARD_SHORTCUTS.md` File table | no row for any of the four | Import Mailbox, Import a Folder of Messages, Export Mailbox, Import PGP Private Key, each `(none)` with what it does; the closing sentence says File also moves mail in and out |
| `docs/changelog.md` | the import entry promising a folder since it was written | the folder entry under `[Unreleased]`, Fixed, with the Thunderbird limitation and the gathered points 4 to 6; "Corrected on 2026-09-17" on the older entry |
| `docs/USER_GUIDE.md` | Import Mailbox and Export Mailbox mentioned nowhere (`grep -c` 0) | Import and Export, ninth in the contents, between Attachments and Other modules |

The four pairs, the sentence issue 53 named and the sentence beside it now:

1. `docs/comparison.md:101-104`: "Mail comes in from mbox files, single messages, folders inside zip
   archives and Outlook data files, keeping whatever folder structure it arrived with, and goes out
   the same way." Beside it: "Corrected on 2026-09-17: 'Outlook data files' above described a reader
   that no command could reach. It was in the tree with its own tests from 2026-09-05, and nothing
   called it until the build of 2026-09-17 that carries the fix for issue 53. File, Import Mailbox
   reaches it since then, and no real Outlook data file has been through it yet, because neither
   this program nor the library it reads with can write one to test against. A folder of saved
   messages is chosen through File, Import a Folder of Messages since the same build; before it the
   only picker could not answer with a folder. Goes out 'the same way' is still generous: export
   writes one zip of mbox files, and nothing else."
2. `docs/privacy.md:88-89`: "...brought in from a saved message, a mailbox archive or an Outlook data
   file." Beside it: "Corrected on 2026-09-17: the last of those was true of the code and not of
   anything you could do until the build of 2026-09-17 that carries the fix for issue 53, because no
   command reached the Outlook data file reader before it. File, Import Mailbox reaches it since
   then, and no real Outlook data file has been through it yet."
3. `.planning/intel/built-and-left.md:55`, the row "Outlook PST/OST import into a non-server-backed
   Imported area" under Built and exercised. A row added under it: "Corrected 2026-09-17: the row
   above was in the wrong table when it was written. On 2026-08-29 the reader was in the tree with
   its tests and reached from no non-test path, which is this page's own definition of 'built but
   unproven'... It became reachable with 09-08's merge, `06fdc9b7` on 2026-09-17, through File,
   Import Mailbox and `src/application/importing_an_outlook_data_file.rs`, and no real Outlook data
   file has been through it. The row stays as it was written, by this page's rule of correction by
   addition."
4. `.planning/codebase/INTEGRATIONS.md:58`: "Outlook `.pst` files read directly from disk for
   one-time import". A line under it: "Corrected 2026-09-17: from the day this line was written until
   09-08's merge (`06fdc9b7`, 2026-09-17) no command reached that reader, so the line described code
   and not an integration anybody could use. It is reached since then from File, Import Mailbox
   through `src/application/importing_an_outlook_data_file.rs`, and no real Outlook data file has
   been through it. A folder of saved messages is chosen through File, Import a Folder of Messages
   since 09-10; the one picker before it could not answer with a folder."

## Task commits

| Commit | What |
|---|---|
| `8ac253e0` | test(09-10): the red half of task 1, three readings named and the count check |
| `6c5fc1e9` | feat(09-10): the id, the item, the arm, the handler split, one record, the remedy over 17, the four shortcuts rows, the changelog entry and the dated older entry |
| `857fadbc` | docs(09-10): task 2, the four pages dated, the guide's section, the gathered Known limitations |
| `7c528ab4` | fix(09-10): the comment on the item's letter corrected, see Deviations 4 |
| `8eba6a38` | Merge 09-10 into `main` |

Branch `a-folder-of-messages-can-be-chosen` from `main` at `3231dd4e`. Not pushed; 89 commits
unpushed before the commit that lands this summary.

## Honest RED and GREEN

Task 1's red named three readings and the count check. Against a tree with no item, no arm and no
`mail_brought_in_from`, the two new readings panicked in `what_the_folder_import_does` at the arm
they could not find, and the older reading, re-pointed at the shared function, failed its first
assertion because `import_a_mailbox` did not yet hand over to a function of that name. The gate in
`red` mode ran exactly the four named; the count check fired on `wired.rs` 75 to 77 and was named.

Task 1's green: `cargo test --test wired` 77 passed on the first run after the code, with no
iteration. The record was measured before the commit (14 s, both named went red, nothing else) and
the remedy over 17 was run and read before the commit.

Task 2 is documents only, which `CLAUDE.md` lists among the exceptions to test-first; the
document-reading targets are its check, and both passed on the first run.

The fix commit changed a comment and nothing else; the hook ran the `wx_app` filter and the eleven
coupled targets over it.

## What the gate selected

| File | On the branch |
|---|---|
| `tests/wired.rs` | `--test wired` on the red commit, in `red` mode: exactly the four named failed |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` and the eleven coupled targets `--suites-for` answers for it (`a_whole_folder_moves_both_bounds`, `one_sign_in_per_piece_of_work`, `nothing_leaves_the_outbox_unasked`, `the_conflict_choice_can_be_heard`, `nothing_sends_a_flag_change_unasked`, `the_list_warning_reads_the_message`, `columns_belong_to_the_folder_they_were_arranged_in`, `the_numbers_the_targets_ask_for`, `the_list_reads_only_memory`, `undo_send_is_where_somebody_looks`, `one_sort_is_checked`), on the green and the fix commits |
| `guards/guards.toml`, `docs/changelog.md`, `docs/KEYBOARD_SHORTCUTS.md` | no scoped target; rode the green commit, and the whole-tree guards read all three |
| `docs/comparison.md`, `docs/privacy.md`, `docs/USER_GUIDE.md`, `.planning/intel/built-and-left.md`, `.planning/codebase/INTEGRATIONS.md`, `docs/changelog.md` | `docs_only`: formatting, clippy and the document-reading targets, `wired` among them (77 passed) |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `7c528ab4`, output to a file and the exit status read
directly, never piped: exited 0 in 390 s on its first run, 7,872 passed and none failed over 67
result lines, the release build included; 2 more than 09-09's 7,870, which is this plan's two.
`main`'s hook ran `all` again on the merge: 7,872 and none failed, 385 s. Ledger 374's keyring
race did not appear on either run.

## Guard records

867 records by the TOML reader before, 868 after; census 802 + 65 before, 802 + 66 after, the line
at `guards/guards.toml:84` moved in the green commit. One new, measured through
`scripts/guards.sh --remeasure`, `WIXEN_TEST_THREADS` untouched.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| a chosen folder of messages is handed to the import worker | `wx_app.rs`, `wired` | the handler keeps its picker and drops the hand-over, `let _ = account;` | 2 | rebuild 12 s, run 2 s |

**What the remedy found.** The count check fired once, on task 1's red, naming the 17 records that
name `wired.rs`; run once after the green, 5 min 24 s in all, rebuilds 14 to 25 s and runs 2 s each.
All 17 exactly what they said; counts written at 77. `wired.rs` is named by 18 records now and
`wx_app.rs` by 57 (the README's 14 and 50 were 17 and 56 at `3231dd4e`; 09-08 said so). The count
check fired on nothing after the remedy.

## Premises the tree contradicted

1. **The plan's letter was taken.** "Import a &Folder of Messages..." claims Alt+F, and Fetch
   Missing Message Text has it on the File menu (`&Fetch Missing Message Text (experimental)`).
   The item is `Import a F&older of Messages...`; O was free. Found by listing the File menu's
   labels before adopting the plan's; `test_no_two_items_on_one_menu_claim_the_same_letter` in
   `tests/wired.rs` reads every menu and would have refused the plan's spelling at the commit,
   which the green commit's message and a code comment wrongly denied (Deviations 4).
2. **Import Mailbox had no row on the shortcuts page.** The plan said "a row beside Import
   Mailbox's"; `grep -n -i 'mailbox' docs/KEYBOARD_SHORTCUTS.md` at `3231dd4e` finds one line, in
   prose, and none of Import Mailbox, Export Mailbox or Import PGP Private Key had a row. Four
   rows added.
3. **`wired.rs` is named by 17 records at `3231dd4e`, not the plan's 14, and holds 75 by the count
   check's count, not 72**; `wx_app.rs` by 56, not 50. 09-08's summary had already said so.
4. **The older wired.rs reading read `import_a_mailbox`'s body for the worker's shape**, so
   splitting the worker start out of it would have blinded that reading. It was re-pointed at the
   shared function in the red commit and named in the trailer, rather than left to be found green
   over nothing.

Premises 1 (the picker cannot return a directory; the folder branch exists at `:189-197` and `:405`),
2 (Thunderbird's layout unrecognised, `grep -n 'sbd\|msf' src/service/mailbox_archive.rs` finding
nothing), 3 (the four pages and the guide's zero) and the shape of `what_was_chosen(true, ...)`
held exactly.

## Deviations from plan

**1. [Decision] The letter O, not F.** Premise 1. Ledger 511.

**2. [Rule 2] Four shortcuts rows, not one.** Premise 2. A page that lists the File menu and
omits the four commands that move mail in and out is a page that says they do not exist. Ledger 511.

**3. [Decision] The handler split into three shared functions and the older reading re-pointed.**
Premise 4 and the key decision. Ledger 511.

**4. [Rule 1] The green commit said no check reads menus, and one does.** The first collision test
found, `test_no_two_controls_in_one_dialog_claim_the_same_alt_key`, says in its own comment that
it cannot see menus; that was taken as the answer for the file, and the code comment and the
commit message said so. The hook's run then listed `test_no_two_items_on_one_menu_claim_the_same_letter`,
which reads every menu. The comment was corrected at `7c528ab4`; the commit message stands as
written and this paragraph is its correction. Observation 621 in the skill log records the shape.
Ledger 511.

**5. [Sequencing] The gathered Known limitations paragraph was drafted during task 1 and
withdrawn.** So task 1's commit carried only its own words; it went in with task 2 as the plan
said. Ledger 511.

**6. [Process] No tracked file was edited by a script.** Every edit to a tracked file went through
Read then Edit or Write; `cargo fmt` ran before each code commit; carriage returns measured with
`tr -cd '\r' | wc -c` on every changed file before each commit and on the ledger after it, zero on
each; no em-dash in any file this plan wrote, measured with `grep -c` for the byte sequence. The
exception set for scripted edits on a tracked file is zero, as 09-07, 09-08 and 09-09 left it.
Commit messages were written to the scratchpad and passed with `-F`; the remedy, the two whole
gates and the merge wrote to a file and the exit status was read directly. `git commit` and
`git merge`, not `gsd-tools query commit`. Nothing was sent to any window; no binary was started;
the tester's profile was not read. `Cargo.toml` untouched; no package added; `WIXEN_TEST_THREADS`
at its default. The ledger through `gsd-tools windows append`, both halves, no backslash in any
description.

Everything else executed as written.

## Threat register

T-09-33: the folder goes to `fill_folders_from` unchanged, so `mailbox_archive::opened`'s
`HowMuchToAllow` bounds the walk (most things in it, most held while looking it over, 32 folders
deep), and the work runs on the same `spawn_blocking` worker; `test_importing_mail_reads_the_list_back_before_it_finishes`
holds `spawn_blocking` in `mail_brought_in_from`. T-09-34: the changelog's Known limitations, the
guide's second bullet and ledger 510 say what a Thunderbird folder becomes. T-09-35: each of the
four pages and the guide say no real Outlook data file has been through the reader, and the
changelog's gathered paragraph says it again. T-09-SC: no package added. New surface outside the
register: none; a chosen folder's file names go through the same `the_folder_named_by` rules an
archive's do.

## Ledger

`.planning/WINDOWS.md` 509 to 511 written through `gsd-tools windows append`, both halves, no
backslash in any description; `test_both_halves_of_the_ledger_say_the_same_thing` green after.
508 before, 511 after; 480 open before, 483 after.

| id | kind | what |
|---|---|---|
| 509 | unrun-verify | nobody has run Import a Folder of Messages by hand and no Thunderbird profile has been read; FOUND-11's `[S]`, the tester's |
| 510 | todo | recognise Thunderbird's `.sbd` and `.msf` layout in the folder import; later work with #53's points 4 to 6 |
| 511 | deviation | the five departures above |

## Known stubs

None. `import_a_folder_of_messages` has one caller, the `ID_IMPORT_A_FOLDER_OF_MESSAGES` arm;
`an_account_to_import_into` and `mail_brought_in_from` are called by both pickers; `refuse_to_import`
by both pickers and by `an_account_to_import_into`.

## The phase's closing read

Read on 2026-09-17 against `main` at `8eba6a38`, every summary of the phase at `status: complete`,
each `[D]` line against the summary's coverage block and the tree. The ticks and the reasons are
written into `.planning/REQUIREMENTS.md` under each requirement and in its traceability row, and
the roadmap's phase 9 entry and progress row, in the commit that lands this summary.

### The twelve requirements

| Requirement | Plan, merge | Read |
|---|---|---|
| FOUND-01 | 09-01, `c0606807` | Ticked by 09-01 on the tree side; the dispatch is Pratik's (ledger 483) |
| FOUND-02 | 09-02, `a3554483` | Ticked. Two `[D]` lines: two `spellcheck` tests for the resolver; `tests/the_language_the_screen_shows_is_the_one_used.rs` for the screen. `[S]`: his profile holds a hand-set `en-US` (484) |
| FOUND-03 | 09-02, `a3554483` | Ticked. Two `[D]` lines: three tests across `bodies`, `searching` and `long_text` with `strip_markup` gone; two `bodies` tests and a run of the binary for the once-only repair. `[S]`: his cache not opened by this build (485) |
| FOUND-04 | 09-03, `f58b9271` | Ticked. One `[D]` line: `tests/undo_send_is_where_somebody_looks.rs`, three readings |
| FOUND-05 | 09-03, `f58b9271` | Ticked. Two `[D]` lines: three `answering` tests with "has been told" in no production line; the comment reading and `grep -c 'Alt+E'` at 0. `[S]`: ledger 155's listening question |
| FOUND-06 | 09-04, `c928cae4` | Ticked. Two `[D]` lines: `tests/the_sort_controls_sit_together.rs`; `test_every_setting_somebody_can_change_is_offered_by_a_screen` green with the control on Compose. `[S]`: the listening pass (487) |
| FOUND-07 | 09-04, `c928cae4` | Ticked. One `[D]` line: `tests/one_sort_is_checked.rs` and `tests/one_sort_is_checked_on_a_live_menu.rs`. The look at the running program is 488 |
| FOUND-08 | 09-05, `165fd811` | **Open on its second `[D]` line.** First by two `scan_target` tests and each editor started here; the naming half of the second by `tests/checkbox_labels.rs`; third by `tests/no_label_is_only_a_space.rs`. The walk half waits: `msaa-names.ps1` left with -1073740791 on every run here (390, 489) and the Accessibility workflow has not run because nothing has been pushed. Closes when CI has walked the five editors |
| FOUND-09 | 09-06, `de58771a` | **Open on its third `[D]` line.** First by `scripts/uia-events.ps1` and the capture; second by `tests/the_settings_tab_row_says_each_tab_once.rs` and the capture taken again. The `nvda-tests` case is written and syntax-checked and has not run, because it runs only on the NVDA workflow at a push of `main` (492) |
| FOUND-10 | 09-07, `f990d023` | Ticked. Three `[D]` lines: four `reading_a_message` tests and the `wired.rs` reading over six surfaces; five `encryption_tests` in `reader_text`; `THE_SURFACES` with its companion and three dated changelog entries. `[S]`: no real key or signed message (495) |
| FOUND-11 | 09-08, `06fdc9b7`; 09-10, `8eba6a38` | Ticked. Four `[D]` lines: 09-08's two by `import_tree`, `importing_an_outlook_data_file`, `importing_messages`, `export_tree` and four `wired.rs` readings; this plan's two by the folder readings, the dated entry, the Thunderbird line, the four pairs and the guide. Both `[S]` lines stay: points 4 to 6 later work with the issue open, no real data file read (499) |
| FOUND-12 | 09-09, `a8b26596` | Ticked. Two `[D]` lines: the before rows at `d169df71` and the line's test; the "built when its tab is first shown" arm with its three harness tests, the after rows, and the settings check green. `[S]`: whether it feels immediate (504, 507) |

Nine ticked by this read, one already ticked by 09-01, two open. Rule 11 of the plan's brief was
applied as written: a requirement is ticked only when a plan closed every `[D]` line, and the two
whose last `[D]` line names a run on CI stay open until that run.

### The roadmap's nine criteria

| Criterion | Read |
|---|---|
| 1 | Closed by 09-01 on the tree side; the dispatch open |
| 2 | Closed by 09-02: the resolver, the screen, the snippet through the reader, stored ones put right once |
| 3 | Closed by 09-03: Undo Send first on Edit with its key; the countdown from the composer's own value; Alt+H |
| 4 | Closed by 09-04: Then by after Default sort order; Cc and Bcc lines on Compose; one sort checked |
| 5 | **Open on its walk clause**, as the criterion itself says it stays until one of the two machines has walked the five editors; the scan-target, naming, spacer and reading clauses closed by 09-05 |
| 6 | **Open on its transcript clause**; the capture and the handler clauses closed by 09-06 |
| 7 | Closed by 09-07: one function, six surfaces, the preview's bar, the guard naming each, two entries dated |
| 8 | Closed by 09-08 and this plan: `*.pst` in the picker and the four kinds through the writers with the closing sentence; Save As as `.eml` with the reader's Save Attachment named on the page; a folder through a directory picker on its own item with Thunderbird said unrecognised; the four documents dated and the guide naming the three commands |
| 9 | Closed by 09-09: the before rows, the after rows beside them, every setting still on the screen |

### The progress table

Phase 9 is marked `10/10`, Complete, 2026-09-17, because every one of the ten summaries is
`status: complete`. Criteria 5 and 6 are open on one clause each and say so in the row and in the
plan list, and FOUND-08 and FOUND-09 are unticked with the same reason; nothing about that changes
when the table says complete, and the row is where the next person reads it.

### For a person

**What changed in this build over 0.125.1.** The number is `1.0.0-alpha.1`, and a settings file
says which build last wrote it. Spelling is checked in the language your computer is set to, and
Settings shows it. Snippets are the words of a message and never its stylesheet, and the ones
already stored have been put right. Undo Send is the first item on the Edit menu, and answering a
meeting says the same countdown any other send says, never that the organiser has been told while
the answer is held. Then by sits after Default sort order, Cc and Bcc lines is on the Compose tab,
and the Sort submenu shows one tick. Every checkbox in the contact, condition, filter, signature
and account editors is named, and the empty spacers before controls are gone. Arrowing along the
Settings tab row raises one focus event per key instead of two. Every place that shows a message
tries the PGP key, says what the S/MIME envelope says and carries the signature bar, the preview
pane included. File, Import Mailbox takes an Outlook data file, File, Save As saves the message
you are on as `.eml`, and File, Import a Folder of Messages takes a folder. Settings opens in under
half a second instead of over two.

**Which issues closed.** #21, #32, #34, #36, #39, #44, #46, #51 and #56 are closed with their merge
commits. Four are advanced and open: #33 for its NVDA transcript on CI, #40 for its points 1 to 4
and 6, #42 for the MSAA walk on CI that 09-05 could not run here, and #53 for points 4 to 6 with
1, 2, 3 and 7 done across `06fdc9b7` and `8eba6a38`. (The phase README's table says 09-05 closes
#42; 09-05's summary and its comment on the issue say advanced, and the issue is open.)

**What only a person can settle.** Whether the tester's own profile shows English (United States)
and reads snippets as words; whether the Reading tab reads as one group; what NVDA says on the
signature and contact editors; whether each Settings tab is heard once and whether Settings feels
at once; whether anything spoken after a mistaken Accept points at Undo Send in time; and
everything about a real correspondent's key, a real signed message, a real Outlook data file and a
real folder of saved mail. Each is a ledger entry (484, 485, 487, 488, 490, 492, 495, 499, 504,
507, 509) and a requirement's last `[S]` line.

**What is next.** A push of `main` runs the Accessibility workflow and the NVDA workflow, which is
what FOUND-08's walk clause and FOUND-09's transcript clause wait for; that is Pratik's, as every
push is. A new installer is the next thing to hand him, built from `main` at or after `8eba6a38`,
so the second day of testing meets this program rather than 0.125.1. The next phase starts from
the README's groups 3 to 7 of Pratik's order, group 3 first: all the mail, and what is said while
it comes (#20, #23, #24, #37, #38).

## Self-Check: PASSED

`src/presentation/wx_app.rs`, `tests/wired.rs`, `guards/guards.toml`, `docs/changelog.md`,
`docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`, `docs/comparison.md`, `docs/privacy.md`,
`.planning/intel/built-and-left.md` and `.planning/codebase/INTEGRATIONS.md` are on disk and
changed; commits `8ac253e0`, `6c5fc1e9`, `857fadbc`, `7c528ab4` and `8eba6a38` are in `git log`;
`guards/guards.toml` holds 868 records by the TOML reader; `.planning/WINDOWS.md` holds 511 in
both halves.
