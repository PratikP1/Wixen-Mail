# Phase 14: The real-account proofs

Thirteen plans, one per wave, written on 2026-10-05 against `main` at `01ef4589`, version
`1.0.0-alpha.1`, from `PROPOSAL.md` and its three research files in this folder, on Pratik's
answers of the same day to the proposal's twelve questions. On the day, `guards/guards.toml`
held 1,437 records by the TOML reader, as the planners read it with the command `CLAUDE.md`
gives; `.planning/WINDOWS.md`'s frontmatter said 806 entries, 692 open and 114 fixed; the last
whole gate was phase 13.1's, 10,566 tests at `0bc790d0`; and
`git rev-list origin/main..main --count` answered 0, with `origin/main` at `01ef4589`. The
build Pratik has installed is `1.0.0-alpha.1+351.g5f363255`, 720 commits behind `01ef4589`
(research 3, `git rev-list --count 5f363255..HEAD`), and `git rev-list --count 01ff57bf..HEAD`
answered 1,071, the build counter the phase's build starts above.

The directory is named the way `gsd-tools generate-slug "The real-account proofs"` answers,
`the-real-account-proofs`.

**Goal.** A Gmail account's calendars, contacts and tasks arrive for Pratik, or the program
and its log say why they cannot (#22, REAL-01). The copy within one account and across two,
the move within one account and across two, and the delete are each proved against his
accounts, after a restart and with the network off where the queue replays, and recorded on
#63 on his word (REAL-02). The warning sentences say, path by path, what was proved and when.
The phase closes on its own full gate.

**Requirements.** REAL-01 and REAL-02. No requirement is added and the coverage stays at 123.
Each plan adds a dated line under the requirement it serves and ticks no box; 14-13's closing
read ticks a box only if every line it carries is proven. Two readings follow from the
answers below, and the plans carry both:

- REAL-02's last `[D]` clause names "`application::allowed`'s default moved per proven path".
  On answer 7 the switch's starting position does not move in this phase and only the
  warning sentences move per path, so 14-13 leaves that part of the clause unearned, says
  why, and writes the switch's split as a `todo` for after the phase.
- REAL-01's `[D]` clause opens "The profile's log at the moment of a Refresh read first".
  Today's build writes no line for a request, an answer or a skipped provider (PROPOSAL.md,
  "What the research changed", point 3), so there is nothing to read until 14-01 and 14-02
  write those lines; 14-09 reads them from his profile after sitting 1.

## Pratik's answers of 2026-10-05

The orchestrator put PROPOSAL.md's twelve questions to Pratik and he answered all twelve on
2026-10-05, in the words below. Each row keeps the question's number, and the plans cite it
as "answer N". Where an answer left a gap, the planners filled it under "Decisions the
planners took" below, for him to overrule.

