---
phase: 10-all-the-mail-and-what-is-said-while-it-comes
plan: 07
subsystem: the alpha, privacy and guide pages, the listening page, the phase's closing read, the ledger
tags: [documents, alpha-testing, privacy, user-guide, listening-lines, closing-read, ledger, issue-29]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "every plan of the phase merged, 10-06 last at 2da50b6b; each summary's coverage block and its For 10-07 section, which are what the closing read reads"
provides:
  - "docs/ALPHA_TESTING.md: the short version says mail comes down whole and keeps coming since the build of 2026-09-18 and that none of it has met a real account; three entries in the known-missing list for the download, the watch and the schedule, and the three announcement levels; two items in what would help most; the build name's shape under how to report"
  - "docs/privacy.md: the mail-provider row names the download; a section under Where your things are says the text of every message is on the disk under the default and what a size does; a section under Who Wixen Mail talks to says what a whole-mailbox download tells a provider and does not (#29's line), that a connection is held open all day, that reads resume on the network without Go Back Online, and what ends up on the disk"
  - "docs/USER_GUIDE.md: How often an account is checked under Account Setup; Getting your mail under Reading and Managing Email; the module-sync line corrected by dating"
  - "docs/manual-accessibility-pass.md: items 44 to 57, one per sentence the phase's summaries list as unheard, each with its technology and its ledger entries; the count line says fifty-seven"
  - "docs/changelog.md: 10-03's corrected line names the entry under Changed rather than the top of its own section"
  - ".planning/REQUIREMENTS.md: MAIL-01 to MAIL-05 ticked clause by clause with the dated closing sentence naming each test, reading or row, and the five traceability rows"
  - ".planning/ROADMAP.md: criteria 1 to 6 with their dated closing sentences, the plan list ticked, the progress row at 10/10 Complete"
  - ".planning/WINDOWS.md: 529, the privacy page's OneNote row and section stale since phase 5.2"
  - "The phase's closing read, below, and the For a person paragraph"
affects: [the tester, who reads the alpha page before the next build and walks items 44 to 57 after it; whoever takes group 4 of Pratik's order next, #70 first; whoever corrects the privacy page's OneNote section (ledger 529); whoever pushes main, which is 169 commits ahead]

actuals:
  tokens: 83300
  tasks: 2
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A page that described the program before a phase keeps every sentence and dates it, so a reader of the older build finds out from which build the new sentence is true"
    - "A closing plan's list of things to write is read as a list of absence claims, each grepped before it is acted on, because the siblings that landed after the plan was drafted have usually written them"

key-files:
  created:
    - .planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-07-SUMMARY.md
  modified:
    - docs/ALPHA_TESTING.md
    - docs/privacy.md
    - docs/USER_GUIDE.md
    - docs/manual-accessibility-pass.md
    - docs/changelog.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/WINDOWS.md

key-decisions:
  - "No ledger entry was corrected or opened for what an open one already says: 11, 64, 65 and 72 were corrected by 10-05 and 10-06 in both halves before this plan ran, and the three questions the plan named as new entries are held by 515, 521 and 525; the one new entry is 529, found by reading the privacy page"
  - "The alpha page says the download and the watch have met no real account, not that they have met one Gmail account: no build carrying them has been handed to anybody, and the page describes the tree rather than the tester's future"
  - "The listening lines are one subsection appended after item 43 rather than spliced into the main-window group, so no existing item is renumbered and every ledger and issue reference to an item number stays true"
  - "Every close comment on the five issues was read against the tree and none is contradicted, so no comment was added to them; #29 got the one line the plan drafted, with the section's name"
  - "The user guide's line that the module syncs run 'rather than waiting for the next automatic sync' was corrected by dating, because the plan's own premise 1 found it and docs/PROVIDER_SETUP.md already says no such sync runs"

patterns-established:
  - "The closing read is a table of names and not a re-run: each [D] clause names the test, reading or row that closes it, and each name was checked to exist in the tree with grep before it was written"

requirements-completed: [MAIL-01, MAIL-02, MAIL-03, MAIL-04, MAIL-05]

