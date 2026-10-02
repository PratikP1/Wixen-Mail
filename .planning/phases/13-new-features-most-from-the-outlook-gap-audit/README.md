# Phase 13: New features, most from the Outlook gap audit

Fifty-three plans, one per wave, written on 2026-09-24 against `main` at
`630e2a67`, version `1.0.0-alpha.1`. Fifty-one carry the numbers
`RESEARCH-CROSS-CHECK.md` gave them; two were added by the planners as
decimals: 13-17.1 (keys locked with a passphrase, wave 18) and 13-24.1 (the
runner, split from 13-24 at its task boundary, wave 26). On the day,
`guards/guards.toml` held 1,088 records by the TOML reader,
`.planning/WINDOWS.md` 609 entries with 550 open and 59 fixed (`head -7`),
and the last whole gate was phase 12's, 8,985 tests at `566116d3`.
`git log origin/main -1` read `630e2a67`, so `main` was pushed at the close
of phase 12 and `git rev-list origin/main..HEAD --count` was 0 before the
commit that lands these plans; that commit is on `main` and not pushed.

Pratik confirmed the order on 2026-09-24 and said "plan and implement phase
13". Every recommendation in the five research documents' question lists
was taken as the decision that day and is recorded in each plan as "taken
2026-09-24 from the research's recommendation; Pratik may overrule". Four
things are not taken and wait on him, each as a checkpoint or a ledger
entry: (a) to (d) under "Four things that wait on Pratik".

**Checked the same day, in five ranges.** Each range's planner wrote its
plans and a checker read them. Two blockers, twenty-six warnings and one
note came back, and the integration commit applied what the plans could
carry:

- The waves collided across ranges: 13-35 and 13-37 both sat at wave 37,
  13-36 and 13-38 at 38, and 13-37 sat below its own dependency. Every plan
  from 13-37 on moved two waves later, so the waves run 1 to 72 with no
  two plans on one.
- The 13-24 split had not reached the plans that call the runner. 13-40,
  13-42, 13-43 and 13-44 now depend on 13-24.1, load its summary, and say
  that the runner answers a `WhatWasDone` and speaks no sentence, so each
  caller says its one sentence.
- 13-51 now names 13-17.1 in `depends_on`, sits at wave 53, and its premise
  about the decimals is rewritten.
- The warnings applied in the plans: 13-03 says why `WIXEN_NO_PDF_PRINTER`
  goes into `release.yml` while `WIXEN_NO_AUDIO` waits for Pratik; 13-06's
  first task says its verify cannot show the reading red and the red commit
  is the evidence; 13-01 re-measures all three records naming the Undo Send
  reading, not one; six acceptance lines quoting a fixed test count in
  13-01, 13-02, 13-04 and 13-09 now quote the count at the start of the
  plan; 13-16 no longer stops if its
  honest test store reddens a sign-in test, the split belongs in 13-16
  (settled here, below); 13-20 corrects the `cms` comment on both of
  Pratik's answers; 13-13, 13-17.1 and 13-21 comment on #50, #49 and #52
  after their merges, as 13-25 does on #54; 13-37 to 13-44 no longer tick
  a box in this README, which keeps none.
- The warnings left as they are, because they are sizes or readings rather
  than defects: twelve plans carry four tasks (13-03, 13-11, 13-19, 13-20,
  13-21, 13-28, 13-39, 13-41, 13-42, 13-44, 13-45, 13-47), 13-03 the
  largest with nineteen files; 13-19's premise counts with `| wc -l`, a
  reading and not a verify; 13-51's action counts plans with `| wc -l`,
  the same.

**Goal.** The features the Outlook gap audit of 2026-08-27 and the first
day of testing found absent or partial arrive as features a screen reader
user can work: print; Undo and Redo for text and then for actions on items;
a PGP key manager; meeting invitations shown, said and answered, with
updates and cancellations reaching the calendar; encrypted mail read and
sent, S/MIME and PGP/MIME, with signatures checked; junk reported and a
block that moves the mail already here; directory lookup through Graph and
LDAP; free/busy from every source an account has; saved searches ordered,
keyed and made from nothing; several addresses per account; Quick Steps; a
rule run over a folder on demand; and the rest of mail import and export.

**Requirements:** GAP-01 to GAP-14 in `.planning/REQUIREMENTS.md`. GAP-01 to
GAP-13 are one per issue, written 2026-09-20, and none was added when the
phase was planned. GAP-14 was added on 2026-10-01 for 13-44.6 and 13-44.7,
from Pratik's request of 2026-09-29 rather than from an issue (decisions 109
to 130), and 13-44.8 and 13-44.9 add dated lines under it on his answers of
2026-09-30 (decisions 132, 142 to 149 and 151 to 156); until then this
paragraph named thirteen and said no requirement was added.

**Roadmap success criteria this phase owns:** all fourteen, the fourteenth
added with GAP-14.

## Pratik's order, and which part of it this is

This is the sixth of the seven groups Pratik agreed on 2026-09-16 (phase 9's
README): #45, #47, #49, #50, #52, #54, #55, #57, #58, #59, #60, #61 and
#53's points 4 to 6. Phase 12's README listed the order inside it as his to
confirm (its Decisions for Pratik, item 11); he confirmed it on 2026-09-24.
The order, in five groups, each researched on its own:

| Group | Research | Plans | Issues |
|---|---|---|---|
| Keyboard basics | `RESEARCH-1-keyboard-basics.md` | 13-01 to 13-09 | #47 (Undo), #45 (Print) |
| Reading mail | `RESEARCH-2-reading-mail.md` | 13-10 to 13-21 with 13-17.1 | #50, #52, #49 |
| Provider features | `RESEARCH-3-provider-features.md` | 13-22, 13-25 to 13-36 | #54, #55, #57, #59 |
| Automation | `RESEARCH-4-automation.md` | 13-23, 13-24, 13-24.1, 13-37 to 13-44 | #58, #60, #61, and #54's block through the runner |
| Import and export | `RESEARCH-5-import-export.md` | 13-45 to 13-50 | #53 points 4 to 6 |

13-51 closes the phase. Three automation plans sit inside the provider
group on purpose: the cross-check moved the label fix (13-23) and the runner
(13-24, 13-24.1) ahead of the block (13-25), which is the runner's first
caller.

## The plans

"Spoken or shown" is whether the plan's branch is pushed with a pull request
under Pratik's standing OK of 2026-09-23 (a plan that changes what is spoken
or shown pushes; others merge locally). "Waits on" names the checkpoint or
ledger entry a plan carries for one of the four answers.