| No. | The question, in short | His answer, in his words | What the plans take from it | Plans |
|---|---|---|---|---|
| 1 | A Google sign-in client: has he one, or will he make one, and where does it live | "I'll make a testing key." | He makes a Google Cloud sign-in key in Testing with himself as the tester and places `oauth.toml` in `%LOCALAPPDATA%\wixen-mail\config\` himself; it stays on his machine and no agent makes, opens or handles it. 14-03 writes Help's step-by-step page for it in words a screen reader user can follow: the console's screens, the Calendar, People and Tasks interfaces and their three permissions, the test user, and where `oauth.toml` goes with its two values, checked against Google's pages read on the day it is written. | 14-03, 14-08, 14-09 |
| 2 | How his Gmail account signs in for calendars, contacts and tasks | "We'll separate the two signins for now. Mail with app password and cal/task/etc with browser signin." | Route B. Mail stays on the app password; a separate browser sign-in asks Google for the calendar, contacts and tasks permissions only, kept under a credential store entry of its own. | 14-01, 14-03, 14-07, 14-09, 14-12 |
| 3 | All his Google calendars, or only the main one | "All Google calendars." | 14-04 reads Google's calendar list, hidden calendars included, and files each calendar under its own row. | 14-04, 14-09, 14-12 |
| 4 | The sending proof of 18 September: app password or browser sign-in | "Correct the comment to accurately describe how signin occurs." | The comment on #63 that says "OAuth" is edited on his own repository to say the app password, with a dated sentence saying what it said before. From 14-05 every sign-in line and the send line say how they signed in, so the next record cannot name the wrong one. | 14-05, 14-07 |
| 5 | The two deletes waiting since 20 September | "Let the two waiting deletes go." | Nothing holds them. They go to Gmail at the new build's first check, at the start of sitting 1, after the reading before it, and are recorded on #63 as evidence for the delete line, not as its proof. | 14-05, 14-08, 14-09 |
| 6 | Other accounts | "I'll add more accounts." | A second account for the crossings (a second Google account on his project's tester list also proves adding an account); an IMAP account run by neither Google nor Microsoft for emptying the Trash at a server; a POP account with Leave mail on the server on. Sheet 4 names what each part needs, and a line without its account is recorded as still open, never dropped. | 14-08, 14-12 |
| 7 | The safety switch for changes to mail | "Do what's appropriate." | The recommendation, (a): the warning sentences change path by path as each line is proven, and the switch is neither split nor moved from its starting position in this phase. Splitting it, (b), is decided after the phase with its cost in front of him; 14-13 writes it as a `todo`. Not (c). | 14-07, 14-13 |
| 8 | Three sentences false today, corrected before the build | "Do what's appropriate." (7 and 8 answered together) | The recommendation: all three corrected before the build, each saying what happened and when. Sending was proven on 18 September 2026; Gmail met the download of everything on 18 to 20 September; a Google account's calendars, contacts and tasks need the browser sign-in. | 14-07 |
| 9 | Reading his profile, and posting the records | "yes to both." | His settings, log and mail database are read before and after each sitting with Python, read-only, printing counts, the build's name and masked lines, never a body, and no subject except those beginning "Wixen proof". His one-line confirmation of a sitting's drafted record is the word to post it on #22 and #63. 14-06 writes that rule once, as a reader with a test. | 14-06, 14-09 to 14-12 |
| 10 | Listening checks in the same sittings | "yes." | Each sheet carries the two to five listening items that walk the same screens at the same moment, and nothing else from the listening page. | 14-08 to 14-12 |
| 11 | What phase 14 covers, and in what order | "Yes." | Microsoft, CalDAV and CardDAV, invitations from a real organiser, S/MIME and PGP with a correspondent, a work directory, free/busy, shared mailboxes and other sending addresses go to a later phase. The order is confirmed: #22 first, then copy, move and delete within Gmail, then across two accounts. | every plan |
| 12 | The build: made here or from CI, and which version | "yes." | Made on this machine from the last code plan's merge, 14-07's, with no push needed; the version stays `1.0.0-alpha.1`. Windows' list of installed programs shows the full build string; `1.0.0.14999` appears only in the setup file's own Properties (14-08, premise 11). | 14-08 |

## The plans

"Who acts" says whose hands each plan needs. "Pushed" is whether the plan's branch goes out
with a pull request under Pratik's standing OK of 2026-09-23, which a plan uses when it
changes what is spoken or shown. Every plan merges with `git merge --no-ff`, and none pushes
`main`.

| Plan | Wave | What it does | Requirement | Who acts | Ledger | Pushed |
|---|---|---|---|---|---|---|
| 14-01 | 1 | Sync Calendar, Sync Contacts and Sync Tasks say why nothing came from Google instead of "0 created", with the cue for an account needing attention and one Info line; an account not at Google never asks Google; an item held for a choice is counted once | REAL-01 | The executor | Two opened and fixed (the silent skip of #22, the held count added twice); one for the tester's ear | yes |
| 14-02 | 2 | One Info line per Google and tasks request and per finished sync, nothing private in it, shown against the loopback servers; the provider follows the account; the three syncs run once when an account is added; F5 in a module syncs that module | REAL-01 | The executor | Four opened and fixed (no line per request or sync, the provider chosen by keys, nothing run on adding an account, F5 in a module); one for the ear | yes |
| 14-03 | 3 | Route B: a separate browser sign-in for a Gmail account's calendars, contacts and tasks, the Account Manager's "Sign In for Calendars, Contacts and &Tasks", kept in its own credential store entry and erased with the account; Help's page for making the testing key and placing `oauth.toml`; `docs/installing.md:90-91` corrected | REAL-01 | The executor | Two opened and fixed (#22's app-password accounts with no page saying how, `docs/installing.md`); a `todo` for an address written to the log; one for the ear | yes |
| 14-04 | 4 | Every calendar on Google's list under its own row, read-only, hidden and free-and-busy calendars handled, events looked up within their calendar, a calendar Google stops listing put away safely | REAL-01 | The executor | Two opened and fixed (other calendars never read, an event found across calendars); one for the ear | yes |
| 14-05 | 5 | The write paths say what they did: a replayed move, copy or delete names its row, the server's answer and where it is now; a crossing writes each step and its outcome; a sign-in and a send say how they signed in | REAL-02 | The executor | None | no |
| 14-06 | 6 | `scripts/read-a-proof.py`, the tested way to read a proof from his profile without printing his mail; one crash writer naming the build; the page window's test out of his `crash.log` | REAL-01, REAL-02 | The executor | One, the test that wrote into his `crash.log` | no |
| 14-07 | 7 | True words before the build: sending proven on 2026-09-18, what Gmail did to the download, the first-run sentence about the provider, the shortcuts page's crossing sentence; the records put straight and #63's comment corrected to the app password | REAL-02 | The executor; the edit on #63 rests on answer 4 | 148 fixed; 191 rewritten; 11, 64, 65, 67, 72, 523, 525 and 546 amended; two new (the Trash unreadable on 18 and 20 September, Refresh's help sentence) | yes |
| 14-08 | 8 | The build at 14-07's merge, the install page and the four sitting sheets | REAL-01, REAL-02 | The executor builds and writes; Pratik installs and places his key | None | no |
| 14-09 | 9 | Sitting 1: the first start and the two waiting deletes, the browser sign-in, the three syncs and F5 in each module, one event, contact and task read back in Google's own apps; recorded on #22 and #63 | REAL-01, REAL-02 | Pratik sits, 40 to 60 minutes, and gives his word; the executor reads before and after, drafts and posts | Dated lines on 154, 194, 203, 216, 218, 434, 435, 523, 525, 543, 597, 628, 671, 727 and 732; one per defect found | no |
| 14-10 | 10 | Sitting 2: copy and move within Gmail, online, offline and back, across a restart, one message changed in Gmail meanwhile, undo at the server and of a mark; recorded on #63 | REAL-02 | Pratik, 45 to 60 minutes; the executor as above | 544, 546, 623 and 625; Refresh's sentence if 14-07 did not open it; one per defect | no |
| 14-11 | 11 | Sitting 3: delete within Gmail online, offline and across a restart, Delete Permanently against his own expunge setting, a delete held by the account's box, undo; recorded on #63 | REAL-02 | Pratik, 25 to 35 minutes; the executor as above | 376, 377, 544, 546 and 625; one per defect | no |
| 14-12 | 12 | Sitting 4: copy and move across two accounts, ledger 187's three questions and 547's, a second Google account added, the Trash emptied at a non-Gmail IMAP server, the POP lines; recorded on #63 and #22 | REAL-01, REAL-02 | Pratik adds his accounts and sits, 60 to 80 minutes and two-minute looks later; the executor as above | 183, 187, 188, 191, 547, 770, 771, 784, 787 and 804, with due dates; one per defect | no |
| 14-13 | 13 | The warnings per proven path on his word, the switch left where it is and its split ledgered, the closing read, and the phase's full gate, `scripts/check.sh all` once by hand on its branch before its merge | REAL-01, REAL-02 | The executor drafts and runs the gate; Pratik says yes per path and close or leave open for #22 and #63, about 10 minutes | One per changed spoken sentence; the switch's split, a `todo`; the ride-along entries read | yes |

The times are the proposal's estimates, not measurements.

## One plan per wave, and why this order

One per wave because every plan writes `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`
and `.planning/STATE.md` for its completion marks, eleven write `.planning/WINDOWS.md`, eight
write `docs/changelog.md`, and every code plan writes `guards/guards.toml`; a wave is a set of
plans that share no file, and these share those. Each plan depends on the one before it.

- **14-01 to 14-07 first,** all the code and the words, so the one build every sitting runs on
  carries what the sittings are read from. 14-01 is REAL-01's tracer, the one answer to "may
  this account ask Google, and if not, why"; 14-02 builds the request lines and the provider
  following the account on it; 14-03's sign-in needs both; 14-04 reads every calendar with
  14-03's permission and 14-02's masked request line. 14-05 and 14-06 give the log and the
  reader that every sitting is read from. 14-07 is last of the code, so the build says only
  true things.
- **14-08** makes the build at 14-07's merge and writes the install page and the four sheets.
- **14-09 to 14-12** are the four sittings, in answer 11's order.
- **14-13 last,** at the highest wave: the wording per proven path, the closing read and the
  phase's full gate.

The proposal said sittings 2 and 3 could go first if his key is not ready. The plans as
written do not allow that without a change: 14-10 and 14-11 read 14-09's summary, and 14-11's
precondition needs it to say the Trash was readable on the installed build. Running them
first is a decision to take with him that re-points those two preconditions.

## Decisions the planners took

Taken on 2026-10-04 by the proposal and on 2026-10-05 by the plans, where an answer left a
gap. Pratik may overrule any of them; each plan that carries one says so in its decisions
block.

- **D-01. New log lines at Info,** his stored level, which a profile keeps; Debug stays his
  choice (proposal; 14-02 choice 1; 14-05).
- **D-02. The proof reader lives in the tree with a test** (14-06), not as a script typed into
  each plan, because it reads his profile at least eight times this phase (proposal).
- **D-03. The sitting sheets and the install page live in this folder,** not under `docs/`,
  which ships in the installer (proposal).
- **D-04. One build,** and a fix build only when a sitting finds a defect (proposal).
- **D-05. Research's sitting 0 is folded into sitting 1:** the first start is the first thing
  sitting 1 does (proposal).
- **D-06. When several reasons apply, only the first is said,** in the order no Google sign-in
  key in this copy, then the app password, then a browser sign-in that has run out, because
  each later one cannot be acted on until the earlier is settled (14-01 choice 1).
- **D-07. A sync that asked Google nothing uses the cue for an account needing attention,**
  not the success cue, since the success cue for a sync that did nothing is the defect #22
  reports (14-01 choice 2).
- **D-08. "0 created, 0 updated, 0 deleted" is left out only when nothing at all was asked;**
  when another pass ran, its counts are said and the reason follows (14-01 choice 3).
- **D-09. One log line per request sent, retries included,** three retries at most (14-02
  choice 2).
- **D-10. Adding an account says at most one sentence per module,** or the reason once for
  all three (14-02 choice 3).
- **D-11. F5 with Contacts, Calendar, Tasks or Notes showing runs that module's sync,** as
  Sync Now does; Mail and Reminders keep what F5 does there. Today F5 in those modules
  reaches the mail folder's refresh handler (14-02 choice 4).
- **D-12. The separate sign-in is a button on the Account Manager,** "Sign In for Calendars,
  Contacts and &Tasks", Alt+T, acting on the account chosen in the list like Sign In Again,
  not a box in the account editor, whose pages have no letters to spare. The Account
  Manager's letters read on 2026-10-05 were A, E, L, O, D, V, U, S and C; 14-03 reads them
  again on the day (14-03 choice 1).
- **D-13. Adding a Gmail account does not open the browser by itself.** It says once which
  button to press, and the three syncs run once after he signs in, when the Account Manager
  closes (14-03 choice 2).
- **D-14. A Gmail account that signs in to mail through the browser keeps using that sign-in**
  for its calendars, contacts and tasks; the separate one is for app-password accounts, and
  the button says so for other kinds of account rather than greying out (14-03 choice 3).
- **D-15. His main calendar's row is named "Google Calendar"** unless he named it himself at
  Google; every other calendar gets a `google:` row with his name for it, else Google's
  (14-04 choice 1).
- **D-16. A calendar hidden in Google's own view starts hidden here,** and his choice
  afterwards is kept (14-04 choice 2).
- **D-17. A calendar that shows only free and busy times is not brought,** and the sync says
  once how many were passed over (14-04 choice 3).
- **D-18. A calendar Google stops listing is put away only after a complete read of the
  list,** keeping any change he made here and has not sent (14-04 choice 4).
- **D-19. The crash entry naming its build moved from 14-05 to 14-06,** which makes one crash
  writer for the panic hook and the page window (14-05 premise 4).
- **D-20. Refresh's help sentence, "Read this folder again from the server", is a ledger
  entry, and neither it nor F5 changes in this phase.** F5 reads this computer's store;
  whether F5 should ask the server or the sentence should say what F5 does is his to choose
  (14-07 premise 6).
- **D-21. Sending's words say it was proven once, on a Gmail account on 18 September 2026,
  and every other path says "not proven" rather than "never run",** since the replays of
  2026-09-20 ran deletes against Gmail unheard (14-07 premise 1).

## Corrections made when the plans were brought together

The three ranges were planned side by side and each passed its plan check with warnings.
These were acted on on 2026-10-05 when the plans were brought together, each edit made in the
plan it names.

- 14-07 looked for 14-03's control "in the account editor by grep". It now looks in
  `build_account_manager_dialog` in `src/presentation/wx_account_manager.rs`, where 14-03
  builds it (premise 4 and task 1's reading list).
- 14-09's step 5 and 14-12's step 3 opened the account editor for that sign-in. Both now
  choose the account in the Account Manager's list and press its button, and 14-12 reads
  REAL-01's "the sync run on account creation" as D-13's two steps.
- The four tracer tasks, task 1 of 14-01 to 14-04, now carry `tdd="true"`, as 45 of the 49
  tracers in earlier phases' plans do (counted with the Grep tool over `.planning/phases`), so
  the RED and GREEN gate check under `workflow.tdd_mode` counts them; each already began with
  a red commit under `Fails-until-green:`.
- 14-09's posting check looks for "Read by 14-09" on both #22 and #63, as 14-10 to 14-12 do
  on #63, rather than for "14-09" anywhere on #22.
- 14-11's and 14-12's ledger checks read the entries each plan names (376, 377, 544, 546 and
  625; 183, 187, 188, 191, 547, 770, 771, 784, 787 and 804) for a line naming the plan,
  rather than passing on any line in the ledger that mentions it. Both are red today: a
  search of `.planning/WINDOWS.md` for either plan's number finds nothing.
- 14-05's verification now reads that every string it adds reaches only the log, because its
  decision not to push rests on nothing spoken or shown changing.

## What the planners found that the proposal did not say

- **F5 does not read from the server.** Its help says "Read this folder again from the
  server" (`wx_app.rs:7479`), and its handler reloads this computer's store; a check compares
  a whole folder with the server at most every six hours (`finding_what_was_deleted.rs:170`).
  So the fourth reading of a proof is Gmail's own view on the web or his phone, not F5 (14-09
  premise 4, 14-10 premise 2), and the sentence is D-20's ledger entry.
- **A message over 25 MB cannot be made here.** The composer refuses an attachment over 25 MiB
  (`attaching.rs:27`) and Gmail's sending limit is 25 MB, so ledger 547's question needs an
  account that already holds such a message, or it stays open and 14-13's wording says so
  (14-12 premise 3).
- **A log line can be caught in a test:** `CapturedLogs` (`screen_reader.rs:723`), which
  research said the project did not have (14-02 premise 3).
- **Google gives one meeting the same event identity in every calendar it sits in,** and
  events are looked up by that identity across all of an account's calendars, so a meeting in
  two would flip between them and a cancellation in one would delete the other; 14-04 looks
  up within the calendar (14-04 premise 4).
- **`scripts/build-installer.sh` reads two values with `sed`,** at `:12` and `:143` (14-08
  premise 1), which is the second item under "What waits on Pratik".
- **Windows' list of installed programs shows the full build string,** because the installer
  writes `AppVersion` as `DisplayVersion`; `1.0.0.14999` is only in the setup file's own
  Properties, and the installed `wixen-mail.exe` shows `1.0.0.0` (14-08 premise 11).
- **Gmail's ImapSettings page states no default for what an expunge does,** so sitting 3 asks
  him to read his own setting rather than assuming one (14-11 premise 3).
- **Ledger 377 may be heard in sitting 3:** a held change's sentence points him to a Settings
  box that is already on; if he hears it, 14-11 names a fix plan, 14-11.1.

## What waits on Pratik

In the order the plans meet them.

1. **Nothing before 14-01 to 14-04 run,** each on the decisions above unless he overrules one.
   The ones most worth his look are D-11 (F5 in a module syncs it), D-12 and D-13 (the button
   on the Account Manager, Alt+T, and no browser opening by itself), D-15 to D-18 (how his
   main, hidden, free-and-busy and dropped calendars are handled), and D-06 and D-07 (the
   first reason only, with the attention cue).
2. **Before 14-08 starts: whether its executor may run `scripts/build-installer.sh` as it
   is.** The script reads the version and the alpha step with `sed` (`:12`, `:143`) and
   changes no file, but this phase's briefs say not to run that tool at all. Yes, and the
   build runs as written; no, and a small red and green plan rewrites those two reads first,
   inserted before 14-08, since `tests/installer.rs` already reads the script. His twelve
   answers do not settle it, and 14-08's task 1 does not start until his ruling is written
   here or in `STATE.md`.
3. **Any time before sitting 1: his Google testing key,** with the Calendar, People and Tasks
   interfaces turned on and himself as the tester, and `oauth.toml` placed in his settings
   folder (answer 1). Help's page from 14-03 walks through it; 30 to 60 minutes if he has no
   project yet, by the proposal's estimate.
4. **14-08: installing the build himself,** 5 to 10 minutes, and one line back: "installed,
   key placed", "installed, key later", or what went wrong.
5. **The four sittings:** 14-09, 40 to 60 minutes; 14-10, 45 to 60; 14-11, 25 to 35, starting
   with reading his own Gmail IMAP expunge setting; 14-12, 60 to 80, after adding the accounts
   he has, then about two minutes on the days the 15-day Trash emptying and the one-day POP
   removal fall due. Each ends with his one-line word on the drafted record.
6. **14-13: his word per path** on the drafted warning sentences, and "close" or "leave open"
   for #22 and #63, about 10 minutes.
7. **Any fix plan a sitting names** (14-09.1 and so on) is inserted with him; each costs an
   install and that line taken again.
8. **Pushing `main`** stays his word.

## What only a person or a real provider can settle

- Whether Google answers his key and his sign-in the way Google's pages say, and whether the
  console's pages read well under NVDA, which 14-03's page says it has not checked.
- Every proof line: his ear, his accounts and each provider's own view, the four readings of
  PROPOSAL.md's "How a sitting works", agreeing.
- What Gmail does with an expunge under his setting, with an appended message, and with the
  same message appended twice.
- The listening items each sheet carries.

## Handover

14-01 is next, on a branch cut from `main` at the commit landing these plans. Nothing needs
Pratik's time until 14-08, apart from his testing key, which he can make any time before
sitting 1, and his ruling on the build script, which 14-08's first task waits for. The commit
landing these plans is on `main` and not pushed; before it, `main` was level with
`origin/main`.