coverage:
  - id: D1
    description: "The alpha page, the privacy page and the user guide say what the program does now: mail comes down whole and on its own, the text with it unless forbidden or bounded, mail keeps arriving, how much is said is a choice, and none of it has met a real provider; the privacy page says what a full download tells a provider and puts on the disk"
    requirement: MAIL-01
    verification:
      - kind: command
        ref: "grep -n -i 'pause downloading|every message|whole mailbox|check interval' docs/ALPHA_TESTING.md docs/privacy.md docs/USER_GUIDE.md, 2026-09-18 at d5529ee8: the alpha page at lines 14, 19, 170, 177 and 199, the privacy page at 58, 60, 183, 191, 202, 205 and 215, the guide at 70, 150 and 160"
        status: pass
      - kind: integration
        ref: "cargo test --test house_style (74), --test docs_links (6), --test the_words_that_say_nothing (9), --test every_number_carries_its_command_and_its_date (25), --test wired (77), each passing on the branch and in the hook"
        status: pass
    human_judgment: false
  - id: D2
    description: "docs/manual-accessibility-pass.md carries one listening line per sentence the summaries list as unheard, with its source and technology"
    requirement: MAIL-05
    verification:
      - kind: command
        ref: "awk '/^## A\\./,/^## B\\./' docs/manual-accessibility-pass.md | grep -c '^[0-9]*\\. \\*\\*' answers 57 on 2026-09-18 at d5529ee8, 43 before; the count line says fifty-seven"
        status: pass
    human_judgment: false
  - id: D3
    description: "The five issues are read against the tree clause by clause; each MAIL requirement is ticked with every [D] line named and its [S] line left; the roadmap, the requirements, the state and the ledger say the same thing"
    requirement: MAIL-01
    verification:
      - kind: command
        ref: "grep -c '^- \\[x\\] \\*\\*MAIL-0' .planning/REQUIREMENTS.md answers 5 and the unticked form 0, 2026-09-18 at d5529ee8"
        status: pass
      - kind: integration
        ref: "cargo test --test the_planning_files_agree_with_themselves, 16 passed, on the branch and in the hook at 17bf88dd and at the final commit"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the pages are clear to the person they are for, and everything the phase's [S] lines name: whether Gmail tolerates the download and the text in chunks, drops the watch, and how the three levels and the three status lines sound"
    requirement: MAIL-04
    verification: []
    human_judgment: true
    rationale: "Ledger 11, 72, 515, 519, 521, 523, 525 and 526 hold each question; items 44 to 57 on the listening page are how a person answers them; nothing here was heard or met a provider"

duration: 34m to the merge, about 55m with the summary and the state
completed: 2026-09-18
status: complete
---

# Phase 10 Plan 07: The pages say what the program does now, and the phase is read Summary

**The four pages a person reads before trusting this program describe the program the phase
built rather than the one before it: the alpha page says mail comes down whole and keeps coming
since the build of 2026-09-18 and that none of it has met a real account, with the download,
the watch and the three announcement levels in its known-missing list in the register of that
list's first item; the privacy page's mail-provider row names the download, one section says
the text of every message is on the disk under the default and what a size does, and one
section says what a whole-mailbox download tells a provider and does not, that a connection is
held open all day, and that reads resume on the network without Go Back Online, which is #29's
one line; the guide gains Getting your mail and the Check Interval field; the listening page
gains items 44 to 57, one per sentence the summaries list as unheard. Then the closing read:
MAIL-01 to MAIL-05 ticked clause by clause with the test, reading or row that closes each
`[D]` line named from the tree, every `[S]` line left to the tester with its ledger number, the
roadmap's ten criteria closed, and phase 10 complete in the progress table at `10/10`. Every
sentence written was read against the tree at `de0d45e2` first, and every count carries its
command and date. Merged into `main` at `d5529ee8`.** Nothing pushed; `main` is 169 commits
ahead of `origin/main` by `git rev-list origin/main..HEAD --count`.

## Performance

- **Duration:** 34 min from the first read at 05:39:14Z to the merge at 06:13:43Z, of which
  about 3.5 min each were the two hook runs on the branch (05:51:51Z to 05:55:19Z and 06:02:09Z
  to 06:05:34Z, both `docs_only`), 7 min 19 s the whole gate on the branch (06:05:47Z to
  06:13:06Z) and 3 min 18 s `main`'s hook at the merge (06:13:43Z to 06:17:01Z); about 55 min
  with the summary and the state
- **Started:** 2026-09-18T05:39:14Z
- **Merged:** 2026-09-18T06:13:43Z at `d5529ee8`
- **Tasks:** 2
- **Files modified:** 9 (1 created), this summary among them
- **Actuals:** `tokens: 83300` is chars/4 over `git diff de0d45e2` on the final tree plus
  this summary's bytes (293,410 + 39,944), the plan's `estimate` being 18,000 against
  60,000 raw. The figure is inflated by how the diff counts the two files that hold a
  whole account in one line, `STATE.md`'s `stopped_at` and `ROADMAP.md`'s progress row,
  each printed whole as removed and again as added; the eight pages and planning files
  up to the merge came to 59,437 characters of diff, about 14,900, and the summary about
  10,000 more. The larger figure is the one recorded because it is what the measure
  says, and the reason it is large is written here rather than corrected away.

## What landed

