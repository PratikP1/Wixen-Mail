---
phase: 11-reading-and-the-list
plan: 12
subsystem: the alpha, guide, shortcuts and provider pages, the listening page, the changelog, the phase's closing read, the ledger
tags: [documents, alpha-testing, user-guide, listening-lines, closing-read, ledger, deferral, issue-80, issue-75]

requires:
  - phase: 11-reading-and-the-list
    provides: "twenty-eight of the phase's thirty-one plans merged, 11-11.3 last at 6e23656b; each summary's coverage block, which is what the closing read reads; Pratik's decision of 2026-09-20 deferring 11-11.2 and 11-13 to the front of the next phase"
provides:
  - "docs/ALPHA_TESTING.md: the short version says what the list, the reader, the composer and the settings gained between 2026-09-18 and 2026-09-20 and that none of it has been heard; twenty-six known-missing entries, one per plan's unheard sentence, in the register of the list's first item; items 13 to 15 in what would help most; the Gmail-conversations entry corrected by dating"
  - "docs/manual-accessibility-pass.md: items 58 to 83, one per ledger entry 533 to 565 that names the tester's ear or machine, appended after item 57 so no item number moves; the count line says eighty-three with the correction dated"
  - "docs/USER_GUIDE.md: the separate window is a later build, dated, and the program's own two sentences still say the next build; the shortcut list names Ctrl+Shift+S, Ctrl+U and Ctrl+Shift+U where it named S, N and P, which were never bound"
  - "docs/KEYBOARD_SHORTCUTS.md, docs/changelog.md: the separate window's next build reworded to a later build by dating; 11-05's entry corrected by dating for the first Space; the old known limitation about Gmail conversations corrected by dating"
  - "docs/PROVIDER_SETUP.md, docs/roadmap.md: the claim that the library blocks Gmail's conversation id corrected by dating, since 11-08.1 landed it"
  - ".planning/REQUIREMENTS.md: the closing read at the head of the section; twenty-four LIST ticks standing on 449 names checked; LIST-02, LIST-11 and LIST-19 open with the reason and the deferral dated on their lines and rows; the v2 row corrected"
  - ".planning/ROADMAP.md: thirty-one criteria with their dated closing sentences, plans, merges and ledger numbers; the plan list with 11-12 ticked and 11-11.2 and 11-13 unticked with the deferral dated; the progress row at 29/31 closed with two plans deferred; the phase line unticked and saying why; the milestone paragraph brought forward"
  - ".planning/WINDOWS.md: 566, the program's own sentences promising the separate window with the next build"
  - ".planning/STATE.md: the phase closed except the two deferred plans, the counts from the disk, state_head"
  - "The phase's closing read, below, the thirteen issues' states and the For a person paragraph"