| Plan | Wave | What it does | Requirement | Spoken or shown | Waits on |
|---|---|---|---|---|---|
| 13-01 | 1 | Edit, Undo and Redo on the main window on the box's own one step, greyed while the menu is open and speaking when pressed; Undo Send third, U to N | GAP-02 | yes | nothing |
| 13-02 | 2 | What a printed page holds and where it breaks: a pure layout module and one field list speech and paper share | GAP-01 | no | nothing |
| 13-03 | 3 | File, Print and Ctrl+P on the message list through Windows' print dialog, drawn with GDI and spooled as one job; the spool target and `WIXEN_NO_PDF_PRINTER` in four workflows | GAP-01 | yes | (b) checkpoint; (d) drafted, ledgered, not posted |
| 13-04 | 4 | Print from the reader window, a conversation row, the formatted conversation window and the other five modules; GAP-01 ticked, #45 closed | GAP-01 | yes | nothing |
| 13-05 | 5 | A history of several steps for each of the main window's text boxes, 100 at most | GAP-02 | yes | nothing |
| 13-06 | 6 | The same history in every dialog's text box, password boxes left out, held by a source reading | GAP-02 | yes | nothing |
| 13-07 | 7 | Undo and Redo of a mark, a star or a label, the message named, one step, no timer | GAP-02 | yes | nothing |
| 13-08 | 8 | Undo of a move, a delete to the trash and a copy, here first and then at the server; GAP-02's box ticked | GAP-02 | yes | nothing |
| 13-09 | 9 | Undo in contacts, calendar, tasks, notes and reminders, with `take_a_deletion_back`; #47 closed | GAP-02 | yes | nothing |
| 13-10 | 10 | The invitation said before the body on every surface that opens a message | GAP-04 | yes | nothing |
| 13-11 | 11 | Accept, Tentative and Decline as buttons in both reader windows, the answer remembered, the reply threaded | GAP-04 | yes | nothing |
| 13-12 | 12 | Synced events carry their iCalendar UID and organiser | GAP-04 | no | nothing |
| 13-13 | 13 | The organiser's update moves the meeting and a cancellation is removed with one button; GAP-04 ticked, #50 commented on | GAP-04 | yes | nothing |
| 13-14 | 14 | S/MIME encrypted mail opens through `CryptDecryptMessage`, decrypted each time, never stored | GAP-05 | yes | nothing |
| 13-15 | 15 | PGP/MIME opens | GAP-05 | yes | nothing |
| 13-16 | 16 | Keys split across credential entries that fit Windows' 1,280-character limit, several keys, other people's public keys | GAP-03 | no | nothing |
| 13-17 | 17 | The PGP key manager on File, "PGP &Keys... (experimental)", where Import PGP Private Key was | GAP-03 | yes | nothing |
| 13-17.1 | 18 | Keys locked with a passphrase, asked when a message first needs one and held until Wixen Mail closes; GAP-03 ticked, #49 commented on | GAP-03 | yes | nothing |
| 13-18 | 19 | PGP signatures checked and said | GAP-05 | yes | nothing |
| 13-19 | 20 | S/MIME signing and encrypting, the service half | GAP-05 | no | nothing |
| 13-20 | 21 | OpenPGP signing and encrypting, the service half; the false `cms` comment in `Cargo.toml` corrected | GAP-05 | no | (b) checkpoint |
| 13-21 | 22 | Sign and Encrypt in the composer; GAP-05 ticked, #52 commented on | GAP-05 | yes | nothing |
| 13-21.1 | 23 | A time read in the zone it was written in, Windows names through ICU in `common::zones`; the invitation said at this computer's hour, the other zone's clock once; ledger 632 | GAP-04 | yes | nothing |
| 13-21.2 | 24 | The calendar's list, readings, page, Calendar window, alerts and editor on this computer's clock; Outlook's UTC times at their real hour; ledger 632 closed | GAP-04 | yes | nothing |
| 13-21.3 | 25 | A message the download brought before 13-10 fetched once for its parts when it is selected, through its own account, and its meeting said with no second announcement; ledger 633 | GAP-04 | yes | nothing |
| 13-22 | 26 | Report as Junk on Action (J, Ctrl+Shift+J), told to the provider where it listens and said where it does not | GAP-06 | yes | nothing |
| 13-23 | 27 | A rule that adds a label puts that label on | GAP-11, GAP-12 | yes | nothing |
| 13-24 | 28 | The five set actions split into quiet do-halves, nothing heard meant to change | GAP-11, GAP-12, GAP-06 | yes, paths move | nothing |
| 13-24.1 | 29 | One runner for several actions over a set, answering a `WhatWasDone`; a flag change carrying the folder it was asked in | GAP-11, GAP-12, GAP-06 | yes, paths move | nothing |
| 13-25 | 30 | A block moves the sender's mail already here after one question with the count; Block on Ctrl+Shift+B; GAP-06 ticked | GAP-06 | yes | nothing |
| 13-26 | 31 | The directory sign-in in the credential store, never sent in clear, and the `ldap3` panic fix | GAP-07 | yes | nothing |
| 13-27 | 32 | The directory sign-in window from the Account Manager | GAP-07 | yes | nothing |
| 13-28 | 33 | Microsoft's people search in People found, asked with a token of its own; GAP-07 ticked | GAP-07 | yes | (c) checkpoint |
| 13-29 | 34 | Google as a free/busy source | GAP-08 | yes | nothing |
| 13-30 | 35 | Free/busy asks every source an account has and merges the answers per person | GAP-08 | yes | nothing |
| 13-31 | 36 | A guest's own zone from Microsoft's answer, each time on at most three clocks; GAP-08 ticked | GAP-08 | yes | nothing |
| 13-32 | 37 | Edit Event scrolls, every field reachable at 768 pixels and at 200% (a conditional plan, in) | GAP-08 | yes | nothing |
| 13-33 | 38 | Other addresses to send from, stored and managed from the Account Manager | GAP-10 | yes | nothing |
| 13-34 | 39 | The outbox and drafts keep the address a message was written from | GAP-10 | no | nothing |
| 13-35 | 40 | Compose's From list offers the other addresses and sends from the one chosen | GAP-10 | yes | nothing |
| 13-36 | 41 | A reply goes out from the address it was sent to; GAP-10 ticked, #59 left open for shared mailboxes | GAP-10 | yes | nothing |
| 13-36.1 | 42 | A PGP/MIME message holding only files says so where its words would be, and an S/MIME one's page stops saying it may not have been downloaded; the key list's Created and Expires follow the Reading tab; stored-before said only below the mark this build notes in the database, a signature file on later mail saying its form is not checked; ledgers 643, 649, 653 | GAP-05, GAP-03 | yes | nothing |
| 13-36.2 | 43 | A meeting answered before the calendar check brought it and the provider's copy become one meeting on Google, Outlook and calendar servers, the provider's copy on the answer's row; ledger 154's other order | GAP-04 | no | nothing |
| 13-36.3 | 44 | An organiser's update or cancellation naming one day of a repeating meeting changes that day only, one naming no day is applied to every day, and Remove from Calendar takes one day off; ledger 638's first half | GAP-04 | yes | nothing |
| 13-36.4 | 45 | One day of a repeating meeting answered for that day with the reply naming the day, and a whole repeating meeting answered keeps its repeat, a defect fixed; ledger 638 closed | GAP-04 | yes | nothing |
| 13-37 | 46 | Saved searches keep an order somebody chooses, moved with the tree's gesture | GAP-09 | yes | nothing |
| 13-38 | 47 | Alt+4 to Alt+9 for the first six saved searches, and the Saved Searches menu | GAP-09 | yes | nothing |
| 13-39 | 48 | A saved search made from nothing, every or any changeable afterwards; GAP-09 ticked, #58 closed | GAP-09 | yes | nothing |
| 13-40 | 49 | Quick Steps as data | GAP-11 | no | nothing |
| 13-41 | 50 | The Quick Step Manager under Action, Quick Steps | GAP-11 | yes | nothing |
| 13-42 | 51 | Quick Steps on the menu with Ctrl+Shift+7 to Ctrl+Shift+9, run over the selection through the runner; GAP-11 ticked, #60 closed | GAP-11 | yes | nothing |
| 13-43 | 52 | What a rule would change in a folder, counted and worded before anything runs | GAP-12 | no | nothing |
| 13-44 | 53 | Run a rule over a folder from This Folder and the Filter Manager; GAP-12 ticked, #61 closed | GAP-12 | yes | nothing |
| 13-44.1 | 54 | Undo after Report as Junk takes `$Junk` off and puts `$NotJunk` on at the server where the report may have set a mark, then moves the messages back; a block is written to the account the message is in; ledgers 677 and 691 | GAP-06 | yes | nothing |
| 13-44.2 | 55 | Look People Up at Work refuses a sign-in over `ldap://` at OK with the lookup's own sentence, focus on the address; a credential store failure names the account by its name, never its id or the password; ledgers 696 and 699; and, added when it landed on 2026-10-01, step 8c of the marker reading made to hold on every run, its cause found first (ledger 754) | GAP-07 | yes | nothing |
| 13-44.3 | 56 | A rule's marks, flags and labels on arriving mail reach the mail server in the check that brought the message, through the gate and waiting queue the menu commands use, and a rule's Delete goes to the Trash there through the gated delete path the menu's Delete uses (decision 107); the next check keeps them and does not bring a deleted message back; ledger 678 closed | GAP-12 | yes | nothing |
| 13-44.4 | 57 | A colleague Microsoft places is judged by their own working days and hours from getSchedule, everybody else by the working day set here; ledger 711 closed | GAP-08 | yes | nothing |
| 13-44.5 | 58 | One check says whether an account is Gmail or Microsoft, by its incoming server, then its address, then the name it was saved with; Report as Junk, the folder chooser and sign-in ask it, so Workspace and Microsoft 365 accounts on their own domains are recognised and can sign in through the browser; on Pratik's answer to decision 140, the account editor's app password advice and Get App Password button ask it once a server is typed, an account it calls Microsoft is told to use the browser sign-in (decision 150), and Send Feedback's list of account kinds asks it | GAP-06 | yes | nothing |
| 13-44.6 | 59 | Empty the Trash, per account on Alt+Y in the account editor and marked experimental: an IMAP account the one check of 13-44.5 does not call Gmail or Microsoft, set to After 15 days or After 30 days, has what went into its Trash here that long ago taken off the server through the menu's Delete path at its first check of the day, at most 500, with one sentence; Gmail and Microsoft accounts are told their provider empties it; the mail database records when a message went into a Trash | GAP-14 | yes | nothing |
| 13-44.6.1 | 60 | The passphrase box's real paste in a test target of its own that says why a refused clipboard was refused (the session locked as Windows reports it, another window holding it, or neither), runs taking turns at the shared test clipboard, the dialog's reading opening no clipboard, and a census keeping other targets off it, so a locked Windows session no longer fails them; ledger 716 | GAP-03 | no | nothing |
| 13-44.6.2 | 61 | The ten tests that send keys to plain controls, and the directory sign-in, build their windows on a desktop made for each run, where nothing a person types, no locked session and no second run reaches them; a census beside the clipboard census keeps every key sender off the interactive desktop; ledger 580 | FOUND-23 | no | nothing |
| 13-44.6.3 | 62 | The five tests that send keys and hold a browser or read from a second copy of themselves run their window tests in a child process on a desktop made for each run; the four with a browser take 13-44.6.1's turn, renamed for what it guards, and the marker reading's own turn goes; a failed child says why and what Windows says about the lock; ledger 761 | FOUND-23 | no | nothing |
| 13-44.7 | 63 | When Wixen Mail closes, with the window gone first and five seconds for every account, nothing lost; POP accounts emptied on this computer only, of their own messages; a POP message moved to the Trash no longer downloaded again (ledger 753 closed); GAP-14 ticked | GAP-14 | yes | nothing |
| 13-44.8 | 64 | Emptying a POP Trash, Empty Folder on the Trash, Delete in the Trash and Delete Permanently take a message kept on this computer alone off it with its words, keeping only its POP identifier, when it was downloaded, whose it was and where; the freed space is overwritten and the search index lets go at the next check | GAP-14 | no, unless its premise 9 finds 13-44.7's description saying the text stays | nothing |
| 13-44.9 | 65 | Measures, at 12,872 and 200,000 messages, what taking mail off still leaves in the mail database file and its write log, and what secure delete on every write (off, fast and on) and a compacting command (VACUUM, and incremental vacuum) cost in time, temporary disk and waiting; the figures on the measurements page, a recommendation by a rule written beforehand, and a ledger entry asking Pratik to choose; builds neither | GAP-14 | no | nothing |
| 13-45 | 66 | One folder out as a bare mailbox file (Alt+F, then F) | GAP-13 | yes | nothing |
| 13-46 | 67 | One folder out as loose message files (Alt+F, then X) | GAP-13 | yes | nothing |
| 13-47 | 68 | The `.msg` reader | GAP-13 | no | (a) checkpoint |
| 13-48 | 69 | `.msg` through both import commands | GAP-13 | yes | nothing |
| 13-49 | 70 | The pages say which export is built and why `.pst` export is not; GAP-13 ticked, #53 closed | GAP-13 | yes | nothing |
| 13-50 | 71 | Imported messages keep their files (a conditional plan, in) | GAP-13 | yes | nothing |
| 13-51 | 72 | The pages, the listening lines, the closing read of GAP-01 to GAP-14, and `scripts/check.sh all` once by hand before its merge | all fourteen | yes | reports where (a) to (d) stand |

Fifty-seven plans push and fifteen do not (13-02, 13-12, 13-16, 13-19,
13-20, 13-34, 13-36.2, 13-40, 13-43, 13-44.6.1, 13-44.6.2, 13-44.6.3,
13-44.8, 13-44.9, 13-47). Four carry a checkpoint (13-03, 13-20, 13-28,
13-47, each `autonomous: false`), and each stops only when its executor's
brief does not carry Pratik's answer.

**Requirement coverage.** GAP-01 by 13-02 to 13-04 (ticked by 13-04);
GAP-02 by 13-01 and 13-05 to 13-09 (the box ticked by 13-08, #47 closed by
13-09); GAP-03 by 13-16, 13-17 and 13-17.1 (ticked by 13-17.1), and 13-36.1
for the key list's dates (ledger 649), and 13-44.6.1 for its paste proof moved
to a test target of its own (ledger 716); GAP-04 by 13-10 to 13-13 (ticked by
13-13), 13-21.1 and 13-21.2 for times written in another zone (ledger 632),
13-21.3 for mail brought before 13-10 (ledger 633), 13-36.2 for a meeting
answered before its calendar check (ledger 154), and 13-36.3 and 13-36.4 for
one day of a repeating meeting (ledger 638); GAP-05 by 13-14, 13-15 and 13-18
to 13-21 (ticked by 13-21), and 13-36.1 for a PGP/MIME message holding only
files and the stored-before reason (ledgers 643 and 653); GAP-06 by 13-22,
13-24, 13-24.1 and 13-25 (ticked by 13-25), 13-44.1 for Undo of a report
and the block's account (ledgers 677 and 691), and 13-44.5 for the one
check of whether an account is Gmail or Microsoft; GAP-07 by 13-26 to 13-28
(ticked by 13-28), and 13-44.2 for a sign-in over `ldap://` refused at OK and
the account's name in the credential store's sentences (ledgers 699 and
696); GAP-08 by 13-29 to 13-32 (ticked by 13-31, 13-32 after it), and
13-44.4 for a colleague's own working hours (ledger 711); GAP-09 by 13-37 to
13-39 (ticked by 13-39); GAP-10 by 13-33 to 13-36 (ticked by 13-36); GAP-11
by 13-23, 13-24, 13-24.1 and 13-40 to 13-42 (ticked by 13-42); GAP-12 by
13-23, 13-24, 13-24.1, 13-43 and 13-44 (ticked by 13-44), and 13-44.3 for a
rule's marks, flags and labels on arriving mail reaching the server (ledger
678) and a rule's Delete going to the Trash there (decision 107); GAP-13 by
13-45 to 13-50 (ticked by 13-49, 13-50 after it); GAP-14, added on
2026-10-01, by 13-44.6 and 13-44.7 (ticked by 13-44.7), 13-44.8 after
the tick for emptied mail's words taken off this computer, and 13-44.9
after it for measuring what that leaves and what reaching it would cost.
13-51 reads all fourteen clause by clause and stands or corrects each tick.
13-44.6.2 and 13-44.6.3 serve no GAP requirement: they move the tests that send keys onto
desktops made for each run, and name phase 12's FOUND-23, whose commit gate those tests stalled,
with a dated line each and no `[D]` clause.