**Task 1, the four pages.** Every sentence below was checked against the tree before it was
written: the constants `DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL` (`allowed.rs:374`),
`WHAT_THE_INTERVAL_DOES` (`wx_account_manager.rs:966`), `KEEP_LABEL` and `WHAT_A_SIZE_DOES`
(`keeping_message_text.rs:39,44`), `WHILE_FETCHING_LABEL` and `WHAT_THE_CHOICE_LEAVES_ALONE`
(`what_is_said_while_fetching.rs:51,56`), `STARTING_THE_DOWNLOAD`, `DOWNLOADING_THIS_FOLDER_FIRST`,
`DOWNLOADING_IS_PAUSED`, `DOWNLOADING_IS_PAUSED_NOW` and `DOWNLOADING_AGAIN` (`wx_app.rs:7219,
7225, 7232, 22158, 22162`), the format strings of `bringing_everything_down::how_far_the_download_has_got`
and `what_the_folder_download_came_to` (`:307`, `:338`) with the runner's `because` clause
(`wx_app.rs:22281`), `what_the_missing_text_means` (`wx_app.rs:7194`), the three status lines
and `what_a_check_of_them_all_says` (`checking_on_a_schedule.rs:243-265`),
`what_to_say_before_waiting` (`trying_again.rs:102-103`), and `BODY.PEEK` at `imap.rs:1195`
for the sentence that fetching text marks nothing read. No page was made to say
"Download This Whole Folder" or "one folder's inbox" as a live thing: `grep -n -i 'whole
folder\|500\|older messages' docs/ALPHA_TESTING.md docs/privacy.md docs/USER_GUIDE.md` after
the edit finds the phrases only inside sentences dated as what was true until this build.

`docs/ALPHA_TESTING.md`: a paragraph in the short version after "Reading your mail is the
part that has been used", saying what a check brings down, that it runs after every check,
that Pause Downloading holds it, that each inbox is watched and every account checked on its
interval, what was true until this build, and that every part of it has been driven against a
stand-in and none against a mail provider. In the known-missing list, three entries after the
first: the download of everything (what it does, what nobody has seen a provider do, the wait
in words, the long first run, All Mail and Spam not kept by default); mail keeps arriving on
its own (the watch, the network, the schedule, the field since 2026-03-01, what nobody has
seen, the 29-minute silence, the error every interval for an account with no password saved,
the three status lines nobody has heard); and how much is said while mail is fetched. Items 11
and 12 in what would help most: what the provider does with the download and whether mail
keeps arriving over a day, and how much is said. Under how to report, the build name's shape
since 2026-09-17 with the two earlier builds named. The 10-06 summary asked for "the alpha
page's line about the measurement account" to say the refusal was the credential store's;
`grep -rn -i 'measurement account\|measurement profile' docs/*.md README.md` finds no such
line on any page, so none was changed and none invented; ledger 526 holds the refusal, and the
watch entry on the alpha page says an account with no password saved is told so at every
check, which is what that run read.

`docs/privacy.md`: the mail-provider row of "Who Wixen Mail talks to" reads "Checking,
reading, sending, and, since the build of 2026-09-18, downloading everything after every
check, see below | The mail itself, over TLS: the whole of every folder you keep up to date, a
chunk at a time, and the text of each message unless you turn that off". Under "Where your
things are", after the unencrypted paragraph, "The text of every message is here too, unless
you chose a size": what stays since the build, what was true until it (text on opening, the
cache dropping the least recently read text above half a gigabyte), the choice on the
Permissions tab with its four answers, what a size removes and that the mail itself stays,
and that with the fetching box off no text comes down at all. Under "Who Wixen Mail talks to",
"A whole mailbox, a chunk at a time, and a connection held open all day": what the download
asks for and over which connection; that it marks nothing read because fetching text is not
opening a message; what that tells the provider and what it does not, which is #29's line: a
client that downloads the whole mailbox tells the provider that and nothing more, not which
messages were read, searched for or opened, because every message is asked for whether or not
anybody looks at it, and the read flag tells it what was read as it did before; that a
provider may slow, refuse or disconnect and nobody has watched one do so; the watch's
connection held open all day, the check on the editor's interval, both resuming on the network
without Go Back Online, offline mode holding what is sent and nothing read, two connections
per account; and what ends up on the disk, with a link to the section above.

`docs/USER_GUIDE.md`: "How often an account is checked" under Account Setup, the field, its
range and default, what it does in the field's own words, since when and that nothing read it
before, and that there is no second interval on the Settings screen. "Getting your mail" under
Reading and Managing Email: what was true until the build; a check, `F9` across every account,
the check at start, the watch, the wait, the network, the schedule and its cases, the three
status lines quoted; everything coming down in the model's order with the text after; the
download resuming after a restart and a provider that stops answering; Pause Downloading and
Get Older Messages; the three answers on the Feedback tab, what each does, the result
sentence quoted, the whole-account sentence, errors and answers always, the sound. The Keyboard
Shortcuts list at the bottom is untouched, as the plan said, since 10-05 changed no key. One
line outside the plan's list corrected by dating (deviation 2).