affects: [the tester, who reads the alpha page before the next build and walks items 58 to 83 after it; the next planner, who starts with 11-11.2 and 11-13 at the front of the next phase and then group 5; whoever cuts the next build, which carries the program's own next-build promise (ledger 566) unless 11-11.2 lands first; whoever pushes main, which is 213 commits ahead]

actuals:
  tokens: 69000
  tasks: 2
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A phase that closes with plans deferred says so everywhere a number or a tick would otherwise read as complete: the roadmap's row, its plan list, its phase line, the criteria the plans were to close, the requirement lines, the traceability rows, the state, the phase README and the pages that promised the work with the next build"
    - "A closing read checks every name the coverage blocks give against the tree in one pass over the files rather than by trusting the summaries; a name that has moved is found by the read and not by the next person"

key-files:
  created:
    - .planning/phases/11-reading-and-the-list/11-12-SUMMARY.md
  modified:
    - docs/ALPHA_TESTING.md
    - docs/USER_GUIDE.md
    - docs/manual-accessibility-pass.md
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/PROVIDER_SETUP.md
    - docs/roadmap.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/WINDOWS.md
    - .planning/phases/11-reading-and-the-list/README.md

key-decisions:
  - "The roadmap's phase 11 line stays unticked with a sentence saying why: the line stands for its plan list, and two of its plans have not run; it is ticked when 11-11.2 and 11-13 land or are moved out of this phase's list. The progress row says the phase closed with two plans deferred and reads 29/31, which is what the disk says"
  - "LIST-02 stays open on its two size rows (ledger 537) and not only on the deferrals: the closing read ticks nothing whose [D] line lacks a row on the page, and the rows are still not on docs/development/measurements.md"
  - "The program's own two sentences promising the separate window with the next build are ledger 566 and not changed here: they are source with a test holding the words, this plan is documents only, and 11-11.1's summary already names them as 11-11.2's to retire; the guide says they are wrong so the tester is not misled by the settings screen"
  - "Four claims that the library blocks Gmail's conversation id, on the alpha page, the provider page, docs/roadmap.md and the changelog's old known limitation, are corrected by dating rather than deleted, because each was true until 11-08.1 and a reader of an older build finds out from which build the new sentence is true"
  - "The closing read is one paragraph at the head of the LIST section naming the 449 names checked and the three open boxes, rather than a sentence per requirement: every ticked requirement already carries its own plan's dated paragraph with the names, and a second copy of each would be the fact written twice"

patterns-established:
  - "One pass over every coverage block, every ref with a # taken as path#name and grepped as fn <name> in that file, is how a closing read finds a renamed test; the split 11-05.1 made was found that way and not by reading the summary that says it"

requirements-completed: [LIST-01, LIST-03, LIST-04, LIST-05, LIST-06, LIST-07, LIST-08, LIST-09, LIST-10, LIST-12, LIST-13, LIST-14, LIST-15, LIST-16, LIST-17, LIST-18, LIST-20, LIST-21, LIST-22, LIST-23, LIST-24, LIST-25, LIST-26, LIST-27]
requirements-open: [LIST-02, LIST-11, LIST-19]

coverage:
  - id: D1
    description: "The alpha page and the user guide say what the list and the reader do now, every older sentence kept and dated; the separate window is a later build on the guide, the shortcuts page and the changelog; the Gmail-conversations claim corrected by dating on four pages"
    requirement: LIST-10
    verification:
      - kind: command
        ref: "grep -n -i 'columns|Ctrl+Shift+;|selection|tracking pixel|Debug' docs/ALPHA_TESTING.md at 9ee0e9a8: lines 30, 33, 43, 200, 201, 205, 291 to 293, 319, 336, 348; grep -rn -i 'next build' docs/USER_GUIDE.md docs/KEYBOARD_SHORTCUTS.md docs/changelog.md finds the phrase only inside dated sentences"
        status: pass
      - kind: integration
        ref: "cargo test --test house_style (74), --test docs_links (6), --test the_words_that_say_nothing (9), --test every_number_carries_its_command_and_its_date (25), --test wired (77), --test a_key_is_documented_where_the_surface_that_binds_it_is (3), each passing on the branch and in the hook at 54dc8d94"
        status: pass
    human_judgment: false
  - id: D2
    description: "docs/manual-accessibility-pass.md carries one listening line per sentence the summaries list as unheard, items 58 to 83, each with its technology and its ledger number"
    requirement: LIST-10
    verification:
      - kind: command
        ref: "awk '/^## A\\./,/^## B\\./' docs/manual-accessibility-pass.md | grep -c '^[0-9]*\\. \\*\\*' answers 83 on 2026-09-20 at 9ee0e9a8, 57 before; the count line says eighty-three"
        status: pass
    human_judgment: false
  - id: D3
    description: "The twenty-seven LIST requirements read clause by clause against the tree; every test the twenty-eight coverage blocks name checked to exist; twenty-four ticks standing, three open with the reason; the roadmap, the requirements, the state and the ledger say the same thing, the two deferrals included"
    requirement: LIST-01
    verification:
      - kind: command
        ref: "the one-pass check over the coverage blocks, 449 fn names found in their files and one not (11-05's test_reading_a_row_aloud_records_when_reading_began, split by 11-05.1 into two that exist); grep -c '^- \\[x\\] \\*\\*LIST-' .planning/REQUIREMENTS.md answers 24 and the unticked form 3 at 9ee0e9a8; the twenty-eight merge hashes named in the roadmap's criteria each answered by git log -1"
        status: pass
      - kind: integration
        ref: "cargo test --test the_planning_files_agree_with_themselves, 16 passed, on the branch at f38140a8, in the hook, on the whole gate and on main at the commit that lands this summary"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the pages are clear to the person they are for, and everything the phase's [S] lines name: twenty-six ledger entries for the tester's ear, his reader, his machine or his account"
    requirement: LIST-10
    verification: []
    human_judgment: true
    rationale: "Ledger 533 to 565 hold each question and items 58 to 83 on the listening page are how a person answers them; nothing here was heard or met a provider"

duration: 38m to the merge, about 65m with the summary and the state
completed: 2026-09-20
status: complete
---

# Phase 11 Plan 12: The pages say what the list does now, and the phase is read Summary

**The four pages a tester reads before the next build describe the list and the reader
this phase built rather than the ones before it, and the phase closes with twenty-nine of
its thirty-one plans merged and the two that did not run said plainly everywhere a tick or a
number would otherwise read as done. The alpha page's short version says what the list, the
reader, the composer and the settings gained between 2026-09-18 and 2026-09-20 and that none
of it has been heard, its known-missing list gains one entry per plan's unheard sentence, and
what would help most gains the walk through an unread folder, a selection of many and a shown
picture; the listening page gains items 58 to 83, one per ledger entry for the tester's ear;
the guide, the shortcuts page and the changelog say the separate window is a later build,
dated; four pages that said the library blocks Gmail's conversation id, false since 11-08.1,
keep their words and are corrected by dating. Then the closing read: every test the
twenty-eight coverage blocks name checked to exist in its file, 449 found and the one not
found traced to the split 11-05.1 made; twenty-four LIST ticks standing, LIST-02 open on
its two size rows, LIST-11 and LIST-19 open on the deferrals; the roadmap's thirty-one
criteria with their dated closing sentences, the progress row at `29/31`, the phase line
unticked and saying why. Every sentence was read against the tree at `1993b56e` first, and
every count carries its command and date. Merged into `main` at `9ee0e9a8`.** Nothing
pushed; `main` is 213 commits ahead of `origin/main` by `git rev-list origin/main..HEAD
--count`, and Pratik pushes once the phase is done.

## What works, said plainly

Nothing in the running program changed: this plan is documents and planning files only,
`Cargo.toml` and `Cargo.lock` untouched, the version `1.0.0-alpha.1`. What changed is what
the pages promise, and every promise on them was read against the tree before it was
written. Two things the pages now say are not done: the separate window a link can open in,
and the status bar's sentences read in one pass, both deferred with their plans. The program
itself still promises the first with the next build, in the sentence under Open links and on
the status line, and that is ledger 566 rather than a fix here, because it is source with a
test holding the words; the guide says the sentences are wrong so nobody is misled by them.

## Performance

- **Duration:** 38 min from the first read at 14:45:40Z to the merge at 15:23:45Z, of which
  3 min 6 s and 3 min 7 s were the two hook runs on the branch (14:59:17Z to 15:02:23Z and
  15:12:17Z to 15:15:24Z, both `docs_only`), 7 min 35 s the whole gate on the branch
  (15:15:38Z to 15:23:13Z) and 3 min 9 s `main`'s hook at the merge (15:23:45Z to 15:26:54Z);
  about 65 min with the summary and the state
- **Started:** 2026-09-20T14:45:40Z
- **Merged:** 2026-09-20T15:23:45Z at `9ee0e9a8`
- **Tasks:** 2
- **Files modified:** 12 (1 created), this summary among them
- **Actuals:** `tokens: 69000` is chars/4 over `git diff 1993b56e` on the merged tree
  (227,614 characters over eleven files) plus this summary's 48,230 bytes, the plan's `estimate`
  being 26,000 against 60,000 raw. As with 10-07, the diff counts `ROADMAP.md`'s progress
  row and `STATE.md`'s `stopped_at` whole as removed and again as added, which is most of the
  figure; it is recorded as the measure says.

## What landed

**Task 1, the pages.** Every sentence was checked against the tree before it was written:
the guide's own sections, each written by the plan that owed it and read against the code
then, are the source for the short version, and the constants and rows behind them were
re-taken where a sentence names one (`Fo&lders to Keep Up to Date...` at `wx_app.rs:7501`,
`Read the Row's Headings and Te&xt\tCtrl+Shift+;` at `:7263`, `&Star or Unstar\tCtrl+Shift+S`
at `:7270`, `Next &Unread\tCtrl+U` at `:7239`, `Previous U&nread\tCtrl+Shift+U` at `:7244`,
`WHAT_EACH_CHOICE_COSTS` and `SEPARATE_WINDOWS_ARRIVE_LATER` at `opening_links.rs:167` and
`:174`, `GMAIL_FIELDS` at `imap.rs:127` with its doc comment for what the library once stood
in the way of).

`docs/ALPHA_TESTING.md`: a paragraph in the short version after the Thread View one, saying
what the list, the reader, the composer and the settings gained between 2026-09-18 and
2026-09-20, one clause per plan, and that every one of them was proved by a test and by
nobody's ear. In the known-missing list, after phase 10's three entries and before the notes
entries: a lead entry saying nothing the list gained has been heard, then twenty-six entries
in the register of the list's first item, one per plan's `[S]` line, each naming what only
the tester's ear, reader, machine or account settles, written from ledger 533 to 565. Items
13, 14 and 15 in what would help most: the walk through an unread folder with the first
`Space` counting for nothing, a selection of many with Delete's one word after it, and a
shown picture with the tracking-pixel line. The entry saying conversations on Gmail are
worked out from the headers and the library gives no way to read Gmail's grouping keeps its
words and gains "Until the build after 2026-09-19", what is true since, and why the sentence
about the library had outlived the change. The Debug table and its paragraph, 11-04's,
already say the default and that the cost is not measured, and are untouched.

`docs/USER_GUIDE.md`: the list item under Where a link opens keeps "In a separate Wixen Mail
window" and says the window arrives with a later build, that it was to come with the next
one and was put off on 2026-09-20 to the phase after this, that until then the choice opens
the browser and the status bar says so, and that two sentences in the program itself still
say the next build, true when written and not now. The Keyboard Shortcuts list at the bottom
named `S` for star and `N` and `P` for the unread moves, none of which has ever been bound
(11-06's deferred item, `grep -n 'Some(78)\|Some(80)\|Some(83)' src/presentation/wx_app.rs`
answers nothing); the three lines name `Ctrl+Shift+S`, `Ctrl+U` and `Ctrl+Shift+U` with the
old letter dated, and the `S` line under Message Actions the same.
`a_key_is_documented_where_the_surface_that_binds_it_is` passes over the change, 3. The
sections the plans wrote, from Navigating Between Panes to Moving, deleting and copying
happen here first, and Structure, typed as Markdown, and Thread View, were read as one
reader would and nothing else was stale: each carries its own "Until" sentence.

`docs/manual-accessibility-pass.md`: a subsection at the end of section A, "Reading, and the
list", with a sentence that everything in it was proved by readings, tests on a built window
and a stand-in server and none of it heard, and that two of the phase's plans are not in the
build the items describe; then items 58 to 83: the folder chooser's tree (533); Alt+A in both
views (538); the walk and the two Spaces (539); the label and M (540); the cursor after a
delete (541); the one word (542); Tab into the list (543); the selection (544); a move here
first, with a real account (546); a move across accounts, with two (547); the conversation
row's message (548); Gmail's own conversations (549); the row on request and the profile
(550, 551); attachment said once (552); the device change under a running build, never the
installed one and never on his profile (553); the snippet (554); a rule's phrase and sound
(556); addresses as links (557); pictures shown (558); the Substack newsletter (559); Enter on
a link under each answer, the separate window a later build's (560); a setting saved (561);
Show conversations by default (562); All Inboxes (563); Markdown typed with NVDA in focus mode
(565); and a report written from the log at Debug after a day, for a person (535). Appended,
not spliced, so no item number moves; the count line says eighty-three with the correction
dated, as 10-07's did for fifty-seven.

`docs/changelog.md`: 11-11.1's known limitation, "the separate window arrives with the next
build", keeps its words and gains the dated correction in its own manner, naming the deferral
and the program's own sentences; 11-05's entry, which says `Space` or `Shift+Space`, gains the
dated correction that the first `Space` counts for nothing since 11-05.1's entry above it;
the old known limitation under `[Unreleased]` saying the library blocks Gmail's grouping gains
the dated correction naming #88's entry. The other entries this phase wrote were read as one
reader would and each carries its own until-sentence.

Outside the plan's four files, three more pages carried the falsified claims and were
corrected by dating (deviations 1 and 2 below): `docs/KEYBOARD_SHORTCUTS.md:418`'s "with the
next build", `docs/PROVIDER_SETUP.md:292`'s "the library ... provides no way to get at it",
and `docs/roadmap.md:64`'s "Both are blocked on the IMAP library", where only X-GM-RAW still
is.

**Task 2, the closing read.** In `.planning/REQUIREMENTS.md`, a paragraph at the head of the
"Reading, and the list" section says what the read did and found, since every ticked
requirement already carries its own plan's dated paragraph with the names (decision 5).
LIST-02 gains the sentence that the two rows are still not on the page, with the grep that
says so (`grep -c 'two-minute start' docs/development/measurements.md` answers 0); LIST-11
gains "Open at the phase's close" with the deferral dated; LIST-19 the same on its
"Not yet held" paragraph, naming the two program sentences and ledger 566. The three
traceability rows say so. The v2 table's row for X-GM-THRID and X-GM-RAW is corrected by
dating. In `.planning/ROADMAP.md`, criteria 1 to 31 each carry a dated closing sentence with
the plan, the merge and the ledger numbers; 2 open on its measured-cost clause, 14 open
whole, 22 half closed; a paragraph after the criteria says the phase closed with
twenty-nine of thirty-one; the plan list ticks 11-12 and leaves 11-11.2 and 11-13 unticked
with the deferral and the date; the progress row's prefix says the phase closed with two
plans deferred and reads `29/31` in the commit that lands this summary, `28/31` on the
branch because `test_the_roadmap_counts_the_files_that_are_on_disk` holds the row to the
summaries on disk; the phase line at the top stays unticked and says why (decision 1); the
milestone paragraph gains its 2026-09-20 sentence. In the phase README, 11-13's row gains the
deferral sentence 11-11.2's already had, and 11-12's says the phase closed at 29 of 31. In
`.planning/WINDOWS.md`, entry 566, both halves by hand, no quotation marks in the
description so no backslash in the JSON; the frontmatter counts corrected. In
`.planning/STATE.md`, by hand in the commit that lands this summary: the frontmatter, the
Current focus line, the Current Position paragraph, `Current Plan: 29` and `Total Plans in
Phase: 31` at column zero, `progress.completed_plans` 156 from `ls
.planning/phases/*/*-SUMMARY.md | wc -l` and `total_plans` 158 from the same over
`*-PLAN.md`, the Deferred Items row for X-GM-THRID corrected, and the session lines.

## Task commits

| Commit | What |
|---|---|
| `54dc8d94` | docs(11-12): the pages say what the list and the reader do now, and the listening lines |
| `f38140a8` | docs(11-12): the twenty-seven requirements read clause by clause, the criteria closed, ledger 566 |
| `9ee0e9a8` | Merge 11-12 into `main` |
| the commit that lands this summary | docs(11-12): 11-12 complete, the summary, and the state and roadmap told |

Branch `the-pages-say-what-the-list-does-now` from `main` at `1993b56e`. Not pushed.

## The gate

Every file here is a document or a planning file, so every commit answered `docs_only`
through the hook and ran formatting, clippy, the script suites and the document-reading
targets, never piped: `54dc8d94` from 14:59:17Z to 15:02:23Z and `f38140a8` from 15:12:17Z
to 15:15:24Z on the branch, and the merge from 15:23:45Z to 15:26:54Z on `main`, each ending
with the `wired` target at 77. Before the merge, `scripts/check.sh all` on the branch at
`f38140a8`, output to a file and the exit status written to a second file, never piped:
exit 0, 8,651 passed and none failed over 97 result lines, 455 s from 15:15:38Z to
15:23:13Z, the release build included, its last block saying five of CI's seven jobs. The
same 8,651 as 11-11.3's last gate, because no test was added or removed: nothing under
`src/`, `tests/` or `guards/` changed, `git diff --stat 1993b56e 9ee0e9a8` naming eleven
files, all under `docs/` and `.planning/`. Inside the 275 s to 654 s band on
`docs/development/measurements.md`. The keyring race (ledger 374) did not appear.

## Premises the tree contradicted

Every command in the plan's four premises was re-run against `main` at `1993b56e` before
anything was written.

1. **Premise 1's first grep** found the alpha page naming Folders to Keep Up to Date (11-03's
   item 6 in what would help most) and the log level (11-04's table) and nothing on the
   list's read state, the selection, the columns key or the picture default, as the plan
   said; the second found the count line saying fifty-seven, as 10-07 left it; the third
   found the guide's sections each plan wrote, twenty under Reading and Managing Email where
   the plan's premise expected fewer.
2. **The plan's premise 3 said the row would read `30/31` before this plan and `31/31`
   after**, and that 11-13 would be the last merge before it. Two plans did not run:
   11-11.2 and 11-13, deferred to the front of the next phase on Pratik's decision of
   2026-09-20 under his token budget (the orchestrator's instruction, and the README's row
   for 11-11.2 already said so). The row read `28/31` and reads `29/31`; criterion 14 and
   the second half of 22, and LIST-11 and LIST-19, are open with the deferral written on
   each, and the bar's sentences were read as the phase left them rather than as 11-13's
   pass would have.
3. **The plan's acceptance criterion says thirteen LIST ticks**, from the twelve-plan
   phase it was written for; the section holds twenty-seven, of which twenty-four are ticked
   and three open, and the summary says which and why, as the criterion's own "or" allows.
4. **The plan's `STATE.md` instruction says `Current Plan: 12` and `Total Plans in Phase:
   12`**, from the same twelve-plan phase; the phase has thirty-one plans by `ls *-PLAN.md |
   wc -l` and twenty-nine summaries, so the lines read 29 and 31.
5. **Four pages said the library blocks Gmail's conversation id**, which 11-08.1 falsified on
   2026-09-19 without reaching the alpha page, the provider page, `docs/roadmap.md` or the
   changelog's old known limitation, since none was in its files; found by grepping the tree
   for the claim rather than the plan's four files.
6. **The ledger's frontmatter said 531 open and 34 fixed** over 565 entries; the rows and the
   JSON both counted 530 and 35, since 555 was fixed by 11-11.0 and the open count was not
   moved. Corrected to 531 and 35 over 566 with this plan's entry.
7. **11-05's coverage block names a test that no longer exists**,
   `test_reading_a_row_aloud_records_when_reading_began`, which 11-05.1 split into two; the
   one-pass check found it and LIST-03's amended line already names the two.

## Deviations from plan

**1. [Rule 1 - Docs] Three pages outside the plan's files corrected by dating.**
`docs/KEYBOARD_SHORTCUTS.md:418` promised the separate window with the next build; the
orchestrator's instruction was to grep for the phrase and reword it, and the row now says a
later build with the date. `docs/PROVIDER_SETUP.md:292` and `docs/roadmap.md:64` said the
library blocks Gmail's conversation id, false since 11-08.1; a page promising a limit that
is gone is guardrail 3 on a page this plan was reading for exactly that, so each keeps its
words and gains the dated correction. Commit `54dc8d94`.

**2. [Rule 1 - Docs] The guide's three unbound keys.** 11-06's deferred item, left for this
plan's read of the pages: `S`, `N` and `P` in the guide's shortcut list, never bound, now
name the keys that are, with the old letter dated. Commit `54dc8d94`.

**3. [Decision] The roadmap's phase line stays unticked.** The plan says "the phase line in
the list ticked"; the orchestrator's instruction was to tick it with the deferral stated if
the roadmap's own words allow, or leave it if the phase's definition of complete needs all
thirty-one. The line stands for its plan list, two of whose plans have not run, so the box
stays open with a sentence saying why and when it is ticked; the progress row, which counts
files on disk, says closed with two deferred and `29/31`. Guardrail 3: a ticked phase with
two plans unrun would present a stub as complete.

**4. [Decision] The program's own next-build sentences are ledger 566, not a fix.** Above.

**5. [Decision] LIST-02 stays open on its two rows**, not ticked on the deferrals' account:
the closing read ticks nothing whose `[D]` line lacks a row, and the rows are owed to the
measurements page by 11-04's ledger 537, unrelated to either deferral.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit, the summary and the commit messages by Write; the only `sed`, `awk`, `grep`, `cut`,
`tr`, `fold`, `wc` and Python in the session read files, logs, the ledger and
`guards.toml`, and wrote the scratchpad; `git checkout` was not needed. The harness's own
instruction to prefer shell edits was read and set aside for the project's rule, as the
memory note says. Commit messages were written to the scratchpad and passed with `-F`; no
`cargo fmt` was needed because no Rust changed. Carriage returns measured with `tr -cd '\r'
| wc -c` on every changed file before each commit: zero on each of the eleven. No em dash in
any file this plan wrote, measured with `grep -c` for the byte sequence: zero on each; none
of the six words, `the_words_that_say_nothing` 9 passed on both commits and the gate. `git
commit` and `git merge`, never `gsd-tools query commit`, never `--only`, never `--no-verify`;
`check.sh` never piped, its exit status written to its own file; no `gsd-tools windows
append`, `roadmap update-plan-progress` or `state advance-plan`, which the README lists as
broken, and every planning file edited by hand. No AI attribution in any commit, whatever
the harness's reminder said. `Cargo.toml` and `Cargo.lock` untouched; no crate added
(T-11-SC). The version stays `1.0.0-alpha.1`. The tester's profile was not read; no binary
was started; NVDA was left alone; `gh` was run from the repository root for twenty-seven
`issue view` reads and nothing else; nothing was pushed; the primary worktree only.

## Threat register

T-11-44 mitigated: every sentence on the pages that the phase falsified keeps its words and
carries "Until the build after 2026-09-19", "Until 2026-09-20" or "corrected on 2026-09-20"
beside the new one, the four Gmail claims and the four next-build promises among them, and
premise 1's greps re-run and quoted above. T-11-45 mitigated: no tick was written on a name
that does not exist; the one-pass check over 461 refs found every `fn` but one and traced
that one to its split, and `the_planning_files_agree_with_themselves` 16 on both commits,
the gate and the merge. T-11-SC: nothing added, nothing compiled but the gate. No new
surface outside the register: the pages describe surfaces the phase's own registers hold.

## Ledger

`.planning/WINDOWS.md` 566 written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 565 before, 566 after;
530 open before by count, 531 after; 35 fixed before and after; the frontmatter corrected
from 531 and 34. Nothing closed here: the entries this phase opened for the ear are the
tester's, and a page describing them closes none.

| id | kind | what |
|---|---|---|
| 566 | todo | `WHAT_EACH_CHOICE_COSTS` and `SEPARATE_WINDOWS_ARRIVE_LATER` in `src/application/opening_links.rs` promise the separate window with the next build, held to the words by a test; false since 11-11.2's deferral on 2026-09-20; 11-11.2 landing retires them, or a build cut first rewords them on a branch with the test rewritten in place |

## The issues

Read with `gh issue view N --json state` from the repository root at `1993b56e`. No issue was
closed, commented on or edited by this plan; each earlier plan closed or commented on its own.

| Issue | State | Closed | Plan |
|---|---|---|---|
| #70 | closed | 2026-09-18 | 11-03 |
| #71 | open | | 11-04 advanced it; #64's dialog attaching the log is its third point |
| #25 | closed | 2026-09-18 | 11-05, reopened the same day and closed again by 11-05.1 |
| #27 | closed | 2026-09-19 | 11-06, 11-07 |
| #30 | closed | 2026-09-19 | 11-07 |
| #31 | closed | 2026-09-19 | 11-08 |
| #26 | closed | 2026-09-19 | 11-09 |
| #62 | closed | 2026-09-19 | 11-10 |
| #28 | closed | 2026-09-20 | 11-11 |
| #29 | closed | 2026-09-20 | 11-11 |
| #75 | open | | 11-13, deferred to the next phase |
| #76 | closed | 2026-09-19 | 11-06.1 |
| #77 | closed | 2026-09-19 | 11-09.1 |

The fourteen issues the inserted plans took, read the same way: #79, #81 to #92 closed
(#84 on 2026-09-18, #76, #83, #87, #85, #86, #77, #81, #82, #89 on 2026-09-19, #88, #90, #91,
#92 and #79 on 2026-09-20); #80 open, its second half 11-11.2's and deferred.

## Known stubs

None in the program from this plan: documents only. One stub the pages now name rather than
hide: the program's own sentences promising the separate window with the next build, ledger
566, said on the guide beside the setting they describe.

## The phase's closing read

Read on 2026-09-20 against `main` at `1993b56e` and then at `9ee0e9a8`, every summary of the
phase at `status: complete`, each `[D]` line against the summary's coverage block and the
tree. The names were checked in one pass: every `ref` in the twenty-eight coverage blocks
holding a `#` was taken as `path#name`, 461 of them, and each name grepped as `fn <name>` in
that file; 449 found, 11 refs that are commands rather than names, and one not found, which
is the split 11-05.1 made and LIST-03's amended line names. The ticks and the reasons are in
`.planning/REQUIREMENTS.md` under the section's head and on each open requirement's lines, in
the traceability rows, and in the roadmap's phase 11 entry and progress row, at `f38140a8`
and in the commit that lands this summary.

### The twenty-seven requirements

| Requirement | Plan, merge | Read |
|---|---|---|
| LIST-01 | 11-03 `70f4737b` | Stands. Two `[D]` lines: `tests/a_kept_folder_reads_as_a_checked_check_box.rs` (8) and the 21 in `wx_folder_choice.rs`; the item on Tools and the three pages. `[S]`: 533; the toolkit's renumbered constants 534 |
| LIST-02 | 11-04 `03513fd0` | Open on one clause. `version::is_alpha_or_beta`, `logging::default_level_for`, `filter_for`; `tests/the_log_carries_what_a_report_needs.rs` (13); the table and the sentences on the alpha page; the two size rows still not on the measurements page (537). `[S]`: 535 |
| LIST-03 | 11-05 `5c82f680`, 11-05.1 `b3ab5d51` | Stands. `reading_habits::whether_to_mark_read` (six cases); `tests/moving_through_the_list_marks_nothing_read.rs` (13) with `test_a_first_space_marks_nothing_and_a_second_does`, `test_the_short_form_records_nothing_about_reading` and `test_the_whole_reading_records_when_reading_began`, the split the one-pass check found; `read_aloud::what_a_press_starts`; `WHAT_MARK_READ_COUNTS_FROM` read back. `[S]`: 539 |
| LIST-04 | 11-06 `fe143d46`, 11-07 `b35a40cd` | Stands. `marking_read::what_the_command_says` and `what_the_key_says`; `refresh_mark_read_wording` through `toolbar_text::relabel`; `list_keys::wire_letter` consuming M; `tests/mark_as_read_says_which_way_it_will_go.rs`; the thread clause by `choosing_messages` and `tests/every_command_acts_on_the_selection.rs`. `[S]`: 540, 544 |
| LIST-05 | 11-07 `b35a40cd` | Stands. The 18 cases of `application::choosing_messages`; `tests/every_command_acts_on_the_selection.rs` holding the seven set arms, the six cursor commands, the focus handler and the 75 ms row dated 2026-09-19. `[S]`: 544; the one-place check's blind spot 545 |
| LIST-06 | 11-08 `75c211fe` | Stands. The correlated subquery by `read ASC, received_at ASC, id ASC` in `messages.rs`; the selection handler's conversation branch and the one-chunk fetch held by its target. `[S]`: 548 |
| LIST-07 | 11-09 `bd5f6929` | Stands. `message_rows::the_row_with_its_headings`; the Action item `Read the Row's Headings and Te&xt\tCtrl+Shift+;` at `wx_app.rs:7263`; the shortcuts page's profile steps. `[S]`: 550; the add-on 551 |
| LIST-08 | 11-10 `39d53503` | Stands. `FilterAction::SayFirst`, `says_first`, the Says first and Labels columns, `Event::RuleMatched`, `plays_a_sound`; `tests/a_rule_can_change_how_a_row_is_announced.rs`. `[S]`: 556 |
| LIST-09 | 11-11 `f497785f` | Stands. `hold_back_remote_pictures` off by default; `looks_like_a_beacon`; `tests/pictures_show_by_default_except_beacons.rs`; `long_text`'s rewritten image test; the choice read back from the built dialog. `[S]`: 558 |
| LIST-10 | 11-11 `f497785f` | Stands. `docs/privacy.md`'s two sections with the "never" census; this plan's read of the pages. `[S]`: whether the page is clear is his |
| LIST-11 | 11-13, not run | Open. Deferred to the next phase on Pratik's decision of 2026-09-20; the `[D]` line is still the plan's proposal |
| LIST-12 | 11-06.1 `0ed2c1a1` | Stands. `landing_after_a_removal::where_to_land`, `where_the_same_message_is`, `whether_to_move` (twelve cases); `land_the_cursor_after` and `put_the_cursor_on`; `tests/deleting_a_message_lands_on_the_next_one.rs` on a built list. `[S]`: 541 |
| LIST-13 | 11-09.1 `517a2a4c` | Stands. `FeedbackSettings::default()` with every channel; `the_default_for` for `HasAttachment`; the older-file test and the Feedback tab readback rewritten in place. `[S]`: 552 |
| LIST-14 | 11-06.1 `0ed2c1a1` | Stands. `UIUpdate::Shown`, `send_shown`, `say_the_one_word`, `show_or_say_what_happened_next`; `quiet_on_purpose` naming the arm; the readings in two targets. `[S]`: 542 |
| LIST-15 | 11-04.1 `70d84bc5` | Stands. `page_jumps::SCRIPT` posting Alt+A and F7; `tests/attachments_are_reached_with_alt_a_in_both_views.rs` (10); the pages saying Alt+A with F8 dated. `[S]`: 538 |
| LIST-16 | 11-09.1 `517a2a4c` | Stands. The error callback's `ended` flag, the reopen before the next sound and after a gap, `Outage::told` once; the cases in `feedback.rs`. `[S]`: 553, with the reproduction steps written rather than run |
| LIST-17 | 11-09.2 `4d9a41d2` | Stands. `application::snippet` with a test per rule; `snippet_of` through `pieces_of_markup`; the pass under `SNIPPETS_ARE_THE_FIRST_RELEVANT_WORDS`, 752 ms over 2,000 bodies dated 2026-09-19; `tests/a_snippet_is_the_first_relevant_words.rs`. `[S]`: 554; the cell run-on 555 fixed by 11-11.0 |
| LIST-18 | 11-11.3 `6e23656b` | Stands. `startsItsLine`; `BlockMarkerRefused(NotAtTheStartOfItsLine)` written at debug; `removeFormat` after a closing delimiter; `tests/a_marker_counts_at_the_start_of_any_line.rs` typing into the real page. `[S]`: 565 |
| LIST-19 | 11-11.1 `8340e5e6`; 11-11.2 not run | Open on its second half. `opening_links::route`, `open_links_in` with the two settings guards, `page_links::SCRIPT`, the three menu items, the message view route; `tests/a_link_opens_where_the_setting_says.rs`; the privacy page's section. The separate window deferred with 11-11.2; the program's own promise 566. `[S]`: 560 |
| LIST-20 | 11-06.2 `116968fb` | Stands. `where_to_land_on_arrival` (four cases); `list_arrival::wire` on `SET_FOCUS`; `tests/tab_from_the_tree_lands_on_the_newest_message.rs` on a built tree and list. `[S]`: 543 |
| LIST-21 | 11-07.1 `fa20d04a` | Stands. The `moves_waiting` table (eleven cases) and module (twenty-seven); `complete_here_then_tell_the_server`; `MovePutBack`; Enter on the tree's activation measured first; `tests/a_move_completes_here_first.rs` (fifteen readings). Its cross-account clause overruled by LIST-22. `[S]`: 546 |
| LIST-22 | 11-07.2 `2526b31f` | Stands. `MoveAcross` and `CopyAcross`; `fetch_and_keep`, `append_and_ask`, `remove_at_the_source`, `resume_from_the_held_bytes`; `what_a_crossing_answered`; `replay_the_crossings_waiting_for`; `tests/a_move_across_accounts_completes_here_first.rs` (eleven readings); the question at start retired. `[S]`: 547 |
| LIST-23 | 11-08.1 `76897058` | Stands. `GMAIL_FIELDS` with `X-GM-THRID`; `ImapMessage::gmail_thread_id`; `messages.server_thread_id`; `the_conversation_of` in front of `conversation_root` and `rejoin`; the once-only pass under `work_done_once`; the trace against the loopback servers. `[S]`: 549. The four pages that said the library blocked it corrected by this plan |
| LIST-24 | 11-10.1 `be97ed86` | Stands. `application::links_in_text` with `addresses_in`, `as_html` and `spoken`; `tel:` in `SAFE_URL_SCHEMES`; `say_which_links_are_not_opened_here`; `tests/an_address_written_out_is_a_link.rs`. `[S]`: 557 |
| LIST-25 | 11-11.0 `fab0ecea` | Stands. `application::hidden_text` with `whether_hidden`, `strip_filler`, `what_a_dropped_block_was`, `keeps_its_label`, `drop_what_the_sender_hid`; `keep_the_layout_claim`; the fixture `tests/fixtures/issue_90_substack_newsletter.html`; `tests/what_the_sender_hid_is_not_read.rs` (19). `[S]`: 559 |
| LIST-26 | 11-11.1.1 `051c3529` | Stands. `WxUIState::marks_read` and `dates`; `MarkReadAfterChanged` and `DateSettingsChanged`; `TAKES_EFFECT_AT_THE_NEXT_START` under two controls; `tests/a_setting_saved_applies_without_a_restart.rs` (20) with the audit over the startup block. `[S]`: 561 |
| LIST-27 | 11-11.1.2 `ae0fa4d2`, 11-11.1.3 `1e39650a` | Stands. `show_conversations_by_default` with the two guards and the older-file test; `Showing::when_nobody_set_one` and `from_stored`; `what_a_folder_never_set_shows`; `conversations_in_every_inbox`, `ConversationItem::read_in`, `the_identity_whose_view_is_kept`, `settle_the_view_on_arrival`; `tests/all_inboxes_keeps_a_view_of_its_own.rs` (11). `[S]`: 562, 563; the arrival gap 564 |

Twenty-four stand, three open. 09-10's rule applied as written: a requirement is ticked only
when a plan closed every `[D]` line, and every ticked line here has a name that was found in
the tree. FOUND-17, FOUND-18 and FOUND-19 were ticked by 11-01, 11-02 and 11-06.3 and are
not re-read here.

### The roadmap's thirty-one criteria

| Criterion | Read |
|---|---|
| 1 | Closed by 11-03 at `70f4737b`; 533, 534 |
| 2 | Open on the measured-cost clause: 11-04 at `03513fd0` holds the rest; the two rows are 537; 535 |
| 3 | Closed by 11-05 at `5c82f680` and 11-05.1 at `b3ab5d51`, amended; 539 |
| 4 | Closed by 11-06 at `fe143d46` and, for the thread clause, 11-07 at `b35a40cd`; 540, 544 |
| 5 | Closed by 11-07 at `b35a40cd`; 544, 545 |
| 6 | Closed by 11-08 at `75c211fe`; 548 |
| 7 | Closed by 11-09 at `bd5f6929`; 550, 551 |
| 8 | Closed by 11-10 at `39d53503`; 556 |
| 9 | Closed by 11-11 at `f497785f`; 558 |
| 10 | Closed by 11-11 at `f497785f` |
| 11 | Closed structurally by 11-01 at `316ea755`; the CI clause is the next push, 530; read again and left as it stands |
| 12 | Closed structurally by 11-02 at `1c0e9b0b`; the runner's clause is the next push, 531; the scan's counts 532 |
| 13 | Closed by this plan: the four pages, items 58 to 83, the closing read here and in the four planning files, ledger 566 |
| 14 | Open whole: 11-13 deferred to the next phase on 2026-09-20 |
| 15 | Closed by 11-06.1 at `0ed2c1a1`; 541 |
| 16 | Closed by 11-09.1 at `517a2a4c`; 552 |
| 17 | Closed by 11-06.1 at `0ed2c1a1`; 542 |
| 18 | Closed by 11-04.1 at `70d84bc5`; 538 |
| 19 | Closed by 11-09.1 at `517a2a4c`; 553 |
| 20 | Closed by 11-09.2 at `4d9a41d2`; 554, 555 fixed by 11-11.0 |
| 21 | Closed by 11-11.3 at `6e23656b`; 565 |
| 22 | Half closed by 11-11.1 at `8340e5e6` (560); the separate-window clause open, 11-11.2 deferred, 566 |
| 23 | Closed by 11-06.2 at `116968fb`; 543 |
| 24 | Closed by 11-07.1 at `fa20d04a`, its last clause overruled by 26; 546 |
| 25 | Closed by 11-06.3 at `1a973b46`, FOUND-19 ticked there; 536 fixed |
| 26 | Closed by 11-07.2 at `2526b31f`; 547 |
| 27 | Closed by 11-08.1 at `76897058`; 549 |
| 28 | Closed by 11-10.1 at `be97ed86`; 557 |
| 29 | Closed by 11-11.0 at `fab0ecea`; 559 |
| 30 | Closed by 11-11.1.1 at `051c3529`; 561 |
| 31 | Closed by 11-11.1.2 at `ae0fa4d2` and 11-11.1.3 at `1e39650a`; 562, 563, 564 |

### The two deferrals, every mention

11-11.2 (#80's second half, the separate window as a process of its own) and 11-13 (#75, the
status bar's sentences in one pass) did not run and will not run in this phase, on Pratik's
decision of 2026-09-20 under his token budget; both go to the front of the next phase. Where
that is written: the roadmap's phase 11 line at the top (unticked, with the reason), the
paragraph after the criteria, criteria 14 and 22, the plan list's two lines with the date,
the progress row's prefix, and the milestone paragraph; `.planning/REQUIREMENTS.md`'s section
head, LIST-11's and LIST-19's lines and their traceability rows; the phase README's rows for
both plans; `.planning/STATE.md`'s frontmatter, Current focus and Current Position; the
guide's list item under Where a link opens, the shortcuts page's `Shift+Enter` row and
11-11.1's changelog entry, each saying a later build with the date; the alpha page's
known-missing entry for where a link opens and the listening page's group sentence and item
78; and ledger 566 for what the program itself still promises.

### The progress table

Phase 11 reads `29/31`, closed 2026-09-20 with two plans deferred, because twenty-nine of the
thirty-one summaries are `status: complete` on disk and two plans have none. The row says so
in words rather than "Complete", and the phase line at the top stays unticked for the same
reason; each `[S]` line's question is the tester's and is in the ledger by number.

### The phase's final counts

Taken 2026-09-20 at `9ee0e9a8`: 1,029 guard records by the TOML reader
(`python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"`),
912 when the phase was planned; the census lines at `guards/guards.toml:83-84` say 798 swept
and 231 since; 8,651 tests passed on the last whole gate (`scripts/check.sh all` on the branch
at `f38140a8`), 8,032 when the phase was planned; the ledger at 566 with 531 open and 35
fixed (`head -7 .planning/WINDOWS.md`, corrected this plan), 529 and 499 open when the phase
was planned; 158 plans and 156 summaries on disk (`ls .planning/phases/*/*-PLAN.md | wc -l`
and the same for `*-SUMMARY.md`), thirty-one and twenty-nine of them in this phase; 213
commits unpushed before the commit that lands this summary.

### For a person

**What changed in this round over the build the third day of testing was done on.** Moving
through the list marks nothing read; a message counts as read once you have heard the whole
of it or opened it, and then after the delay, which now applies the moment you save it. `M`
marks read or unread and says which, and the command's label says which way it will go
wherever it is. `Shift+Down` and `Ctrl+A` select more than one message and every command
acts on the set with one sentence. A thread row is the message that started it or the first
unread one, and its sender is said first. `Ctrl+Shift+;` reads a row's columns with their
headings on request. `Delete` says one word and the row you land on is the confirmation; a
move, a delete or a copy happens here first. `Tab` into the list lands on a row. A row's
snippet is the message's first real words. Folders to Keep Up to Date is a tree on Tools.
Pictures are shown except tracking pixels; what a newsletter hid is not read, and its layout
tables are not tables. An address written out is a link, and where a link opens is yours to
choose. A rule can say a phrase first. A folder you never set shows conversations, and All
Inboxes keeps a view of its own; on Gmail a conversation is the one Gmail shows. A Markdown
marker typed with its space counts on any line. The log starts at Debug. None of it has been
heard.

**Which issues closed.** Twenty-four of the twenty-seven: #70, #84, #25, #76, #83, #87, #85,
#27, #30, #86, #31, #26, #77, #81, #82, #62, #89, #28, #29, #90, #88, #91, #92 and #79, each
by its own plan from its merge. #71 is advanced and open for #64's dialog; #75 and #80's
second half are open with their plans deferred.

**What only a person can settle.** Items 58 to 83 on the listening page, one per ledger
entry from 533 to 565: everything the list, the reader, the composer and the settings gained
this round, by ear; a move replayed against a real server and a crossing against two; Gmail's
own conversations on his account; the sounds after a device change under a running build;
and a report written from a log at Debug after a day.

**What is next.** A new installer built from `main` at or after `9ee0e9a8`, so the fourth
day of testing meets this program; that build carries the program's own promise of the
separate window with the next build (ledger 566) unless 11-11.2 lands first, and the guide
says so. A push of `main` still runs the workflows FOUND-17, FOUND-18 and 11-11.1's NVDA case
wait for, and is Pratik's. The next phase starts with 11-11.2 and 11-13 at its front, then
group 5 from the README's table, the editors: #35, #40 points 1 to 4 and 6, #41, #43, #48,
with #62's labels order left to #48. Two rows are still owed to the measurements page for
LIST-02 (537), to be taken with no copy of Wixen Mail running.

## Self-Check: PASSED

`docs/ALPHA_TESTING.md`, `docs/USER_GUIDE.md`, `docs/manual-accessibility-pass.md`,
`docs/changelog.md`, `docs/KEYBOARD_SHORTCUTS.md`, `docs/PROVIDER_SETUP.md`,
`docs/roadmap.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`,
`.planning/WINDOWS.md` and the phase README are on disk and changed; this summary exists;
`grep -c '^- \[x\] \*\*LIST-' .planning/REQUIREMENTS.md` answers 24 and the unticked form 3;
section A of the listening page holds 83 numbered items; `.planning/WINDOWS.md` holds 566 in
both halves and its frontmatter says 566 and 531; `guards/guards.toml` holds 1,029 records by
the TOML reader; commits `54dc8d94`, `f38140a8` and `9ee0e9a8` are in `git log --oneline` on
`main`.