**The issues.** Closed from the merge commit: #45 (13-04), #47 (13-09), #58
(13-39), #60 (13-42), #61 (13-44), #53 (13-49). Commented on from the merge
commit and left open for Pratik, because each has a line only a real
account settles: #50 (13-13), #49 (13-17.1), #52 (13-21), #54 (13-25), #55
(13-28), #57 (13-31), #59 (13-36, which also stays open for shared
mailboxes and delegation). Closing an issue is not a publish; filing or
editing any other issue is not an executor's.

## One plan per wave, and why this order

Every plan writes `guards/guards.toml` and `.planning/WINDOWS.md` (53 of
53), 49 write `docs/changelog.md`, and 32 write
`src/presentation/wx_app.rs`, each counted from the plans' `files_modified`
lists on 2026-09-24. A wave is a set of plans sharing no file, and with the
files the project writes by rule added to every list (`CLAUDE.md`, "Add the
files this project writes by rule to every plan's `files_modified`") no two
plans are disjoint. So the phase is a chain: each plan depends on the one
before it, and the waves run 1 to 72 in plan order.

- **Keyboard basics first** (13-01 to 13-09). Undo and Print are small,
  touch every surface, and every later plan that adds a text box or a
  list gets them for free: 13-06's reading makes every dialog box keep a
  history, so a box 13-27 or 13-33 adds arrives with one.
- **Reading mail second** (13-10 to 13-21). Invitations first, because
  they are read in the reader every later reading plan changes; then
  encrypted mail read (13-14, 13-15) before the key manager, on the
  research's question 5 confirmed by the order; keys (13-16 to 13-17.1);
  then signatures and sending, which need the unlocked key.
- **The provider features third** (13-22 to 13-36), with the runner inside
  them: Report as Junk, then the label fix and the runner (13-23 to
  13-24.1) that the block (13-25) calls; the directory (13-26 to 13-28);
  free/busy (13-29 to 13-32); identities (13-33 to 13-36).
- **Automation fourth** (13-37 to 13-44): saved searches, then Quick Steps,
  then a rule over a folder, the last two calling the runner built in
  13-24.1.
- **Import and export fifth** (13-45 to 13-50), after 13-03 and 13-17 took
  their File menu letters.
- **The close last** (13-51), in the highest wave, so it reads the pages and
  the tree as every plan before it left them.

## Decisions taken on 2026-09-24

Each was taken that day from the research's recommendation, or is the
planner's own choice where marked, and each is recorded in the plan that
carries it. **Every one is taken, and Pratik may overrule it**; an overrule
is a change to the named plan before it runs, or a plan of its own after.

**Keyboard basics** (RESEARCH-1, section 3):

1. Plain text first: the page carries the header lines and the words, no
   pictures (13-02, 13-03).
2. No Print Preview; Windows' dialog has none (13-03).
3. The print job is named plainly by kind, "Wixen Mail message" for a
   message, because a shared printer's queue is read by other people
   (13-02, 13-03).
4. Undo takes U and Redo R, first on Edit; Undo Send moves third and from
   U to N, which moves a letter a tester may have learned (13-01).
5. A fixed 11 point size, no new setting (13-02, 13-03).
6. One sentence when a job is sent; a cancelled dialog says one sentence
   too (the planner's) (13-02, 13-03).
7. Several steps of undo in every text box, in this phase (13-05, 13-06);
   at most 100 per box and a password box keeps none (the planners').
8. An item undo lasts until the next action on items, one step, no timer
   (13-07 to 13-09).
9. Undo in the other five modules, in this phase (13-09).
10. Redo on a plain box only right after an Undo in that box, because
    Windows' one step makes a Redo after typing undo the typing (the
    planner's, 13-01).
11. Crossings between accounts are refused with a sentence, not undone;
    Delete Permanently is undone while it still waits here; an undone
    copy goes to the trash, here and at the server (13-08).
12. PDFPurr 0.4.0 misreading Microsoft Print to PDF files is ledgered for
    his schedule, not fixed (13-03).

**Reading mail** (RESEARCH-2, section 7):

13. The invitation's sentence comes after the encryption notice and before
    the signature notice (13-10).
14. An invitation inside decrypted mail is shown and offered the buttons,
    and never changes the calendar on its own (question 9; 13-10, 13-11,
    13-13 to 13-15).
15. No answer button is ever greyed; with no answer possible there are no
    buttons and the reason is on the bar. Accept is "A&ccept", because
    Alt+A is the attachments key in both reader windows (13-11).
16. An organiser's update is applied when the message is opened and said;
    a cancellation is shown with a Remove from Calendar button; neither
    for a sender who is not the organiser (question 3; 13-13).
17. A repeating meeting on a Microsoft account is said and not worked
    around, because Graph gives each occurrence its own `iCalUId`
    (13-12).
18. Decrypted mail is never stored, and search does not look inside
    encrypted mail (question 4; 13-14, 13-15).
19. Private keys are split across fixed-name credential entries, so the
    uninstaller names them without reading anything (question 1, option
    a; 13-16). Other people's public keys and certificates are kept in
    the mail database (question 7; 13-16, 13-19).
20. A key locked with a passphrase is asked for when a message first needs
    it and held until Wixen Mail closes, never stored (question 2, option
    b), as its own plan, 13-17.1; there is no command to forget typed
    passphrases sooner (13-17.1).
21. The key manager is on File as "PGP &Keys... (experimental)", on K,
    where Import PGP Private Key was, which goes (Pratik's placement and
    question 8; 13-17).
22. S/MIME encrypts the key with PKCS #1 v1.5 until a real Outlook and
    Apple Mail round trip shows OAEP opens (question 6; 13-19); OpenPGP
    uses SEIPD v1 (13-20); both encrypt to the sender too, so the Sent
    copy can be read (13-19, 13-20).
23. When both S/MIME and PGP could protect a message, S/MIME goes first; a
    draft keeps its Sign and Encrypt choice; Sign and Encrypt are never
    greyed and the reason is said at Send (the planner's; 13-21).

**Provider features** (RESEARCH-3):

24. A Microsoft account's junk report moves the message to Junk Email and
    says Microsoft has not been told; no beta call, no new permission
    (question 1; 13-22). Gmail gets the move alone, no keyword (the
    planner's; 13-22).
25. Report as Junk on Ctrl+Shift+J and J on Action; Block This Sender on
    Ctrl+Shift+B (question 3; 13-22, 13-25).
26. A block asks once, with the count, over every folder but Junk, Trash,
    Sent and Drafts, Yes the default; above 5,000 it refuses with the
    count and the bound (question 2; 13-25).
27. A password is refused over `ldap://`; `ldap3` is kept; one credential
    service name per account (question 6; 13-26).
28. The directory sign-in has a window of its own from the Account Manager,
    its button on L, with no Forget button (question 5, option a; 13-27).
29. `/me/people` on v1.0 now; a third "from Microsoft" source on each row
    (13-28). One token per new permission rather than a longer shared list
    is the planner's design, not the research's, and is offered to Pratik
    beside (c) at 13-28's checkpoint.
30. Google's free/busy needs no new permission; its `notFound` is a reason
    of its own, "their calendar is not shared with you" (13-29).
31. A person answered by any source is known, busy stretches united; a
    personal Microsoft account's refusal is ledgered, not guessed at
    (13-30).
32. A guest's zone comes from Microsoft's answer now and from a contact's
    field later; Windows' ICU maps the names; at most three clocks per
    time (question 7; 13-31).
33. Edit Event's scrolling is in this phase, after free/busy, through
    wxWidgets' own scrolling (question 8; 13-32, the first conditional
    plan).
34. An other address signs with its account's signature; typed addresses
    first and Gmail's "Send mail as" list later; the manager is on the
    Account Manager with the manager loop's letters (questions 9 and 11;
    13-33).
35. A row written before 13-34 has no address and goes out as it always
    did (13-34). The From list is named "From", an account's own entry
    reads as it does today, and the send follows the list (13-35).
36. A reply and a forward go out from the address the message was sent to;
    other addresses count as your own in Reply All; shared mailboxes,
    sending on behalf and delegation are later work, said and not stubbed
    (question 10; 13-36).

**Automation** (RESEARCH-4, section 8):

37. A missing label is said, not created; arrival rules' read and flag
    staying on this computer is ledgered, not fixed (13-23). The second
    half is superseded by Pratik's answer of 2026-09-29, item 3 (decision
    88): 13-44.3 sends them to the mail server in the check.
38. The runner goes through the set commands' gated server paths, resolves
    folders and labels in the actions' own account, and carries the folder
    a flag change was asked in rather than waiting (question 12;
    13-24.1).
39. Above 5,000 is read as a bound on one run: the question says the count
    and the bound, a run takes the first 5,000, and running it again does
    the next (question 4; 13-43, 13-44). The recommendation said both
    "refuse" and "running again does the next"; this reading keeps both
    halves, and it is an interpretation he should see named.
40. Everything is per account (question 9). Saved searches take Alt+4 to
    Alt+9, Quick Steps Ctrl+Shift+7 to Ctrl+Shift+9 (question 1); a
    search's key does what Enter on its row does (question 2); every or
    any can be changed afterwards, in the conditions window both doors
    open (question 3); no key for Save This Search (question 11); the
    500-result cap stays, with a ledger entry and no issue opened
    (question 10) (13-37 to 13-39).
41. A step is one rule `Outcome`, one field per question, one label at
    most, because a check-box list in wxWidgets does not report its state
    to a screen reader reliably (the planner's); its folder is chosen from
    the account's folders (question 5) (13-40, 13-41).
42. The Quick Step Manager is inside Action, Quick Steps, as "&Manage Quick
    Steps..." (Pratik's placement); Action takes Q (question 7); a
    selection holding another account's message is refused, not narrowed
    (the planner's) (13-41, 13-42).
43. A switched-off rule can be run by hand, marked "(switched off)" in the
    chooser (question 6); Enter answers No for a rule that deletes and Yes
    otherwise (question 8); no context-menu entry; "the rule editor" in
    GAP-12 is read as the Filter Manager (13-43, 13-44).

**Import and export** (RESEARCH-5's eight questions):

44. The bare mailbox file is the chosen folder alone, and the sentence says
    how many folders inside it were left out (question 5; 13-45).
45. Two items beside Export Mailbox, F and X, nothing already learned
    moves; `writing_out`, reached by nothing but its tests, goes with
    `CHOOSE_THE_MESSAGES_TO_EXPORT` and `what_the_mail_export_did`
    (question 6; 13-45, 13-46).
46. A message file's name starts with the date in the message's own offset
    (question 4; 13-46).
47. A `.msg` that is not a message is refused by name (question 2; 13-47).
48. Imported messages keep their files for every import but `.pst`, as its
    own plan (question 7; 13-50, the second conditional plan).
49. `.pst` export is said plainly to be out and is not a menu item; the
    guide's advice to drag `.eml` files into Outlook says it is untried
    (question 1; 13-49).

**Settled when the plans were integrated**, the orchestrator's and not the
research's, for Pratik to overrule like the rest:

50. If 13-16's honest test store reddens a sign-in test (an OAuth token or
    a password longer than Windows' limit), the split belongs in 13-16,
    through the same parts functions, because 13-16 cannot land green
    without it; 13-16 stops only if the value is larger than the parts can
    hold.
51. This README keeps no `- [ ]` lines. The plan list with its boxes is the
    roadmap's, and a second copy would be one fact written twice.

**Pratik's answers of 2026-09-24**, to the four in the table below, recorded
by 13-03. His words: "Yes to all questions. Before you file an issue with
wxDragon, give me more details."

52. (a) `cfb` 0.15.0 for reading `.msg` files is confirmed (13-47).
53. (b) The three `windows` 0.62.2 features, `Win32_Graphics_Gdi`,
    `Win32_Storage_Xps` and `Win32_UI_Controls_Dialogs`, are confirmed, and so
    is `rand` 0.8 as a renamed direct dependency (13-03, 13-20). 13-03 added
    the three features in `e33f76fc`; `Cargo.lock` did not change.
54. (c) Microsoft Graph's `People.Read` and the tasks permission are
    confirmed (13-28).
55. (d) The wxDragon printing defect: he asked for details before anything is
    filed, and is running a test program that prints through wxDragon to
    confirm the defect first. Nothing is posted. 13-03's summary holds the
    details and a draft, and ledger 614 carries it.

**Pratik's answer of 2026-09-26**, to 13-10's two findings (ledger 632 and
633). His words: "yes. Create/modify/add to plans to resolve the
identified issues."

56. Ledger 632: a stored time is read in the zone stored beside it, where it
    is read and not where it arrives, because three writers send the stored
    clock face and zone back to the provider and a series repeats on its
    clock face; no column is added and no stored row changes (13-21.1,
    13-21.2).
57. Windows zone names, which Outlook writes, are read through Windows' own
    ICU in `common::zones`, and 13-31 builds on it rather than on a module
    of its own; 139 of the 141 names this machine knows map, and the two
    that do not are retired zones (13-21.1; 13-31 amended).
58. Taken from the research, for him to overrule: every time is said on
    this computer's clock, as Google shows invitees; the zone a meeting was
    written in is said once, in the invitation's sentence, an event's full
    reading and its printed page, only when its clock differs and it names
    a place; never in rows, button descriptions or alerts (13-21.1,
    13-21.2).
59. A zone this computer cannot place keeps the hour as written and the
    sentence says so (13-21.1).
60. The planner's: two plans, one sitting each; the event editor converts
    at the dialog's two edges; the Day and Week views' day windows and a
    Google series' offset across a change of the clocks are ledgered, not
    fixed (13-21.2).
61. Found and ledgered for him, not fixed: an Outlook invitation answered on
    a Google or calendar-server account keeps a Windows zone name that
    Google's reference does not accept and the calendar-server writer
    refuses; and the guide's "Windows 10 or later" while the program already
    needs version 1709 (13-21.1).
62. Ledger 633: the reader fetches the whole message once when the row
    says it has attachments and none are stored, stores what it learns,
    and the message is not announced twice; the download of everything
    does not keep every file, because of the 512 MB budget (13-21.3).
63. The planner's, for him to overrule: both fetches on selection go
    through the account the message is filed under, which also fixes a
    body fetched under All Inboxes from the account last opened; a
    message whose own parse finds no attachment has its row corrected so
    it is fetched once; the preview loads again only when a meeting was
    kept and the page differs, and nothing is said (13-21.3).
64. Open for him: an invitation sent only as part of the message's text,
    as Outlook sends one, is stored with no attachment bit by the header
    sync, so the accepted rule does not reach it when it arrived before
    13-10. Reaching it means fetching every earlier download once when
    selected. 13-21.3 leaves it, ledgers it, and recommends leaving it.

**Pratik's answer of 2026-09-27**, to six questions the ledger carried from
13-11 to 13-21, each asked with a recommendation in brackets. His word:
"Yes." Items 1 to 5 are carried by 13-36.1 to 13-36.4, written that day and
added on 2026-09-29; item 6 by no plan.

65. Item 1, ledger 638: one day of a repeating meeting, both halves ("a plan
    handling one day of a series, both"). An organiser's update or
    cancellation naming one day changes that day only (13-36.3), and one day
    is answered for that day only (13-36.4). Two plans, because one executor
    cannot finish both halves in one sitting; 13-36.4 is this item's second
    half, not a split of 13-36.1.
66. Item 2, ledger 154's other order: a Google, Outlook or calendar-server
    check that meets a meeting whose UID matches a row an answer filed merges
    the two rather than adding a second meeting (13-36.2).
67. Item 3, ledger 643: a PGP/MIME message that opened to files and no words
    says "This message was encrypted with PGP and was opened here. It holds
    files and no words." where its words would be, and never in the bar
    (13-36.1).
68. Item 4, ledger 649: the key manager's Created and Expires follow the date
    setting on the Reading tab (13-36.1).
69. Item 5, ledger 653: the moment this build first opens the mail database
    is noted, and the stored-before reason is given only for mail stored
    before it (13-36.1).
70. Item 6, ledger 659: encrypting a message that has a Bcc stays refused
    until phase 14 has tried a real correspondent; then each Bcc recipient
    gets an encrypted copy of their own, the only shape that keeps a blind
    copy blind. No phase 13 plan carries it, and ledger 659 records the
    answer.

The planners' choices under that answer, each for him to overrule; an
overrule is a change to the named plan before it runs:

71. The words for mail stored after the mark, taken from ledger 653's own
    recommendation, since item 5 answered only its stored-before half: "This
    message carries a signature in a form Wixen Mail does not check." Saying
    nothing would put such a message back among unsigned mail, the silence
    #52 point 6 was raised against (13-36.1, D4).
72. The mark is data in the mail database, one row of a new table,
    `last_message_before`, and not a setting. It describes the messages in
    that database, so it lives and dies with them; nobody should choose it;
    and a field no screen offers would break the rule that every setting is
    reachable from the settings screen. It is the highest message number
    when this build first opens the database, written with `INSERT OR
    IGNORE` so no later open moves it, and a mark that cannot be read gives
    decision 71's sentence, which claims nothing about when a message was
    stored (13-36.1, D5 and D6).
73. Found while planning, by reading and not running: an S/MIME message that
    opened to files alone says on its page, in the formatted window and the
    preview, that it has no text or has not been downloaded. The line that
    fixes PGP's page fixes it too. 13-36.1's red commit must show the S/MIME
    half failing, or that half is dropped and the summary says the reading
    was wrong (13-36.1, D7).
74. The key dates follow the date order and the numbers-or-words choice, not
    the relative style or the clock, as a birthday does, each day on this
    computer's clock (13-36.1, D8). A PGP/MIME message that opened to nothing
    at all keeps today's sentence, because "It holds files" would be false
    about it (D9).
75. What the mark cannot see: on a computer that already ran a build carrying
    13-18, a message with a lone signature file, or a signed message a
    mailing list wrapped, stored between that build's first run and this
    one's, is below the mark and still says the stored-before reason. The
    changelog says so (13-36.1, D10).
76. The merged meeting is the provider's copy, on the answer's row. The
    answer keeps its record and its busy, tentative or free; the title, time,
    place, guests and repeat are the provider's, so a stranger's invitation
    carrying a real meeting's UID cannot put its words on the real meeting.
    The cost: anything typed onto the answered meeting before its first
    calendar check is replaced, and the guide and the changelog say so
    (13-36.2, D2).
77. After a merge the row waits to be sent only when the answer's busy,
    tentative or free differs from the provider's copy: an Accept sends
    nothing, a Tentative or a Decline sends once (13-36.2, D3). Google keeps
    only busy or free, so a Tentative merged onto a Google copy is sent as
    busy and reads busy after the next Google read, while the answer stays
    recorded as Tentative. That half was read on `main` at `fe9bd124` and
    not run, and a ledger `todo` carries it.
78. The provider's organiser wins, and the answer's stays only where the
    provider names nobody; "a row an answer filed" is a row in the account
    whose provider identifier is still the meeting's UID, with no source,
    answered here and not one day of a series; a merge counts as updated;
    the merged row moves into the provider's calendar; a published feed is
    not merged (13-36.2, D4 to D8).
79. A database where a calendar check already put the provider's copy beside
    an answer keeps both rows. None is known (his profile held no calendar
    events, read 2026-09-27), and folding two rows means deleting one, which
    is his decision; a ledger `todo` carries it (13-36.2, D9).
80. 13-36.2 merges locally without a pull request, because no file under
    `src/presentation` changes and no NVDA or Accessibility case reaches what
    a person meets differently. He may prefer a pull request.
81. One day of a repeating meeting is kept apart the calendar's own way: an
    appointment of its own naming its series, with that day called off the
    series. That happens only where the calendar can carry one day on its
    own, a calendar server or a calendar kept here
    (`calendar::can_be_honoured`); on a Google or Outlook calendar the change
    is said, not applied, and the calendar is named. A cancelled day gets the
    same Remove from Calendar button, which takes that day off. The day is
    read on the series' clock; a change from one day onwards
    (RANGE=THISANDFUTURE) is said and not applied; a one-day message against
    a single appointment applies only when that appointment is on that day
    (13-36.3).
82. Past the literal item: an update or cancellation that names no day is
    applied to every day of the series, as decision 16 applies a single
    meeting's, because 13-13 refused it only while the day could not be read.
    Overruling this keeps 13-13's refusal for a message naming no day, with
    its sentence reworded, and takes 13-36.3's every-day cases out; the
    answer is wanted before 13-36.3 runs (13-36.3).
83. A one-day answer is filed the way 13-36.3 keeps a day apart, and an
    Accept at the series' own time is kept apart too, so the answer has a row
    to be remembered on. On a Google or Outlook calendar the answer is sent
    and the calendar left as it was, and the sentence after answering says
    so. With no series on the calendar the day is filed alone. A series sent
    with one changed day is answered as the series; a change from one day
    onwards is refused; the reply's RECURRENCE-ID is rebuilt from the value
    and the zone the invitation wrote (13-36.4).
84. Found while planning and fixed in 13-36.4's first task: answering any
    repeating meeting files one appointment, and for a series a calendar
    server holds, the next push would send the server the meeting with no
    RRULE and no EXDATE. Nothing has met a real calendar server, so it has
    not happened. A ledger `todo` opened when these plans landed, 723,
    carries it until then. He may want the fix sooner than the wave order
    puts it.
85. Found while planning and ledgered, not fixed. An answer on an account
    whose calendar nothing syncs sits in My Calendar as a change waiting to
    be sent, so every calendar check says one change made here cannot be
    sent; 13-36.2 merges it away for Google, Outlook and calendar servers and
    opens a `todo` for the rest. And the guard check
    `test_every_guard_record_still_names_one_place_in_the_tree` has been
    blind for all of `src/presentation/wx_app.rs` since 13-07's `4d7d513f`,
    through the whole-file exemption ledger 545 already names; 545 was
    updated when these plans landed, and 13-36.3 and 13-36.4 find that
    file's anchors by script.

**Pratik's answer of 2026-09-29**, to six questions the ledger carried from
13-22 to 13-36, each asked with a recommendation in brackets. His word:
"yes. But explain 5. Why should a password be spoken?" The answer to his
question: no password is spoken, before 13-44.2 or after it. The sentence
at `credentials.rs:102` says whose password Windows would not save, read
back or remove, and it named the account by its internal id, a UUID a
screen reader reads out a character at a time. 13-44.2 names the account
by the name it was given, or says "this account's password" where the code
holding the failure has only the id, and its cases hold that neither the
password nor the id is ever in the sentence. The six items are carried by
13-44.1 to 13-44.4, written that day and added on 2026-10-01. A seventh
question, found by 13-44.3's planner, he answered "yes." the same day;
decision 107 records it.

86. Item 1, ledger 677: Undo after Report as Junk also takes the junk mark
    off and puts the not-junk mark on at the server, through the gated flag
    write the report used, and then moves the messages back (13-44.1).
87. Item 2, ledger 691: a block is written to the account the selected
    message is in, found the way Report as Junk finds a message's account,
    not the account that is open; in All Inboxes the two differ (13-44.1).
88. Item 3, ledger 678: a rule's Mark as read, Mark as unread, Flag, Unflag
    and Add a label on arriving mail reach the mail server through the gate
    and the waiting queue the menu commands use, in the check that brought
    the message, and the next check keeps them. This reverses decision 37's
    second half (13-44.3).
89. Item 4, ledger 699: Look People Up at Work refuses a password for an
    address beginning `ldap://` when OK is pressed, with the sentence the
    lookup says, and saves nothing (13-44.2).
90. Item 5, ledger 696: a credential store failure names the account by its
    name, or says "this account's password"; never its id, and never the
    password, which no sentence has carried (13-44.2).
91. Item 6, ledger 711: a colleague Microsoft places is judged by the days
    and hours set in their own Outlook, read from the getSchedule answer
    already asked for; everybody else, the organiser included, by the
    working day set in Settings (13-44.4).

The planners' choices under that answer, each for him to overrule; an
overrule is a change to the named plan before it runs:

92. Undo of a report waits for one round trip to the server: the mark comes
    off first and the messages move back when the server answers, the
    reverse of the report's own order, so the two never reach the server in
    the wrong order, the race ledger 688 names (13-44.1, D3).
93. The server is asked about the mark only where the report may have left
    one, after the report's own answer said the mark was kept or failed part
    way. After a report on Gmail, on a folder that keeps no mark, or where
    the report never asked, Undo moves the messages back and sends nothing,
    so `$NotJunk` never goes on a message the report never marked (13-44.1,
    D4).
94. The Edit menu names the step "Undo Report as Junk" with the message; at
    the key the one word "Undo" is said, and once the messages are back one
    sentence: "Undid Report as Junk on Quarterly report. The server was told
    it is not junk.", or the server's reason when the mark could not be
    taken off. Redo reports the messages again, mark included (13-44.1, D5
    and D6).
95. A block on a message no account holds is refused in the report's words
    and writes nothing; a second Undo while the first is on its way is not
    blocked, because it finds the messages back and says so; the account's
    permission to change mail is asked at the key, before any worker starts
    (13-44.1, D7 to D9).
96. Past the literal item: any sign-in name for an `ldap://` address is
    refused at OK, even with the password box empty and nothing saved,
    because a sign-in over `ldap://` can never be used. Overruling this
    refuses only a typed or saved password (13-44.2, D2).
97. That refusal puts focus in the Directory address box, since the address
    is what has to change, and writes nothing; a password already saved
    stays until the sign-in name is cleared (13-44.2, D3 and D4).
98. "the password for Work", the name as the person gave it; a blank name,
    and the loads and the removal, which hold the id alone, say "this
    account's password"; the log keeps the id, which is never spoken. The
    "Security error:" in front stays, as 13-26 left it (13-44.2, D6 and D7).
99. The check that brought the message sends a rule's changes on the session
    it already holds, before the rule's move and before it reads flags back,
    not through the window's per-message worker, which could reorder a mark
    and a move (ledger 688) and would speak a line per message. Ledger 688
    itself, the runner's mark and move reaching the server in either order,
    is carried by no plan: a brief that said 13-42 fixes it was wrong, since
    13-42's text never names it, and his answer of 2026-09-29 did not
    include it (13-44.3, D2). 13-42's executor then fixed 688 from its brief
    on 2026-09-30, a run's marks riding on its waiting move and sent before
    it, and closed it in both halves; what still travels apart from a move
    is ledger 748. Written when these plans landed on 2026-10-01; 13-44.3's
    premise 7 says what that changes for it, which is nothing in its design.
100. With mail changes off nothing is sent: a rule's read or flag is kept
    here and waits, as a Mark as Read made with changes off does, and goes
    at the first check after changes are allowed; a rule's label comes off
    again, because the waiting queue holds read and flag only, and a queue
    for labels is ledgered. The overrule for the first half is to hold a
    mark back whole, as a move is held back (13-44.3, D3 and D4).
101. A check never writes the server's flags over a mark or flag waiting to
    go, which also stops a Mark as Read made with changes off from being
    undone by the next check; a message whose mark waits because the server
    could not be reached is left where it arrived for that check (13-44.3,
    D5 and D6).
102. Nothing is said per message: each outcome that is not a success is one
    clause with a count in the folder's line; `Outcome::touches_the_server`
    is renamed `moves_or_deletes`, because its old name was the old claim;
    POP is unchanged (13-44.3, D7 to D9).
103. Hours Microsoft sends in a shape this cannot read, or kept in a zone
    built by hand, give no week, and that colleague is judged by the working
    day set in Settings on their own clock, as every colleague is today.
    Only the guide says so; nothing new is spoken. The alternative is a
    spoken sentence like the one said for a guest nobody placed (13-44.4, D2
    and D3).
104. A colleague's week is judged on the clock its hours are written on, to
    the minute, and a day their answer does not list is not a working day.
    So a colleague's Saturday and Sunday now count as outside their working
    day: weekend times are still offered, after weekday times, with "outside
    Ada's working day" beside them, as Outlook does, and the changelog says
    so (13-44.4, D4 and D5).
105. The hours are read for the one search and stored nowhere, and the
    privacy page says they are read; no sentence changes its words (13-44.4,
    D6 and D7).
106. Found while planning and ledgered, not fixed: a report over several
    accounts keeps only the last account's move as the Undo step, and the
    recommendation is one step for the whole report (13-44.1). When the
    server refuses a label put on from the menu, the program says it was
    undone while the label stays in the database; and the check sends
    waiting moves before waiting marks, ledger 688's shape in the replay
    (13-44.3).
107. A question 13-44.3's planner found by reading, and Pratik's answer:
    a rule's Delete marks the message deleted on this computer only, and on
    a server that cannot report just what changed, the next check's flag
    read clears that mark, so the message comes back. Item 3 did not name
    Delete, so he was asked: "Should a rule's Delete go through the gated
    delete the menu's Delete uses, to Trash at the server?" His word, on
    2026-09-29: "yes." So a rule's Delete goes through the same gated
    delete path the menu's Delete uses: to the Trash, honouring Allow
    Changes, on the queue 13-44.3 already uses for the rule's other actions
    (13-44.3, D10 and task 3).

The planner's choices under that answer, each for him to overrule; an
overrule is a change to 13-44.3 before it runs:

108. The check uses the menu's own parts rather than the window's: where a
    deleted message goes, the change made here and kept in the store of
    waiting moves the menu's Delete waits in, the replay's own step to send
    it on the check's session, and the replay's reading of the answer. A
    refusal puts the message back where it arrived; a server that cannot be
    reached leaves it waiting in the Trash here for the next check, which
    sends it before reading any folder. "To Trash" is taken literally: a
    rule never deletes for good, and a message already in the Trash is left
    there, where the menu's Delete in the Trash deletes outright. An account
    whose Trash is not recognised deletes nothing and says the menu's
    sentence once. POP's rule Delete stays as it was, marked deleted on this
    computer, since no server brings it back (13-44.3, D11 to D13).

**Pratik's request of 2026-09-29**, in full: "That's fine. However there
should be a setting to delete mail from local deleted mailbox/trash for
pop/smtp accounts or other imap based accounts that are not Gmail or
Microsoft. It's my understanding that they control trash. A setting to
delete deleted mail upon the close of the app or 15 or 30 days should
suffice." None of GAP-01 to GAP-13 is it, so GAP-14 was added for it, and
13-44.6 and 13-44.7 carry it, written that day as 13-44.5 and 13-44.6,
renumbered on 2026-09-30 when 13-44.5 was put before them, and added on
2026-10-01. The brief that planned them took the defaults below for him,
each his to overrule; an overrule is a change to 13-44.6 or 13-44.7 before
it runs:

109. Four answers per account: Never, When Wixen Mail closes, After 15 days
    and After 30 days. Never is the default for every account, because
    emptying the Trash cannot be undone. 13-44.6 offers Never and the two
    day counts; 13-44.7 adds When Wixen Mail closes with the close that
    makes it work (13-44.6, D1).
110. Offered in the account editor, as "Empt&y the Trash (experimental):" on
    Alt+Y, for POP accounts and for IMAP accounts that are not Gmail or
    Microsoft by the one check (decision 119). A Gmail or Microsoft account
    gets a line in its place saying its provider empties the Trash itself:
    Gmail after 30 days, read from Google's help page 7401 on 2026-09-29;
    Outlook.com after about 30 days, which could not be confirmed that day
    because three Microsoft support addresses answered 404, so 13-44.6's
    task 2 reads Microsoft's own page first and leaves the number out, with
    a ledger `todo`, if it finds none; a work or school Microsoft 365
    account as its organisation set it (13-44.6, D2).
111. Days count from when a message went into the Trash on this computer,
    not from when it arrived. Nothing stored that, so it is added; a
    message already in a Trash when this build first opens the database
    counts from that moment, and the changelog says so (13-44.6, D3).
112. On an IMAP account emptying deletes at the server through the Allow
    Changes gate and the waiting queue the menu's Delete uses inside the
    Trash, sharing the step 13-44.3 wrote for a rule's Delete rather than
    building a second path. With mail changes off nothing is emptied and
    one sentence says why (13-44.6, D4).
113. On a POP account, on this computer only, never at the POP server, whose
    own removal setting decides. The brief said the stored row is removed;
    13-44.7's planner found it must stay, because it is what stops the next
    check downloading the message again and what the removal setting
    counts from, so the row is marked deleted instead, through the delete
    Empty Folder uses (13-44.7, D5). What the row keeps of the message's
    words is decision 132's.
114. When Wixen Mail closes is bounded, so closing never hangs, and nothing
    is lost (13-44.7, D6).
115. After 15 or 30 days is done at the account's first check of each day on
    this computer's clock. The brief said "during the regular mail check";
    once a day is the planner's, so a sentence is not said at every check
    and a refusal is said at most once a day. The overrule is every check
    (13-44.6, D7).
116. One sentence per account per emptying, such as "Emptied 12 messages
    from Trash in Work that had been there more than 30 days.", never one
    per message, shown on the status bar and said at Normal priority on a
    topic of its own; nothing said at close; the log keeps counts and the
    account's name, never a subject (13-44.6, D8).
117. The Trash is found the way Delete finds it, and an account with no
    recognised Trash is left alone, said once a day at the check and in
    the account editor. Everything that deletes for good is marked
    experimental where it is seen, in the label and in a description beside
    the other experimental ones (13-44.6, D9 and D10).

The planners' choices under that request, each for him to overrule; an
overrule is a change to the named plan before it runs:

118. The answer is kept in the settings as a map from account to answer,
    offered by the account editor, and a word this build does not know
    reads as Never (13-44.6, D11).
119. Whether an account is Gmail or Microsoft is the one check 13-44.5
    builds on Pratik's answer of 2026-09-30 (decision 131):
    `who_empties_the_trash` answers POP first, emptied here whatever its
    provider, then takes `WhoRunsTheMail`'s answer and reads no fact
    itself; the mail check and the close hand it `WhoRunsTheMail::of`, and
    the account editor `WhoRunsTheMail::from_what_is_known` with its
    address box, its IMAP server box and no recorded name, because OK
    writes the recorded name from the address alone. Until 2026-09-30 this
    decision was a fourth reading of the question, by the address, the
    recorded provider or the preset server, beside Report as Junk's, the
    folder chooser's and sign-in's, with a ledger `todo` naming the four
    and recommending one function all four ask; his answer that day
    accepted it, and the `todo` is not opened (13-44.6, D12).
120. When a message went into a Trash is recorded by four triggers in the
    mail database rather than by each writer, because the writers are
    several and a later one would forget (13-44.6, D13).
121. At most 500 a day, the oldest first; only messages stored here, so a
    Trash not kept up to date keeps the rest at the server; a message whose
    move is still waiting is left for a later day (13-44.6, D14 to D16).
122. On a server that cannot remove one message at a time the message is
    marked for removal and left, as the menu's Delete in the Trash leaves
    it, and counted as emptied because the replay answers done for both.
    Whether the replay should tell the two apart is a ledger `todo`
    (13-44.6, D17).
123. The account a POP message came from is recorded the first time it
    moves into a folder every account shares, and the check counts it as
    mail that account has had, which fixes decision 130's defect. Messages
    already in the shared Trash before this build belong to no account and
    are emptied only by Empty Folder; the overrule gives them an owner
    where exactly one account's folders hold the same message (13-44.7,
    D18).
124. Only a real close empties, never a hide to the tray. The window goes
    first, so a screen reader moves on; then five seconds for every account
    together; then the sign-off as before (13-44.7, D19).
125. At the five seconds the one delete under way waits in the queue and
    goes at the next start's first check, before any folder is listed; the
    rest stay in the Trash, untouched, for the next close (13-44.7, D20).
126. Nothing is said at close and the log records the counts; a POP account
    set to 15 or 30 days is emptied at the start of its check, before the
    POP server is dialled, and POP goes first at close; an account set to
    empty at close is not emptied at a check, nor the reverse (13-44.7, D21
    to D23).
127. A question the planner put to him, answered on 2026-09-30 (decision
    132): emptied POP mail kept its text in the database file, as every POP
    delete and Empty Folder do, and the question was whether emptying the
    Trash, which nothing can undo, should drop the text too and keep only
    what stops the message being downloaded again. The planner recommended
    yes, in a plan of its own, and he said yes; 13-44.8 carries it, and
    13-44.7 opens no ledger `todo` for it and writes no sentence saying the
    text stays (13-44.7, D24).
128. The experimental description says what emptying a POP account and
    emptying at close cannot do: POP mail leaves this computer's Trash and
    never the POP server, and a close empties what it can in a few seconds
    and the rest next time. Until 2026-09-30 it also said the text stays in
    the mail database, which decision 132 makes untrue one plan later
    (13-44.7, D25).
129. An account set to empty at close whose mail changes are off, whose Trash
    is not recognised, or, on POP, whose deleting here is off, hears why
    once a day at its first check, because nothing is said at close and
    silence would read as working (13-44.7, D26).
130. Found while planning and fixed in 13-44.7's second task: with "Leave
    mail on the server" on, its default, a POP message moved to the Trash
    is downloaded again by the next check, because the Trash every account
    shares is stored under a reserved id and the query of what an account
    has had reads only folders stored under the account. A probe on a copy
    of `main` at `90a4662e` printed `held {}` and `fetched 1`. A ledger
    `todo` opened when these plans landed, 753, carries it until
    then. He may want the fix sooner than the wave order puts it.

**Pratik's answer of 2026-09-30**, "Yes to both.", to two questions put to
him that day, each with its recommendation. Two plans carry them, written
that day and added on 2026-10-01: 13-44.5 before the Trash plans, because
they ask it, and 13-44.8 after them, because it changes what their emptying
keeps.

131. Item 1, the answer to decision 119's `todo`: one shared check says
    whether an account is Gmail or Microsoft, asked by Report as Junk, the
    folder chooser, both halves of sign-in and the Trash plans, so a Google
    Workspace or Microsoft 365 account on its own domain is recognised
    everywhere the same way (13-44.5, D1).
132. Item 2, the answer to decision 127: emptying a POP account's Trash
    drops the message's text from the mail database too, keeping only what
    stops the message being downloaded again, because emptying the Trash
    cannot be undone (13-44.8, D27).

The planners' choices under that answer, each for him to overrule; an
overrule is a change to the named plan before it runs:

133. The check reads three facts and the first that names Google or
    Microsoft decides: the incoming server (IMAP's for IMAP, POP's for
    POP), then the address, then the name recorded on the account. The
    server first because it is where the mail is and the one fact that
    says who runs a mailbox on its own domain; not the browser sign-in,
    whose provider is a copy of this answer (13-44.5, D2).
134. A server is Google's when its host is `gmail.com` or `googlemail.com`
    or ends at a dot in one of them, and Microsoft's the same way for
    `outlook.com` and `office365.com`, the domains every server the two
    providers' own pages name sits under. Nothing is looked up on the
    network, so a server name of an organisation's own that points at
    either is not recognised, and the pages say so (13-44.5, D3).
135. The outgoing server is not read, since mail can be sent through
    Gmail's or Microsoft's server for a mailbox kept elsewhere (13-44.5,
    D4).
136. The address is read through the six consumer domains the sign-in
    already knows and the recorded name ignoring case and space; where two
    facts disagree the earlier wins. A probe over 936 rows for each old
    reading found no answer lost except where another fact names the other
    provider (13-44.5, D5).
137. POP stays each caller's own first question, so a POP account on
    `pop.gmail.com` signs in through Google (13-44.5, D6).
138. Both halves of sign-in ask the check, so a token is filed and read
    back under one name. Two sentences change: the Account Manager's
    reason after "Signing in failed" names the server and the way out, and
    the mail check's names the account and the Account Manager in place of
    "no provider is recorded"; the NVDA case that waits for the first
    changes its words in the same commit (13-44.5, D7).
139. The folder chooser's own Gmail test goes, its rows joining the check's
    cases, and its line keeps its name so the record anchored after it
    stays (13-44.5, D8).
140. Pratik's answer of 2026-09-30, "yes to shared checks", to the
    question 13-44.5's planner put: whether the account editor's browser
    sign-in default, app password hint and Get App Password link, and the
    feedback report's list of account kinds, which go by the address or
    the recorded name, should ask the one check too. He took the
    recommendation. The app password hint and the Get App Password link
    ask the check once a server is typed and go by the address until then;
    the browser sign-in box's default stays with the address, so it never
    moves under somebody who chose; Send Feedback's list of account kinds
    asks the check. 13-44.5's task 3 carries it, and no `todo` is opened
    (13-44.5, D9). Until that day this decision was the question, carried
    by a ledger `todo` from 13-44.5.
141. A census reads every file under `src` and refuses a place that decides
    Gmail or Microsoft for itself, with a companion that plants each fault;
    13-44.6 and 13-44.7 add their callers to it (13-44.5, D10).
142. The words go wherever a message kept here alone is taken off this
    computer, because every such way is one arm of one delete: the Trash
    setting at a check and at close, Empty Folder on the Trash, Delete in
    the Trash, Delete Permanently, and a redo of either. The overrule is
    the setting and Empty Folder only (13-44.8, D28).
143. Every kind of message kept here alone, not only POP mail: copies of
    sent mail filed here and mail brought in from a file too. The overrule
    is POP mail only (13-44.8, D29).
144. What stays is the row under its own id, folder and number, marked
    deleted, holding its POP identifier, when it was downloaded, whose it
    was and whether this program filed it. It is done by removing the row,
    so the cascades take its text, headers, attachments where no other
    message carries the file, labels, identifiers and search entry, and
    writing it back with only those, so a column or table added later is
    covered without a list. No column is added or dropped (13-44.8, D30).
145. The removal runs with SQLite's secure delete switched on, so the space
    it frees is overwritten, and puts the setting back (13-44.8, D31).
146. The search index lets go of the words at the next check for mail, in
    short steps on the check's worker, because each pass rewrites the whole
    index: about 10 seconds in steps of at most 0.07 seconds at 200,000
    messages on the planner's machine. A record says the work is owed and
    is settled only when no removal happened meanwhile. The overrule is
    only after Empty Folder and the setting, or never, with the privacy
    page saying the words stay in the index (13-44.8, D32).
147. A rule's Delete on POP mail is unchanged: it marks the message deleted
    and keeps its words, because nobody took it off this computer and a
    rule can be written wrong. The overrule takes its words off too
    (13-44.8, D33).
148. Pratik's answer of 2026-09-30, "Yes to the measuring as well.", to
    the question 13-44.8's planner put: the privacy page says what taking
    a message off cannot reach, the copies earlier changes left in unused
    space in the file, the disk's own copies of deleted files, Windows
    Search's index, and replies that quote the message; should every
    write in the mail database overwrite what it frees, or should a
    command compact the database? It was asked with the recommendation to
    measure both first in a plan of their own, and he took it. 13-44.9
    measures both and opens the ledger entry that asks him to choose, and
    13-44.8 opens no `todo` for it (13-44.8, D34; 13-44.9, D36). Until
    that day this decision was the question, carried by a ledger `todo`
    from 13-44.8.
149. Nothing new is spoken or shown. Found and ledgered, not changed: Empty
    Folder's question says "there is no other copy" of a POP Trash whose
    mail may still be on the POP server under Leave mail on the server,
    which overstates what is lost (13-44.8, D35).

The planners' choices under his two answers of 2026-09-30, each for him to
overrule; an overrule is a change to the named plan before it runs:

150. An account the one check calls Microsoft is told to turn on the
    browser sign-in rather than to use an app password, under the address
    box, on the password box and by Get App Password, which opens no page
    for it; only Google's page is opened. Microsoft's own pages say no
    password reaches a Microsoft 365 mailbox over IMAP or POP, app
    passwords included, and give 2024-09-16 as the day the same stopped
    for Outlook.com. Without it, asking the check would tell a Microsoft
    365 account on its own domain, which gets no advice today, to use an
    app password and send it to the page for personal accounts. The
    overrule keeps Microsoft's advice and page as they are, asked by the
    check (13-44.5, D11).
151. The measurement is a module of the mail store compiled only for
    tests, because secure delete is a setting of the store's own private
    connection; nothing is added to the program (13-44.9, D37).
152. Two sizes, 12,872 and 200,000 messages, the two the measurements page
    already uses; in each, 1,000 messages written and changed the way a
    POP check writes and changes them, taken off through 13-44.8's path,
    and the rest written the way an IMAP check writes them (13-44.9, D38).
153. Secure delete measured at all three of SQLite's settings, off, fast
    and on, because SQLite's page says fast clears pages in use and leaves
    free pages. The overrule is off and on only (13-44.9, D39).
154. The compacting command measured two ways, VACUUM and incremental
    vacuum, each for its time, the temporary disk it needs, what it
    leaves, and whether a second connection's read and write wait or fail
    while it runs (13-44.9, D40).
155. The plan ends with one ledger `todo` for him with the figures, a
    recommendation reached by a rule written beforehand (what reaches every
    copy found; of those, what never makes other work wait past the store's
    five seconds; then what needs no temporary disk; then what costs
    least) and the choices; it builds neither and does not change the
    privacy page (13-44.9, D41).
156. The file reading stays in the suite pinning today's finding, so
    building either choice turns it red and whoever builds it turns it
    round (13-44.9, D42).

When these plans landed on 2026-10-01, the brief that landed them added one
more, carried by a plan already written:

157. Ledger 754, opened that day: `tests/a_marker_counts_at_the_start_of_any_line.rs`
    fails about one run in two on `main` with nothing changed, at step 8c
    (the line typed after **bold** and Enter comes out bold), found by
    13-39 when it stopped that plan's merge. 13-44.2 carries its fix as a
    task of its own, its cause found before anything changes and step 8c's
    assertions never loosened (13-44.2, D8 and premise 9).

When 13-44.6.1 was written on 2026-10-02, on Pratik's answer that day:

158. Pratik's answer of 2026-10-02, "yes.", to a durable fix for ledger
    716, so the tests that use the clipboard run whether or not the
    Windows session is locked, instead of failing with "OpenClipboard
    failed". Windows refuses the clipboard to every program while the
    session is locked, so a real paste can only be proven while it is
    unlocked; 13-44.6.1 keeps that one reading and makes everything else
    run either way (13-44.6.1, D1).

The planner's choices under that answer, each for him to overrule; an
overrule is a change to 13-44.6.1 before it runs:

159. The paste into the passphrase box stays a real paste: text put on a
    clipboard and the paste message sent to Windows' own password box.
    No program code is put between the box and a paste so that a test can
    hand it text, because the test would then prove that code instead of
    the box a password manager pastes into (13-44.6.1, D2).
160. The dialog's reading opens no clipboard and keeps running on every
    change to the main window; the real paste moves to a test target of
    its own, run when the passphrase dialog changes, at the phase's
    closing gate and on CI. While the session is locked it fails in about
    two seconds with a sentence saying the session is locked, never
    passing and never waiting for an unlock. The overrule is to have a
    locked run leave the real paste to CI, which is a check that skips
    itself (13-44.6.1, D3, D4, D6).
161. Two runs of the paste target at once take turns at the test
    clipboard that every process of one Windows logon shares, and a
    census keeps every other test target off the clipboard. The census
    runs with the dialog's reading, which most commits reach, rather
    than in the checks that read the whole tree, sparing the 29 guard
    records that would need measuring again (13-44.6.1, D7, D8).

When 13-44.6.2 and 13-44.6.3 were written on 2026-10-02, on Pratik's answer that day:

162. Pratik's answer of 2026-10-02, "yes to moving the plan as well as others.", to
    whether the tests that type keys get the same treatment as the clipboard test: they fail
    on a locked session and in the minute after an unlock while somebody types, which stopped
    the merges of 13-44 and 13-44.5 and, earlier, 13-39. He also accepted the clipboard plan's
    choice that a refused check fails fast with its cause named rather than skipping
    (13-44.6.2 and 13-44.6.3, D1).

The planner's choices under that answer, each for him to overrule; an overrule is a change to
13-44.6.2 or 13-44.6.3 before it runs:

163. No test keeps a reading of real keyboard input. None sends one today: every test that
    types posts or sends its own messages to its own controls, and real keys are the NVDA
    cases' to prove (13-44.6.2, D2).
164. Every test that sends a key or a click builds its windows on a desktop made for its
    run, on the window station it is already on. Nothing a person types reaches those
    windows, none is put in front of the person or read by the screen reader, and two runs at
    once share nothing. A new window station was measured and not used, because a posted
    Alt+letter pressed nothing there (13-44.6.2, D4).
165. The five that cannot move their window thread alone, because their window holds a
    browser or their reading starts a second copy of itself, run their window tests in a
    child process started on such a desktop, and each test passes only on its own line from
    the child (13-44.6.3, D2).
166. No test asks whether somebody is typing, since none sends keys where typing reaches; the
    session lock is asked only to name the cause when a child run fails. The overrule is to
    keep a target on the interactive desktop and ask both questions before it (13-44.6.2 D5,
    13-44.6.3 D5 and D6).
167. The four with a browser take the clipboard plan's turn, its mutex renamed for what it
    now guards, and the marker reading's own turn goes, so there is one turn, not two
    (13-44.6.3, D7; ledger 761).
168. A census beside the clipboard census refuses a key sender that builds no desktop for its
    run, and a key sender holding a browser without a child and the turn. It runs with the
    dialog's reading, which most commits reach; the overrule is a target of its own, reached
    at the phase's full gate and on CI (13-44.6.2 D7, 13-44.6.3 D8).
169. A count taken while nobody types cannot show this failure, since it needs a person at
    the keyboard: before, the ten plain-control targets were 20 of 20 each on the
    interactive desktop and the marker reading 3 of 9 with somebody at the machine. So both
    plans measure 20 runs and 20 rounds of two at once after, ledger the attended and locked
    runs as unrun rather than claim them, and close ledgers 580 and 761 as their causes
    removed, not as measured fixed (13-44.6.2 D9, 13-44.6.3 D9).

## Four things that wait on Pratik

Each is a checkpoint that stops only when the executor's brief does not
carry his answer, or a ledger entry no plan acts on.

| | What | Plan | If the answer is no |
|---|---|---|---|
| (a) | `cfb` 0.15.0 for reading `.msg` files (one lock entry, no `unsafe`, MIT with its licence shipped, read all four real samples) | 13-47, task 1 | Windows' own structured storage instead, and 13-47's tasks 2 to 4 are rewritten before they run |
| (b) | Three `windows` 0.62.2 features for printing, `Win32_Graphics_Gdi`, `Win32_Storage_Xps` and `Win32_UI_Controls_Dialogs`; and `rand` 0.8 as a renamed direct dependency, already locked through `pgp`, adding no package | 13-03, checkpoint; 13-20, task 1 | Printing declares every call by hand (13-03 has the fallback written); OpenPGP sending is not built, 13-21 offers S/MIME only and says so, and 13-20 still corrects the `cms` comment |
| (c) | Microsoft Graph's `People.Read` on both scope lists, and `Tasks.ReadWrite` on `THE_SCOPES_A_GRAPH_TOKEN_CARRIES`, which lacks it while the consent list has it (`oauth.rs:853` and `:99` on 2026-09-24). The sign-in screen asks everybody for more | 13-28, task 1 | Microsoft people search is not built; the same checkpoint offers him the planner's token-per-permission design to overrule |
| (d) | Filing the wxDragon printing defect upstream, a public post | 13-03 drafts the text in its summary and opens a ledger `todo` | Nothing: no plan posts it, and 13-51 checks it was not posted |

`GlobalLock` and `GlobalUnlock` need a fourth feature,
`Win32_System_Memory`, which the research's probe did not reach; 13-03
declares those two by hand, so (b) stays at three features.

**Also for Pratik, not blocking anything:** the `release.yml` quality gate
runs `cargo test` without `WIXEN_NO_AUDIO` (13-03 ledgers it and adds only
`WIXEN_NO_PDF_PRINTER` there, with the reason); PDFPurr 0.4.0 misreading
Microsoft Print to PDF files (13-03 ledgers it); whether classic Outlook
takes `.eml` files dragged into one of its folders (13-49 ledgers it); a
real `.msg` saved from his Outlook (13-48 ledgers it); whether to open an
issue about the saved-search 500 cap (13-39 ledgers it); decision 39's
reading of the 5,000 bound; and, once 13-44.9 has measured them, which way
of taking deleted text out of the mail database file he wants: secure
delete on every write, a command that compacts the database, both or
neither (13-44.9 ledgers it with its figures and a recommendation,
decisions 148 and 155). The last was added on 2026-10-01.

## Collisions, and how the waves settle them

From `RESEARCH-CROSS-CHECK.md`'s Collisions section, each with where it now
stands:

| Collision | Settled by |
|---|---|
| Edit menu: Undo Send moves from U to N | 13-01 alone, which rewrites `tests/undo_send_is_where_somebody_looks.rs` and its record and re-measures all three records naming that file |
| File menu: Print (P), the key manager (K), the two export items (F, X) | 13-03, 13-17, 13-45 and 13-46 in that order; each re-takes the letter grep after the plan before it lands. 13-03's accepted set is N, S, A, M, D, I, O, E, K, P, Q |
| Tools menu: the key manager and the Quick Step Manager both needing a letter, one free (Q), a blocker in the cross-check | Pratik's placements of 2026-09-24: the key manager on File (13-17) and the Quick Step Manager under Action, Quick Steps (13-41), so neither manager goes on Tools and its one free letter is left to nobody in this phase's plans |
| Action menu: J, Q and Z free | Report as Junk takes J (13-22), Quick Steps Q (13-41, 13-42), Z stays free; the next new Action item goes on a submenu. Run a Rule on This Folder is on the This Folder submenu, on L (13-44) |
| Keys: Ctrl+Shift+J, Ctrl+Shift+B, Ctrl+Shift+7 to 9, Alt+4 to 9 | 13-22, 13-25, 13-42 and 13-38; none was bound or documented on 2026-09-24, and each plan adds its keys to `tests/wired.rs`'s stated list where they are counted |
| The shared runner | 13-23 (the label fix), 13-24 (the do-halves), 13-24.1 (the runner) before 13-25 (the block), 13-42 (Quick Steps) and 13-44 (a rule over a folder), which call it. Since Pratik's answer of 2026-09-29, 13-44.3 sends a rule's marks, flags and labels on arriving mail through the same gate and waiting queue the menu commands use, and a rule's Delete through the menu's delete path and its store of waiting moves, on the check's own session rather than through the runner (decisions 99, 107 and 108) |
| The schema, `message_cache/mod.rs` | Seventeen plans in sequence, each additive with `CREATE TABLE IF NOT EXISTS` or `ensure_column_exists`: 13-09, 13-11, 13-12, 13-15, 13-16, 13-18, 13-19, 13-21, 13-33, 13-34, 13-36.1, 13-36.3, 13-37, 13-40, 13-42, 13-44.6, 13-44.8. All three were added on 2026-10-01, when this row said fourteen: 13-42 had added `read_first` and `starred_first` to `moves_waiting` with `ensure_column_exists` (`mod.rs:2348-2349`, ledger 688's fix) and its merge left this row as it was. 13-44.6 adds two tables, `in_the_trash_since` and `trash_last_emptied`, four triggers with `CREATE TRIGGER IF NOT EXISTS` that stamp a row entering a Trash folder and drop the stamp when it leaves or is deleted, and a first-run stamp with `INSERT OR IGNORE`. 13-44.8 adds one table, `search_index_owes_a_compaction`, and takes a message off this computer by removing its row and writing it back under the same id holding eight columns, a delete and an insert rather than a change to any table's shape; no column is added, dropped or renamed |
| The composer, the account editor and the item form | The composer by 13-06, 13-21, 13-35; the Account Manager by 13-06, 13-27, 13-33, 13-44.2 (the directory sign-in refused over `ldap://`), 13-44.5 (the browser sign-in asks the one check and says why it cannot sign in, and the app password advice asks it from a change handler on the IMAP and POP server boxes, which 13-44.6 calls from rather than binding its own), 13-44.6 and 13-44.7 (Empty the Trash on the connection page, on Alt+Y), all four added 2026-10-01; the item form by 13-06 and 13-32; in wave order |
| The credential store's 1,280-character limit | 13-16 splits keys into parts and makes the test store refuse what Windows refuses; a sign-in test it reddens is fixed there (decision 50); 13-26's directory password is short; the Microsoft token question is a ledger `todo` 13-16 opens |
| Item undo built before junk moves, blocks, Quick Steps and rule runs | Open. 13-42 reads 13-24.1's and 13-08's summaries and says whether a step's writes join Edit, Undo; 13-22, 13-25 and 13-44 do not ask, and 13-51's closing read says which of the four are undoable. Undo of a report also takes the junk mark off at the server where the report may have set one (13-44.1, decisions 86 and 92), and 13-51's read says so. 13-42 said on 2026-09-30 that a Quick Step is not one undo, Edit, Undo taking back its last write alone (ledger 747); written here when 13-44.1 to 13-44.9 landed on 2026-10-01, since its merge left this row as it was |

## What the tree contradicted in the research

Each row is carried in the named plan's premises, with the command that
read it on 2026-09-24 at `630e2a67`.

| Source | It says | The tree says | Plan |
|---|---|---|---|
| RESEARCH-CROSS-CHECK | a merged question list to take decisions from | the file has four sections (Numbering, Collisions, Premises checked, Packages) and no question list, so each range took its own research's list | all |
| RESEARCH-1, K2 | three `windows` features give printing | `GlobalLock` and `GlobalUnlock` live in `Win32_System_Memory`, a fourth; declared by hand | 13-03 |
| RESEARCH-1, K6b | whether a waiting move can be ended here is open | `application::moves_waiting::undo_here` already ends one, and `the_server_holds_it_at` records the new number after a server move | 13-08 |
| RESEARCH-1, K6c | an undone delete drops its deletion note | `deletions.rs` says no caller may drop one; 13-09 adds `take_a_deletion_back`, one transaction, only while unsent, refused while a sync sends it | 13-09 |
| RESEARCH-1, K2 | print the message the reader shows | the composition lives inside `open_in_the_text_reader`, which three readings in `tests/wired.rs` name; extracted, and the readings rewritten in place | 13-03, 13-04 |
| RESEARCH-2 | S/MIME calls through the `windows` crate | `signed_mail.rs` declares its calls in its own `#[link]` block, the file's stated convention | 13-14, 13-19 |
| RESEARCH-2 | Google's `iCalUID` and Graph's `iCalUId` assumed | read from each provider's own reference page; a Graph occurrence's UID is not its series' | 13-12 |
| RESEARCH-2 | answer buttons before the page in the formatted window | a guarded rule gives the browser the keyboard first, so the buttons sit after it | 13-11 |
| RESEARCH-2 | the backlog row at `:102` | `:104` | 13-13 |
| RESEARCH-2, R2-08 | the key manager on Tools; GAP-03's `[D]` line says Tools | Pratik moved it to File on K; 13-17 corrects the `[D]` line | 13-17 |
| RESEARCH-2 | fix the `cms` comment in 13-14 | any `Cargo.toml` change beyond the version line runs the whole gate, and 13-20 changes the manifest anyway | 13-20 |
| RESEARCH-3 | add both scopes to the shared token list | an older sign-in whose refresh lacks a new scope would stop four unrelated features; each new permission gets a token of its own | 13-28 |
| RESEARCH-4 | a stored order for saved searches | the folder tree shows runnable searches first, so a stored order alone would record a move the tree never shows; the order is carried into the tree | 13-37 |
| RESEARCH-4 | buttons reading Run and Don't Run | wxDragon 0.9.17's `MessageDialog` cannot relabel Yes and No; the question ends "Run it?" | 13-44 |
| RESEARCH-4 | the letter check reads each menu | `tests/wired.rs` finds a menu's letters by its variable name, so a rebuild function keeps the builder's name | 13-38, 13-42 |
| RESEARCH-4 | the key check accepts new keys | it accepts a counted key only through its `stated` list, which 13-38 and 13-42 extend | 13-38, 13-42 |
| RESEARCH-4 | the Quick Step Manager on Tools | Pratik settled it inside Action, Quick Steps | 13-41 |
| RESEARCH-5 | the first red test in `export_tree` | that module's header says it never touches a file or the database; the loop and the test go to a new `application::exporting_mail` | 13-45 |

## Costs every plan is written around

**Guard records, 1,088 by the TOML reader on 2026-09-24.** The files most
plans touch, with the records naming them in `tests_last_seen` and their
test count by the count check's own rule: `src/presentation/wx_app.rs` 119
and 199; `src/application/contacts_sync.rs` 76; `src/presentation/managers.rs`
54 and 137; `src/data/message_cache/messages.rs` 28 and 183;
`tests/house_style.rs` 29 and 74; `src/presentation/wx_settings.rs` 26;
`tests/wired.rs` 19 and 77; `src/presentation/wx_managers.rs` 14 and 44;
`src/data/message_cache/mod.rs` 12 and 23; `src/presentation/wx_compose.rs` 8
and 43; `src/presentation/ui_types.rs` 6 and 79. So no plan adds a test to
`wx_app.rs`, `contacts_sync.rs` or `managers.rs`; new readings go in new
integration targets or new modules at zero records, each with a record
whose `suite` names it. Every plan lists its records by both readings,
`tests_last_seen` and `file`, and the anchor text inside the regions it
edits, and quotes counts as the count at its start plus what it adds.

**The gate.** A branch commit runs what reaches the change; a merge made
with `git merge --no-ff` runs what the branch's whole diff earns; the whole
suite, the release build and the audit run once, in 13-51's task 3, by
hand, with `main` unmoved since its branch was cut. Three plans change
`Cargo.toml` (13-03, 13-20, 13-47), and each of those commits pays the
whole gate. Six change a workflow under `.github/` (13-03, 13-17, 13-27,
13-33, 13-39, 13-41), which answers `all` too.

**Carried from phase 12's README into every brief:** trace an absence claim
with a pattern that tolerates a line break; run the `--remeasure` remedy
whenever the count check prints it; never pipe `check.sh`; red trailers
name lib tests by module path and integration tests bare, on a branch,
never on `main`; `WIXEN_TEST_THREADS` untouched; a changelog entry in the
same commit as a user-visible change; every key in
`docs/KEYBOARD_SHORTCUTS.md` in the same commit; carriage returns measured
with `tr -cd '\r' | wc -c`; no em dash; none of the six words; **no
scripted rewrite of a tracked file, Read then Edit or Write**; commit
messages from a file with `-F` and no AI attribution, whatever the harness
says; the NVDA tests never run on this machine; the four completion marks.

**Three planners broke the scripted-edit rule while writing these plans**,
each once, each saying so in its return: a Python replace in a plan
(observation 0825), a Python line fix in 13-12, and four `sed` ranges fixed
by Python in 13-48. The other two ranges reported no break, and the
integration made its changes with Edit and Write only. Every plan and this
README read 0 carriage returns with `tr -cd '\r' | wc -c` when the plans
landed. Keep the sentence in every executor's brief.

## What only a person or a real provider can settle

Each requirement's last `[S]` line names it, and no plan claims it. The
tester's ear: every new menu item, key, letter, question and sentence, the
greyed Undo read as unavailable, an undo naming the item, the invitation
before the body and its buttons, the passphrase question, the key manager,
the Quick Step Manager, a rule's question and its count. A sighted reader:
a printed page. The runner: the Accessibility scan on each new window
(`pgp-keys`, the directory sign-in, `identities`, the step editor), the
spool target on CI with no PDF printer. His accounts: an invitation from a
real organiser and its answer; a message from a real correspondent's key; a
junk report reaching Gmail; a block moving mail at a real server; a real
directory; a real guest's free/busy; a shared mailbox; a `.msg` saved from
his Outlook. Everything a built window, a fixture, a cache built in the
test or a reading can prove, the plans prove.

## Estimates, and the factor behind them

`raw_tokens` is each planner's, 5,540,000 over the 53 plans. The `tokens`
field is not on one basis: 13-22 to 13-36 multiplied by 0.32, phase 12's
factor, and the other plans left `tokens` equal to `raw_tokens`. The factor
from phase 12's sixteen landed plans, each summary's `actuals.tokens` over
its plan's `raw_tokens` read on 2026-09-24, has a mean of **0.407** (0.116
for 12-12 to 1.183 for 12-05), so the phase projects at about 2,250,000
tokens (derived: 5,540,000 times 0.407). The fields were left as each
planner wrote them rather than rewritten across 53 files; read `raw_tokens`
and apply the factor here.

## What is owed to documents, and who does it

1. **The roadmap's phase 13 entry, plan list and progress row.** Done in
   the commit that lands these plans: the Plans line, a `- [ ]` line for
   each of the 53, the row at `0/53`, the phase line and the milestone
   paragraph brought forward.
2. **`.planning/REQUIREMENTS.md`.** Done in the same commit: a paragraph at
   the head of the phase's section and a planned line under each GAP
   requirement naming its plans, and each traceability row.
3. **`.planning/STATE.md`.** Done in the same commit, by hand: phase 13
   current, plan 0 of 53, 13-01 next, `progress.total_plans` 225 from
   `ls .planning/phases/*/*-PLAN.md | wc -l`, `completed_plans` 172 from
   the same over `*-SUMMARY.md`.
4. **Each plan's four marks**: the roadmap's row, its own line in the
   roadmap's plan list, `STATE.md`'s plan number in the frontmatter and the
   body, and its requirement's lines and traceability row. This README
   keeps no boxes (decision 51).
5. **The changelog, the pages and the ledger**, by each plan in its own
   commits; 13-51 reads them as one.

## Handover

Nothing in phase 13 has run. **The next plan is 13-01**, Undo and Redo on
the Edit menu, then the rest in wave order to 13-51 at wave 53.

What an executor's brief needs before its plan starts:

- **13-03** needs Pratik's answer to (b), the three `windows` features, or
  it stops at its checkpoint. **13-20** needs his answer on `rand` 0.8,
  also (b). **13-28** needs (c). **13-47** needs (a). No answer to any of
  the four is recorded in these plans; asking him all four before 13-03
  starts saves three stops later.
- Every brief says: no scripted edit of a tracked file; no AI attribution
  in any commit or pull request; the red and green on a branch; `main` is
  pushed only on his word; plans that change what is spoken or shown push
  their branch and open a pull request, and the others do not.
- 13-16's executor reads decision 50 before its second red commit.
- 13-42 and 13-44 read 13-24.1's summary for the runner's names as built;
  where the summary's name differs from the plan's, the summary's is the
  one every criterion follows.

`main` was pushed at `630e2a67`, so ledger 609's premise ("main has not
been pushed since 6cb8f17c") no longer holds; 13-51's premises name it to
close or reword. The commit that lands these plans is on `main` and not
pushed.