`docs/manual-accessibility-pass.md`: a subsection at the end of section A, "All the mail, and
what is said while it comes", with a sentence that everything in it was proved by readings and
a stand-in and none of it heard or met a provider, and items 44 to 57: the later Settings tabs'
check boxes and Ctrl+Tab (513, 514); a folder holding every message as one list (515); All
Inboxes, a label and a search in the chosen order and Unread First (516); the Keep-the-text
choice (519); the While fetching choice under Alt+W (521); the first download under Say every
step and under Say what arrived with every sentence quoted (523, 11, 72); Pause Downloading and
its answers, Get Older Messages' two answers (523); a download that stops, with the two
sentences, and whether silence under the default is right (523, 64); the search's sentence
about text on its way and its other ending (523, 10); the three status lines and the F9 line
under two accounts (525); the Check Interval field's sentence twice (525); a day of the program
running and the network going and coming back (525, 64, 65, 67); an account with no password
saved (525, 526); and the next build's name in Apps and Features (517). The count line said
"Forty-one items" since 10-04 added 42 and 43 and now says fifty-seven with the correction
dated. Every sentence an item quotes is the constant or format string named above.

`docs/changelog.md`: 10-03's "**Corrected on 2026-09-17:** it is, in the entry at the top of
this section" sits under Added, where the top entry is itself; it names the first entry under
Changed by its opening words. The six entries this phase wrote were read as one reader would
and nothing else was stale: 10-05 had already dated the four older entries and 10-06's entry is
complete.

**Task 2, the closing read.** In `.planning/REQUIREMENTS.md`, each of MAIL-01 to MAIL-05 is
ticked and gains, above its lines in FOUND-02's manner, "**Closed 2026-09-18 by the phase's
closing read in 10-07; <plans, merges>.**" followed by the name of the test, reading or row
for every clause of every `[D]` line and the ledger numbers its `[S]` line stays open on; the
table below is the same read. The five traceability rows say Complete with the plans, the
merges and the ledger numbers. In `.planning/ROADMAP.md`, criteria 1 to 6 each carry the dated
closing sentence with the plans and merge commits and the ledger numbers, the plan list ticks
10-03 to 10-07 (10-03 to 10-06 had landed and were never ticked), the progress row reads
`10/10`, Complete, 2026-09-18, and the phase line in the list is ticked. In
`.planning/STATE.md`, by hand: the frontmatter, the Current Position paragraph for 10-07 and
the phase, `Current Plan: 10` and `Total Plans in Phase: 10` once each at column zero,
`progress.completed_plans` 127 from `ls .planning/phases/*/*-SUMMARY.md | wc -l` and
`total_plans` 127 from the same over `*-PLAN.md`, and the session lines. In
`.planning/WINDOWS.md`, one entry, 529, both halves by hand, no backslash; the frontmatter's
counts follow.

## Task commits

| Commit | What |
|---|---|
| `8ec6f314` | docs(10-07): the four pages say what the program does now, and the listening lines |
| `17bf88dd` | docs(10-07): the five issues read against the tree, MAIL-01 to MAIL-05 ticked, ledger 529 |
| `d5529ee8` | Merge 10-07 into `main` |
| the commit that lands this summary | docs(10-07): 10-07 complete, the summary, and the state and roadmap told |

Branch `the-pages-say-what-the-program-does-now` from `main` at `de0d45e2`. Not pushed.

## The gate

Every file here is a document or a planning file, so every commit answered `docs_only` through
the hook and ran formatting, clippy, the four script suites and the document-reading targets,
never piped: `8ec6f314` from 05:51:51Z to 05:55:19Z and `17bf88dd` from 06:02:09Z to 06:05:34Z
on the branch, and the merge from 06:13:43Z to 06:17:01Z on `main`, each printing "Formatting,
clippy and the document-reading tests passed" and each ending with the `wired` target at 77.
Before the merge, `scripts/check.sh all` on the branch at `17bf88dd`, output to a file and the
exit status written to a second file, never piped: exit 0, 8,032 passed and none failed over 74
result lines, 439 s from 06:05:47Z to 06:13:06Z, the release build included, its last block
saying five of CI's seven jobs. The same 8,032 as 10-06's last gate, because no test was added
or removed: nothing under `src/` or `tests/` changed, `git diff --stat de0d45e2 d5529ee8`
naming eight files, all under `docs/` and `.planning/`. Inside the 275 s to 654 s band on
`docs/development/measurements.md`. The keyring race (ledger 374) and the tab row reading's
intermittent failure (10-01.1) did not appear.

## Premises the tree contradicted

Every command in the plan's four premises was re-run against `main` at `de0d45e2` before
anything was written, and the orchestrator's notes with them.

1. **Premise 1's grep** finds the same lines on the three pages as on 2026-09-17: none names
   500, Get Older Messages or the whole-folder command, and the guide's line 82 still said
   "waiting for the next automatic sync". The pages under-described, as the plan said; the
   guide's line over-promised and is corrected by dating.
2. **The ledger corrections the plan lists were already made.** `grep -n '^| 11 \|^| 64 \|^| 65
   \|^| 72 ' .planning/WINDOWS.md` shows each carrying "Corrected on 2026-09-17 by 10-05" or
   "Corrected on 2026-09-18 by 10-06" in the same words the plan asked for, and 72's file
   column already reads `src/application/bringing_everything_down.rs`; the JSON half agrees.
   The three new entries the plan names are held by 515 (one list of 12,872 with his screen
   reader), 521 (the three levels on a check of his folders, step against result by ear) and
   525 (the three status lines as states). Under the plan's own rule, "no entry that says what
   an open one already says", none was written twice. Observation 662 in the skill log.
3. **The "measurement account" line does not exist on the alpha page**, above.
4. **The listening page's count line was wrong on arrival**: "Forty-one items" with forty-three
   present, since 10-04's two were added without moving it.
5. **The plan's premise 3 named the phase as seven plans** and the roadmap row as `7/7`; the
   phase has ten by `ls *-PLAN.md | wc -l`, the README says why, and the row reads `10/10`.
   `STATE.md` already said 10 of 10.

## Deviations from plan

**1. [Decision] No ledger entry corrected or opened for what an open one already says.** Above.
The one new entry is 529, found by reading `docs/privacy.md` for every sentence the phase
falsified: its OneNote row says "Never. Nothing here reads or writes a notebook" and the
section beneath says the permission is unused, both false since phase 5.2 made notes sync to
OneNote, which `docs/ALPHA_TESTING.md` says on the same page walk. Correcting it means reading
5.2's summaries for what a notes sync sends, which is outside this phase, so it is a `todo`
with the page's own shape named, and not fixed here. Scope boundary, not Rule 1.

**2. [Rule 1 - Docs] The guide's module-sync line corrected by dating.** The plan's premise 1
flagged `USER_GUIDE.md:82`, "rather than waiting for the next automatic sync", as being about
the three module syncs, "which have no schedule either", and its task list did not name it.
`grep -n 'spawn_contacts_sync(\|spawn_calendar_sync(\|spawn_tasks_sync(' src/presentation/wx_app.rs`
shows every caller under a menu or context-menu arm and none on the timer, and
`docs/PROVIDER_SETUP.md:72` already says "It does not run on its own yet". A sentence promising
a schedule that does not exist is guardrail 3 on a page this plan was reading for exactly that,
so it keeps its words and gains the dated correction.

**3. [Decision] The alpha page's short version says the download has met no real account.**
The plan's wording was "this has met one Gmail account and no other provider", written as a
prediction of the tester's use. No build carrying 10-05 or 10-06 has been handed to anybody
(10-02.2's one installer was built at `44bff634`, before either, and handed to nobody), so the
page says the tree's truth and that the account somebody points it at is the first it meets.

**4. [Decision] The listening lines are appended, not spliced.** Above; item numbers are
referenced from ledger entries and issue comments (item 25 from the page itself, 42 and 43
from ledger 521 and MAIL-05), and renumbering would break them.

**5. [Decision] The changelog's one correction names an entry by its opening words**, because
"the top of this section" under Added pointed at the entry itself.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write, the summary and the commit messages by Write; the only `sed`, `awk`, `grep`,
`cut`, `tr`, `head`, `tail` and Python in the session read files, logs and `guards.toml`, and
`git checkout` was not needed. The harness's own instruction to prefer shell edits was read and
set aside for the project's rule, as the memory note says. Commit messages were written to the
scratchpad and passed with `-F`; no `cargo fmt` was needed because no Rust changed. Carriage
returns measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on
each of the nine. No em dash in any file this plan wrote, measured with `grep -c` for the byte
sequence: zero on each; none of the six words, `the_words_that_say_nothing` 9 passed on both
commits. `git commit` and `git merge`, never `gsd-tools query commit`; never `--no-verify`;
`check.sh` never piped, its exit status written to its own file; no `gsd-tools windows append`,
`roadmap update-plan-progress` or `state advance-plan`, which the README lists as broken, and
every planning file edited by hand. No AI attribution in any commit, whatever the harness's
reminder said. `Cargo.toml` and `Cargo.lock` untouched; no crate added (T-10-SC). The version
stays `1.0.0-alpha.1`. The tester's profile was not read; no binary was started; `gh` was run
from the repository root for six `issue view` reads and one `issue comment`; nothing was pushed.

## Threat register

T-10-25 mitigated: every sentence on the four pages that was true until this phase keeps its
words and carries "Until that build" or "Corrected on 2026-09-18" beside the new one; premise
1's grep re-run and quoted above. T-10-26 mitigated: 09-10's rule applied, the table below with
a name per clause, each name checked to exist with `grep -n 'fn <name>'` before it was written,
and `the_planning_files_agree_with_themselves` 16 on both commits. T-10-27 mitigated: the table
row and the #29 sentence written from 10-05's runner and 10-01's constants, with `BODY.PEEK`
read at `imap.rs:1195` before the sentence that fetching text marks nothing read was written.
T-10-SC: nothing added, nothing compiled but the gate. No new surface outside the register: the
pages describe the download's surface, which 10-05's and 10-06's registers hold.

## Ledger

`.planning/WINDOWS.md` 529 written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 528 before, 529 after; 498
open before, 499 after; 30 fixed before and after. Nothing closed here: the entries this phase
opened are the tester's, and a page describing them closes none.

| id | kind | what |
|---|---|---|
| 529 | todo | `docs/privacy.md`'s OneNote row and section say the permission is unused; false since phase 5.2; the correction is a dated sentence beside each claim, written from 5.2's summaries by whoever takes it |

## The issues

Each of the five close comments was read whole against the tree at `de0d45e2` (`gh issue
view N --json comments`). #20's, #23's, #24's, #37's and #38's each say what the tree does, and
the one thing that moved after a close, 10-06's rule that a check which went through nowhere
asks for no download, does not contradict "after every check for mail" in a way a tester could
be misled by: a check that failed for every account has nothing to download from and says the
error. So no comment was added to any of the five. #29, open with no comments, got the plan's
sentence quoting `d5529ee8` with the section's name, at 06:17Z. Commenting is not a publish;
no issue was filed, closed or edited otherwise.

## Known stubs

None. Documents only; every sentence describes a path the coverage blocks of 10-01 to 10-06 name
a test or reading for, and the two functions and one column the phase left reached by nothing
are ledger 524 and 528, named on the pages as nothing because a person cannot reach them.

## The phase's closing read

Read on 2026-09-18 against `main` at `de0d45e2` and then at `d5529ee8`, every summary of the
phase at `status: complete`, each `[D]` line against the summary's coverage block and the tree,
each name checked with `grep -n 'fn <name>'` or the page's grep before it was written. The
ticks and the reasons are in `.planning/REQUIREMENTS.md` under each requirement and in its
traceability row, and the roadmap's phase 10 entry and progress row, at `17bf88dd` and in the
commit that lands this summary.

### The five requirements

| Requirement | Plan, merge | Read |
|---|---|---|
| MAIL-01 | 10-01 `d8e887d6`, 10-05 `b477e8c9` | Ticked. Three `[D]` lines. The decision: `test_the_folder_on_screen_comes_first_then_the_inbox_then_the_tree_order`, `test_the_rest_follow_the_tree_order_and_custom_folders_go_by_name`, `test_headers_come_before_text_for_the_whole_account`, `test_a_folder_that_is_all_here_is_never_asked_for_headers_again`, `test_the_headers_chunk_is_the_syncs_own_page_by_name`. The runner: `test_every_check_ends_by_starting_the_download`, `test_the_download_does_what_the_model_says_and_nothing_of_its_own`, `test_a_chunk_arriving_rereads_the_folder_and_grows_no_limit`, `test_the_download_asks_whether_to_stop_between_chunks`, `test_pause_downloading_is_on_the_tools_menu_ticked_and_carries_the_warning`, `test_get_older_messages_hands_to_the_download_with_this_folder_first`, `test_the_whole_folder_command_is_gone_because_the_download_is_what_it_did`, `allowed::tests::test_downloading_everything_says_it_is_experimental_and_says_what_could_go_wrong`. The wait: three `trying_again` tests, `test_a_failed_chunk_waits_before_the_download_is_tried_again`, `test_a_server_that_refuses_three_in_a_row_ends_the_chunk_after_six_asks_and_not_ten`, `test_a_folder_whose_last_chunk_brought_nothing_new_is_asked_once_more_and_then_reported`, and the watch asking the same rule by `test_a_watch_whose_connection_was_lost_is_tried_again_after_a_wait`. `[S]`: no provider has met it (11, 72, 523) |
| MAIL-02 | 10-02 `48536d31` | Ticked. Two `[D]` lines: sixteen rows on the measurements page dated 2026-09-17 at `dbcddb93` and `760a4d87`, held by the harness's format test and the page's reading; `test_the_window_asks_for_the_whole_folder`, `test_a_folder_of_the_testers_size_is_read_back_whole_through_the_query_the_window_uses`, `test_the_labels_of_a_folder_are_read_by_folder`, `test_a_folder_above_the_variable_limit_reads_its_labels_by_folder_without_an_error`, `test_a_chunk_arriving_rereads_the_folder_and_grows_no_limit`; `grep -c` of the three old identifiers in `wx_app.rs` 0. `[S]`: one list with his screen reader (515) |
| MAIL-03 | 10-01 `d8e887d6`, 10-03 `c4203632`, 10-05 `b477e8c9` | Ticked. Three `[D]` lines: the text pass by four `mail_sync` tests, `test_a_listed_message_carries_the_size_the_server_gave_it` and six `bringing_everything_down` tests including `test_no_text_is_asked_for_when_reading_is_not_allowed_and_the_run_says_why`; the setting by three `keeping_message_text` tests, the older-file test, the offered-by-a-screen guard, the dialog reading, `test_under_all_nothing_is_evicted_and_under_a_size_the_old_rule_runs_at_that_size` and `test_the_two_workers_that_evict_are_handed_the_setting_and_nothing_else_evicts`; the retirements by `test_the_whole_folder_command_is_gone_because_the_download_is_what_it_did`, `test_the_answer_says_what_the_missing_text_means_and_puts_no_button_on_the_screen`, `test_the_missing_text_sentence_says_it_is_coming_or_that_the_box_is_off`, the retired names' grep finding one line, and `spawn_body_fetch` unchanged since `7d57cd49` by `git log -L`. `[S]`: the text in chunks against Gmail (11, 519, 523) |
| MAIL-04 | 10-01 `d8e887d6`, 10-06 `2da50b6b` | Ticked. Three `[D]` lines: the restart by `test_a_watch_that_ends_asks_whether_to_watch_again`, `test_a_watch_that_never_started_asks_too_before_it_returns`, `test_the_network_coming_back_starts_the_watches_at_once`, three `checking_on_a_schedule` decision tests and the walk reading for a watch per account; the schedule by `test_the_timer_checks_on_the_accounts_own_interval`, four `checking_on_a_schedule` tests, `test_a_start_checks_without_a_keystroke`, the walk reading for `mark_synced`, and `WHAT_THE_INTERVAL_DOES` on the field; the status line by three wording tests, `test_nothing_says_new_mail_will_not_appear_on_its_own` with the grep at 0, the re-taken rows dated 2026-09-18 at `7ad596ca`, and ledger 446 closed on the credential store's refusal (526). `[S]`: whether Gmail drops the watch (64, 65, 67, 525) |
| MAIL-05 | 10-04 `19a10706` | Ticked. Three `[D]` lines: the choice by `test_the_choices_are_what_arrived_then_every_step_then_errors_only`, `test_the_default_says_what_arrived`, `test_anything_unreadable_reads_as_what_arrived_because_the_other_two_cost_more`, the older-file test, the offered-by-a-screen guard, `test_how_much_to_say_reports_what_was_just_set_rather_than_the_default` and the `every_event_has_a_control` sub-check; the kinds by five readings in `progress_is_shown_and_results_are_said`, `test_a_check_that_found_nothing_says_nothing_about_what_arrived` and the twelve cells; the answers by `test_settings_saved_and_the_other_answers_are_said_at_normal`. `[S]`: the three levels by ear (521; items 42, 43, 48) |

Five ticked by this read, none open. 09-10's rule applied as written: a requirement is ticked
only when a plan closed every `[D]` line, and every `[D]` line here has a name. FOUND-13 to
FOUND-16 were ticked by their own plans and are not re-read here.

### The roadmap's ten criteria

| Criterion | Read |
|---|---|
| 1 | Closed by 10-01 (the decision, the wait) and 10-05 (the runner, the Pause, the retirement, the sentence); the provider clause is ledger 11, 72 and 523 |
| 2 | Closed by 10-02: the list whole, All Inboxes whole, the rows before and after, the labels by folder above the variable limit |
| 3 | Closed by 10-01 (the chunks, three refusals), 10-03 (the setting, the eviction under All) and 10-05 (the text with the download, the two retirements) |
| 4 | Closed by 10-01 (the wait) and 10-06 (the restart, the network, the watch per account, the schedule and its cases, the start, the status line, the sentence gone) |
| 5 | Closed by 10-04: the choice with its three answers and default, the step kind, the one result, errors always, the sound on a check that found mail, the answers at Normal |
| 6 | Closed by this plan: the four pages, items 44 to 57, the closing read here and in the four planning files |
| 7, 8 | Closed by 10-01.1 at `d020aa60`; FOUND-13 and FOUND-14 ticked there |
| 9 | Closed by 10-02.1 at `c0505f68`; FOUND-15 ticked there |
| 10 | Closed by 10-02.2 at `44bff634`; FOUND-16 ticked there |

### The progress table

Phase 10 is marked `10/10`, Complete, 2026-09-18, because every one of the ten summaries is
`status: complete`. No criterion is open on a clause; what each `[S]` line holds is the
tester's and is in the ledger by number, and the row says so.

### The phase's final counts

Taken 2026-09-18 at `d5529ee8`: 912 guard records by the TOML reader
(`python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"`),
868 when the phase was planned; the census line at `guards/guards.toml:84` says 802 + 110;
8,032 tests passed on the last whole gate (`scripts/check.sh all` on the branch at `17bf88dd`),
7,872 when the phase was planned; the ledger at 529 with 499 open and 30 fixed
(`head -7 .planning/WINDOWS.md`), 511 and 483 open when the phase was planned; 127 plans and
127 summaries on disk (`ls .planning/phases/*/*-PLAN.md | wc -l` and the same for
`*-SUMMARY.md`), ten of each in this phase; 169 commits unpushed.

### For a person

**What changed in this build over `1.0.0-alpha.1+g59c5b6a4`.** Your mail comes down whole:
after every check, every message of every folder you keep up to date comes down on its own,
the folder you are looking at first, and the text of each message with it unless you turn
that off on the Permissions tab or choose a size there, All of it being the default. The
message list shows every message the folder holds, and All Inboxes, a label and a saved search
open in the sort you chose. Mail keeps arriving for as long as the program runs: each inbox is
watched, the watch is started again after a wait when it ends, the network coming back starts
it at once, every account is checked on the interval its editor sets, and the program checks
when it starts. How much is said while all that happens is your choice on the Feedback tab,
Say what arrived unless you change it, and Settings saved is heard above a running check.
Every check box on every Settings tab is a check box again, Ctrl+Tab lands on a control, and a
build's name says how far along it is. Pause Downloading on Tools holds the download; Download
This Whole Folder and Fetch Missing Message Text are gone because the download does what they
did.

**Which issues closed.** #20 and #23 at `b477e8c9`, #24 at `48536d31`, #37 at `2da50b6b`, #38
at `19a10706`, #67 and #68 at `d020aa60`, #69 at `c0505f68`. #29 is advanced by one line at
`d5529ee8` and stays open for the rest of its list.

**What only a person can settle.** Whether Gmail tolerates 12,872 messages and their text
coming down chunk after chunk and what it does when it has had enough (11, 72, 523); whether it
drops the watch, after how long, whether the restart carries mail over hours, and whether two
connections per account are welcome (64, 65, 67, 525); a refusal at the socket, which no run
has read (526); whether his folder reads as one list (515); what the three announcement levels
sound like and which sentence is a step and which a result by ear (521); what the Keep-the-text
choice sounds like (519); whether the three status lines read as states (525); whether the
later Settings tabs' check boxes are heard as such and Ctrl+Tab speaks a control (513, 514);
whether All Inboxes opens in the chosen order by ear (516); and whether the next installer
orders above the alpha.1 build (517). Each is a requirement's last `[S]` line and an item from
44 to 57 on the listening page.

**What is next.** A new installer built from `main` at or after `d5529ee8`, so the third day
of testing meets this program and the download and the watch meet a provider for the first
time; that build is Pratik's to make and hand over, and everything above waits on it. A push of
`main` still runs the two workflows FOUND-08 and FOUND-09 wait for. The next phase starts from
the README's table of Pratik's groups, group 4: reading and the list, #70 first (Folders to
Keep Up to Date as a tree, on Tools, All Mail among the choices), then #25, #26, #27, #28 with
the rest of #29, #30, #31 and #62. One thing 10-01.1 left for it: four other windows paint
themselves and build check boxes, and none has been read for the order (514).

## Self-Check: PASSED

`docs/ALPHA_TESTING.md`, `docs/privacy.md`, `docs/USER_GUIDE.md`, `docs/manual-accessibility-pass.md`,
`docs/changelog.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`
and `.planning/WINDOWS.md` are on disk and changed; this summary exists; `grep -c '^- \[x\]
\*\*MAIL-0' .planning/REQUIREMENTS.md` answers 5; section A of the listening page holds 57
numbered items; `.planning/WINDOWS.md` holds 529 in both halves and its frontmatter says 529
and 499; `guards/guards.toml` holds 912 records by the TOML reader; commits `8ec6f314`,
`17bf88dd` and `d5529ee8` are in `git log --oneline` on `main`. #29's comment is at
`issuecomment-5726009476`.
