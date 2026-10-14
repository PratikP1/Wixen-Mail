# Phase 13.2: The audit's fixes

Thirty-five plans, one per wave, written on 2026-10-14 against `main` at `dc592d07`, version
`1.0.0-alpha.1`, inserted between phase 13.1 and phase 14 the way GSD inserts a decimal phase,
while phase 14 stood at 8 of 13 plans with none of its sittings held. On the day,
`guards/guards.toml` held 1,488 records by the TOML reader
(`git show main:guards/guards.toml | py -c "import sys,tomllib;print(len(tomllib.loads(sys.stdin.read())['guard']))"`),
`.planning/WINDOWS.md`'s frontmatter said 830 entries, 704 open and 126 fixed, the last whole
gate was phase 13.1's, 10,566 tests at `0bc790d0`, and `git rev-list origin/main..main --count`
answered 50, with this clone's `origin/main` at `01ef4589`. The build Pratik has installed is
`1.0.0-alpha.1+1118.gd3455792`, made from 14-07's merge on 2026-10-06.

The directory is named the way `gsd-tools generate-slug "The audit's fixes"` answers,
`the-audit-s-fixes`.

**Goal.** The four regressions Pratik reported on 2026-10-14 are fixed first: Enter opens a
message that is alone in its conversation, a mark or a delete made here shows on its
conversation row at once, and the conversation window acts on the message under its cursor.
Then every finding of the full audit of 2026-10-14 is fixed as its verifier's verdict
corrects it, or settled by Pratik where a plan asks him: the 23 findings in the report he read
and the 30 it left out. No message is sent twice or left in Drafts after it goes; saving
accounts never deletes another account; Delete goes to the server's own Trash; every new
message reaches the rules once and is announced once; a stranger's message cannot crash the
program, fetch pictures through a reply or move the calendar while labelled Phishing; a
sign-in's token goes only to the provider and is kept only for the account that consented;
the calendar, contact, task and note syncs keep a change made while they push and see what the
provider deleted; announcements made together are joined rather than lost; settings survive a
crash mid-save; the gate and CI test what they say they test; an all-users uninstall says
what it leaves; the code nothing reaches is deleted; and the phase closes on its own full gate.

**Requirements.** No requirement is added and the coverage stays at 123. Thirty plans name
the requirements they work under, thirty requirements between them, and each adds a dated
line under its requirement, adding no `[D]` clause and ticking no box: FOUND-03, FOUND-04,
FOUND-11, FOUND-23, MAIL-01 to MAIL-05, LIST-03 to LIST-06, LIST-09, LIST-10, LIST-21, LIST-22,
LIST-25, GAP-04, GAP-06, GAP-14, FOLDER-01, SHIP-02, PIM-03 to PIM-06, SCALE-06, FEEDBACK-02 and
REAL-01. Five plans name none and each says why: 13.2-23 (no requirement is about how settings
are written), 13.2-27 (none covers the uninstaller), and 13.2-28, 13.2-29 and 13.2-31
(deletions of code nothing reaches, which no requirement describes). Those five leave three
completion marks, not four. REAL-01 stays phase 14's to tick: 13.2-15, 13.2-17 and 13.2-21 add
lines under it and leave its box open.

## Pratik's words of 2026-10-14

After the full audit of that day, Pratik said:

> "I want all of the items that need to be fixed as a phase including the ones not reported
> to me, the 28/29 items."

The audit is two files, written to the session scratchpad that day and copied into this folder
when the plans land, so a later session can read them:

- `ponytail-audit-2026-10-14.md`, the report he read: twenty items, which carry 23 of the 53
  findings, and the line "I left out 29 smaller findings: 21 should fix and 8 nice to have."
- `audit-2026-10-14-findings.json`, every verified finding with its verdict, key `all`, 53
  findings. A finding's number in every plan is its position in that array, read with
  Python's `json` module; the report's own items are written R1 to R20.

The 29 reconciles: 30 findings were left out of the report, 21 should and 9 nice by their
verdicts, and two of them, #8 and #53, name the same two functions. Every verdict says its
finding is real, and many correct the finding's own fix, so each plan works from the
verdict's corrected fix, re-read on `main` by command with file and line in its
`premise_corrections`, which also say where the audit was wrong. `GROUPING.md` in this folder
maps every finding to its plan, with the duplicates merged and the order explained.

## Pratik's four reports of 2026-10-14

The same day, testing build `1.0.0-alpha.1+1118.gd3455792`, Pratik reported four things he
meets every time he uses the mailbox. His words, as `REGRESSIONS.md` in this folder records
them:

1. "Pressing enter on a mail doesn't open a message any more. It appears that the fix to
   space/shift+space broke the message opening completely."
2. "Read/unread status change doesn't apply immediately. While the announcement says it does,
   the change only takes place when message sync occurs. This is confusing."
3. "Delete is the same as 2."
4. "When the thread is opened, user is unable to use context menu or other commands that can
   act on a message within the thread. Actions can only be taken on an entire conversation
   upon returning to the mailbox."

`REGRESSIONS.md` holds the investigation of each: what happens, the cause, the commit that
introduced it, and the red tests it proposes. The common thread is `478f5885`, which made a
folder never set show conversations. Four plans fix them, and they run before the audit's:
13.2-01 for the first report, 13.2-02 for the second and third, which share one cause, and
13.2-03 with 13.2-03.1 for the fourth, split in two because one executor could not carry both
halves. They went in front of the chain on 2026-10-14, and every audit plan moved up by three.

## The plans

"Pushed" is whether the plan's branch goes out with a pull request under Pratik's standing OK
of 2026-09-23, which a plan uses when it changes what is spoken or shown, so the NVDA job runs
before the merge. Every plan merges with `git merge --no-ff`, and none pushes `main`. Findings
are numbered as in the findings file; R numbers are the report's. A report is numbered as in
the section above.

| Plan | Wave | What it does | Findings | Requirement | Pushed |
|---|---|---|---|---|---|
| 13.2-01 | 1 | Enter opens a message that is alone in its conversation, in the view Pratik uses, and says why when there is nothing to open | none; report 1 | LIST-06 | yes |
| 13.2-02 | 2 | A mark, a star, a label, an undo or a delete made here shows on its conversation row at once, and the cursor lands after a conversation row leaves | none; reports 2 and 3 | LIST-21, LIST-04 | yes |
| 13.2-03 | 3 | The conversation window acts on the message under its cursor: the menu key, Delete, Shift+Delete and M, through the command code the list uses | none; report 4 | LIST-05 | yes |
| 13.2-03.1 | 4 | The conversation window gets the rest of the message keys, and marks a message read from it after the wait | none; report 4 | LIST-05, LIST-03 | yes |
| 13.2-04 | 5 | The Outbox claims a message before sending it, so two passes never send it twice and Undo Send never says "Taken back" about a message on its way | #31 (R1), #6 | FOUND-04 | yes |
| 13.2-05 | 6 | Sending a draft takes its filed copy out of Drafts, here and at the server, and Undo Send's too-late path files no copy of a message that went | #35 (R6) | FOUND-04 | yes |
| 13.2-06 | 7 | Saving accounts never deletes another account or resets the chosen order, a second account on one address is refused with a sentence, and a removed or switched-off account's inbox watch stops | #3 (R3), #33 | MAIL-04 | yes |
| 13.2-07 | 8 | Delete and the Trash schedule use the server's declared Trash, never a person's folder named Bin or Deleted, and Empty Folder says a folder was emptied only when the server emptied it | #9 (R2), #34 | GAP-14, FOLDER-01 | yes |
| 13.2-08 | 9 | Every new message reaches the rules once: all of a large arrival is fetched and sorted, and a message a rule files is one row in its folder, not two | #12 (R8), #10 (R7) | MAIL-02, MAIL-04, MAIL-05 | yes |
| 13.2-09 | 10 | The download of everything stops asking for text that always fails, and re-reads the open folder at most every few seconds without speaking each chunk | #11 (R9), #36 | MAIL-01, MAIL-03, MAIL-05 | yes |
| 13.2-10 | 11 | One check per account at a time, so new mail is announced once and the rules run once, and a check's flag pass reads the account once | #15 (R19), #14 | MAIL-04, MAIL-05 | yes |
| 13.2-11 | 12 | A change kept from offline is replayed only into the folder numbering it was kept under, and an older waiting flag change no longer overwrites a newer one | #5, #2 | LIST-21, LIST-22 | yes |
| 13.2-12 | 13 | A stranger's deeply nested HTML no longer overflows a thread's stack: depth is checked once where the markup is parsed, and anything deeper is read as its words alone | #17 (R4) | LIST-25, FOUND-03 | yes |
| 13.2-13 | 14 | Reply and Forward load none of the sender's pictures, and a meeting change in a message labelled Spam or worse is not applied | #19 (R5), #21 | LIST-09, LIST-10, GAP-04 | yes |
| 13.2-14 | 15 | A calendar feed that never answers no longer freezes the window or fills memory, and one unreadable POP message no longer ends that account's mail | #20, #13 | MAIL-04 | yes |
| 13.2-15 | 16 | A browser sign-in's token goes only to the provider's own servers, and is kept only when the account that consented is the one asked for | #18 (R18), #23 | GAP-06, REAL-01 | yes |
| 13.2-16 | 17 | What Wixen Mail saves from a message carries the Mark of the Web, and the update check trusts the project's signing profile rather than a name | #22, #24 | FOUND-11, SHIP-02 | yes |
| 13.2-17 | 18 | Google contacts keep syncing after the marker expires, and a contact made here goes to Google and Outlook on the next sync | #25, #26 (R11) | REAL-01 | no |
| 13.2-18 | 19 | Week and Month show events made here on their first day, and the Outlook calendar's window keeps moving and takes off what Outlook deleted inside it | #4 (R13), #27 (R12), #29's Outlook calendar quarter | PIM-06 | yes |
| 13.2-19 | 20 | A subscribed feed keeps a repeating event and its changed days apart, and the calendar server's XML is read as the text it carries | #28 (R14), #46 (R15) | PIM-03, PIM-05 | yes |
| 13.2-20 | 21 | A tick, a move or a note edit made while a sync is pushing is kept and sent, not wiped, and the notes editor's Save marks the note waiting | #1 (R17), tasks and notes | SCALE-06, PIM-04 | no |
| 13.2-20.1 | 22 | An edit to a calendar event made while a sync is pushing it is kept and sent, and an event deleted while its create is in flight stays deleted | #1 (R17), events | SCALE-06 | no |
| 13.2-21 | 23 | A full read of an address book or a Google calendar takes off what the provider deleted, the Google calendar reads in full only on an expired marker, and the contact syncs read the address book once | #29, #30 | REAL-01 | no |
| 13.2-22 | 24 | Announcements released together are joined into one line, most urgent first, so the first is never overwritten | #32 (R10) | MAIL-05 | yes |
| 13.2-23 | 25 | Settings are written beside the file, flushed and renamed over it, so a crash mid-save loses nothing | #7 | none | no |
| 13.2-24 | 26 | The release gate and the Linux and macOS jobs fetch the whole history, the guard reads every workflow, and the release gate holds a read-only token; a checkpoint asks Pratik about the sound setting | #39 (R16), #42 | FOUND-23 | no |
| 13.2-25 | 27 | The hook tests the commit it decides about, and a red commit whose test program crashed is refused | #40, #44 | FOUND-23 | no |
| 13.2-26 | 28 | The scoped run reaches the 73 guard records whose red tests live outside the broken file's module, and a `build.rs` change earns its tests | #38, #41 | FOUND-23 | no |
| 13.2-26.1 | 29 | A checkpoint asks Pratik whether a documents-only commit stops running clippy, and his answer is carried out | #45 | FOUND-23 | no |
| 13.2-27 | 30 | An all-users uninstall says the person's mail and settings may still be on this computer, and the comment says what the flag does | #43 | none | yes |
| 13.2-28 | 31 | The pretend accessibility tree is deleted, so start-up no longer says "Focus moved to Folders", and the log of spoken lines becomes test-only | #37, #47 (R20) | none | yes |
| 13.2-29 | 32 | The original skeleton's account manager, search engine and cache service are deleted, and `docs/architecture.md` is drawn from the real modules with a test that holds it there | #49 | none | no |
| 13.2-30 | 33 | The translation registry that translates nothing is deleted, and three dependencies leave the build or move to dev-dependencies, with a census that stops the next one | #48, #50 | FEEDBACK-02 | no |
| 13.2-31 | 34 | Functions nothing calls are deleted, each proved dead by grep: the ten the lean pass named, three IMAP helpers and the per-folder store reads | #51, #16, #8, #53 | none | no |
| 13.2-32 | 35 | vCard unfolding calls the calendar reader's instead of a copy; the closing read of all 53 findings and the four reports, the listening lines and the phase's full gate, `scripts/check.sh all` once by hand on its branch before its merge | #52 | FOUND-23 | only if the closing read changes the changelog |

## One plan per wave, and why this order

One per wave because every plan writes `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md` and
`.planning/STATE.md` for its completion marks, and nearly every plan writes
`guards/guards.toml`, `.planning/WINDOWS.md` and `docs/changelog.md`; a wave is a set of plans
that share no file, and these share those. Each plan depends on the one before it. Several
plans also edit the same source files in turn (`GROUPING.md`, "What a plan must read from a
sibling"), and each later one reads the earlier one's change and moves its guard anchors by
both readings.

- **13.2-01 to 13.2-03.1 first:** Pratik's four reports, which he meets every time he uses the
  mailbox, and the third of which phase 14's delete sitting hears.
- **13.2-04 to 13.2-11:** what phase 14's sittings prove, so the build they run on holds
  these fixes. Sending, accounts, the delete path, new mail, and the changes kept while offline.
- **13.2-12 to 13.2-16:** untrusted input and privacy.
- **13.2-17 to 13.2-21:** the contact, calendar, task and note syncs.
- **13.2-22 and 13.2-23:** announcements, then settings.
- **13.2-24 to 13.2-27:** the gate, CI and the installer.
- **13.2-28 to 13.2-31:** the deletions, last, so nothing deleted is a thing an earlier plan
  still needed.
- **13.2-32 last,** at the highest wave: the closing read and the phase's full gate.

Two plans were split off on 2026-10-14 at the plan check's request and keep the numbering by
decimals: 13.2-20.1 carries the events half of #1, and 13.2-26.1 carries #45, which changes a
rule `CLAUDE.md` records as Pratik's. A third, 13.2-03.1, was split from 13.2-03 the same day
because one executor could not carry both halves of the fourth report. So the waves run from 1
to 35 and match the plan numbers only for 13.2-01 to 13.2-03.

## What waits on Pratik

In the order the plans meet them. The questions in items 1, 3 and 4 can be answered before
their plans start. If he answers early, his words are written here, dated, and the plan or its
checkpoint quotes them rather than asking again.

1. **Before 13.2-02 and 13.2-03: the four questions at the end of `REGRESSIONS.md`.** 13.2-02
   is written on the recommended answer to question 1, that a conversation row brought up to
   date keeps its place until the folder is next read, and ledgers question 2, whether the
   folder tree's unread count changes at once, for his answer. 13.2-03 is written on the
   recommended answers to questions 3 and 4: after a command the conversation says only the
   command's own words, and deleting its last message lands him back in the list on the next
   conversation. Neither plan stops to ask, and he may overrule either.
2. **Any time: the identifier of his Artifact Signing certificate profile** (ledger 329). 13.2-16
   makes the update check trust that profile rather than the name "Pratik Patel", and ships
   the identifier empty, so the check refuses every installer until he gives it, which is
   already true because nothing is signed yet. The plan does not wait for it. Once a release
   ships with the identifier set, recreating the profile would leave every installed copy
   refusing updates, which the plan's premise explains from Microsoft's page.
3. **Before 13.2-24's task 3: should the release workflow's quality check set
   `WIXEN_NO_AUDIO`, as every other workflow does?** (ledger 616). The release check runs on a
   machine with no sound card, where the sound library crashes the test program and every test
   after the crash never runs. Tasks 1 and 2 fix the history and the token. On a no, the release
   check still stops at the sound crash. On a yes, whether it then passes is known only from his
   first dispatch of a release (ledger 326). Nothing is released by answering.
4. **Before 13.2-26.1's task 1: should a commit that changes only documents stop running clippy
   and the rebuild before it?** (finding #45). `CLAUDE.md` records today's rule as his, so
   nothing changes before he answers; on a yes the scripts and `CLAUDE.md` change, on a no
   nothing does.
5. **Things he may ask for now rather than later,** each ledgered by its plan rather than built:
   Microsoft's check of which account consented to a sign-in (13.2-15 builds Google's and sends
   the login hint to both); a sentence telling the person when a POP message nothing can read
   was kept as plain text (13.2-14 keeps it and writes a log line, as the IMAP path does); and a
   sentence when the settings file cannot be read for a reason other than a crash mid-save
   (13.2-23).
6. **Each plan's choices** under "Decisions the planners took" below, any of which he may
   overrule.
7. **After 13.2-32's merge: installing the new build** for phase 14's sittings (see the
   Handover), and then the sittings themselves.
8. **Pushing `main`** stays his word.

## The version

The version does not move in this phase. 13.2-04's premise 9 sets out the record, read at
`dc592d07`, and every plan follows it: `Cargo.toml` reads `version = "1.0.0-alpha.1"`, no tag of
any kind exists, and `CLAUDE.md`'s Versioning section says the prerelease counter moves on the
first behaviour change "after a build is cut". A tester build is not a cut: on 2026-09-17,
holding two builds of `1.0.0-alpha.1`, Pratik decided that the round's name stays and builds are
ordered by the counter after the `+`, and the same paragraph records the shape he declined, "a
release per tester build". 324 commits titled `feat` or `fix` have landed since that day with
tester builds installed between them, 14-08's the latest, and the version has not moved once.
So no plan edits `Cargo.toml` or `Cargo.lock` for the version. If Pratik answers otherwise later
and the answer is written here, it wins: the first green commit of the phase then moves the
counter with `Cargo.lock`, and that plan's summary records the two files as a deviation. 13.2-30
edits both files for another reason, the dependencies it removes.

## Decisions the planners took

Taken 2026-10-14 where a finding or its verdict left a choice open; Pratik may overrule any of
them. The full reasoning is in each plan's `decisions` block or premises.

- **13.2-01:** the store is read for one row when the loaded rows do not hold the row's message,
  rather than refusing; the refusal reads "That message is no longer on this computer, so it
  could not be opened."; the root fix is ledgered rather than done here, as `REGRESSIONS.md`
  decided.
- **13.2-02:** a row brought up to date keeps its place until the folder is next read; one
  narrowed read keyed by the messages a command touched, rather than a full re-read or a count
  kept in memory; the catch-up inside `take_row_out_of_the_list`, so every path that takes a
  row out is covered by one call.
- **13.2-03:** the conversation does not come back over a window a command opens, such as
  Reply or Forward, because a modal window over it would disable it; M here is the Action
  menu's command and says what that command says; the menu key stays in `wx_app.rs`.
- **13.2-03.1:** Print, Save As, the receipt and Block Sender are not added to the conversation
  window; a message read from a conversation is marked after the wait only while the cursor
  row in the list is still that conversation, the same rule the list keeps.
- **13.2-04:** a claim column on the Outbox row rather than a lock; claims let go at start-up
  only when this copy holds the running marker; "The Outbox is already being sent." for a pass
  that finds everything claimed.
- **13.2-05:** a remover over `KeepsTheDraft` rather than either wrapper; one lock across
  filings and removals rather than a second search after a delay.
- **13.2-06:** addresses compared ignoring case; the refusal in the dialog and in the store
  through one function; the Add window shown again with what was typed; the window's account
  list put in the moved order.
- **13.2-07:** each folder role decided over the whole listing, keeping INBOX by name; a
  name's precedence within its role's list when the server declares nothing.
- **13.2-08:** one fetch above the newest held message rather than batches; one settling
  function for both moves.
- **13.2-09:** the verdict's set of what this run asked over the audit's break; a text chunk
  counts as work only when it fetched something; ten seconds between re-reads; the reason
  carried on the loaded-messages update.
- **13.2-10:** one turn per account, the verdict's shape; no transaction around the flag pass.
- **13.2-11:** the flag push lets go by the time asked and the replay by the change sent; the
  flag queue gets the folder numbering now; one decision for all three replays.
- **13.2-12:** too-deep markup read as its words alone rather than shown raw, because the raw
  fallback would have a screen reader read every tag aloud; the limit chosen by measurement
  starting at 512; the parse's own cost timed and ledgered rather than fixed here.
- **13.2-13:** Spam or worse refuses a meeting change and Suspicious does not; the sender and
  the server's verdict travel as one value; ledger 637 narrowed and kept open.
- **13.2-14:** a 20 MB cap on a feed; the store read stays on the window's thread because the
  cache cannot cross threads; a kept POP message is said by its row and a log line, not a new
  count.
- **13.2-15:** the same check runs before the browser opens; an empty outgoing server is not
  refused; Microsoft's consenting-account check ledgered, as the verdict allows.
- **13.2-16:** the Mark of the Web also on a message saved as a file; its function in
  `attachment_name.rs`; the signing identifier shipped empty (item 2 above).
- **13.2-17:** one red and one green for both findings, so the 78 guard records naming
  `contacts_sync.rs` are re-measured once.
- **13.2-18:** a reading round runs out after 30 days; an Outlook removal notice outside the
  round's window keeps the row; #29's Outlook quarter closed here.
- **13.2-19:** a CDATA section comes back exactly as written, which neither the finding nor the
  verdict asked for.
- **13.2-20 and 13.2-20.1:** the identity and stamp writes stay unconditional and only the
  waiting flag depends on the row being unchanged; split in two at the plan check's request.
- **13.2-21:** a matched contact re-read by id before its fate is decided.
- **13.2-22:** the visual channel joined the same way as speech.
- **13.2-25:** the hook's pathspec widened to the build inputs the verdict's list missed.
- **13.2-27:** the uninstall-time call Inno forbids, found in the same function, fixed with it.
- **13.2-28:** the log of spoken lines changes type rather than taking `#[cfg(test)]` alone,
  since its event type goes with `automation.rs`; six guard records retired, the
  feedback_settings and signal records kept.
- **13.2-29:** `data/email_providers.rs` and `ui_types::AccountConfig` deleted with the
  skeleton, which the audit missed; a reading in `tests/docs_links.rs` ties the page to modules
  on disk.
- **13.2-30:** the two picture words, "Image" and "Figure", pinned by a test before the registry
  goes; a census in `service::outward` of what a release build carries.
- **13.2-31:** `is_connected` made test-only rather than deleted, because eight assertions use
  it; both unused update-download host constants deleted, since 13.2-16 adds no host check.
- **13.2-32:** the vCard unfolding moved onto the calendar reader's with no red, the 103
  contacts tests as the oracle and a re-measure of the calendar records as the proof. At the
  plan check's request, three vCard folds the calendar reader may not have met are first added
  as assertions inside an existing unfolding test and shown green against the old code, so the
  file's test count and every record's count stay where they are.

## Corrections made when the plans were brought together

Made on 2026-10-14 through Part A of `13.2-amendments.md`, written in the staging folder
because each planner could edit only its own files. Part A was first meant to be applied to the
copies in this folder when the plans landed. Later the same day, when the four regression plans
went in front and every audit plan was renumbered, Part A was applied in the staging folder
instead, to the staged files themselves, and `13.2-amendments.md` now says so where Part A
stood. Each item names the plans by their numbers after the renumbering:

- **One folder name.** Plans 13.2-28 to 13.2-32 had guessed a folder name of their own; the
  other thirty plans and this README use `13.2-the-audit-s-fixes`, the slug GSD's tool gives, so
  every `phase:` line, context path and summary path now names it.
- **One plan per wave.** Before the renumbering, 13.2-26.1 and 13.2-28 both claimed wave 25,
  and 13.2-27 and 13.2-29 both claimed wave 26, so plans 13.2-28 to 13.2-32 moved to waves 27
  to 31; their `depends_on` chain was already right. The renumbering then moved every audit
  plan's wave up by four, so those five sit at waves 31 to 35.
- **13.2-32 follows the splits.** Its closing read names 13.2-26.1 for #45, counts thirty-four
  summaries (twenty-eight before the splits, thirty before the regression plans), reads the
  gate stages 13.2-25, 13.2-26 and 13.2-26.1 changed, and expects three completion marks for
  the five plans that name no requirement. It also reads Pratik's four reports, one line each.
  Its first task shows three vCard folds green against the old code before the unfolding
  moves, and stops rather than guesses if the two readers differ on one of them.
- **13.2-18** names 13.2-20.1, not 13.2-20, as the later plan working in the calendar files.
- **`GROUPING.md`** carries rows for 13.2-20.1 and 13.2-26.1, #1 and #45 point at their plans,
  and its line saying plan 13.2-NN is wave NN is corrected by dating. It also carries rows for
  the four regression plans, which carry no finding.
- **`REGRESSIONS.md`'s version paragraph** said the first green commit moves the version to
  `1.0.0-alpha.2`; corrected by dating to "The version" above, and 13.2-04's premise that
  pointed at it says so.
- **The renumbering.** The four regression plans became 13.2-01 to 13.2-03.1, and 13.2-01 to
  13.2-29 became 13.2-04 to 13.2-32, with 13.2-17.1 becoming 13.2-20.1 and 13.2-23.1 becoming
  13.2-26.1. Each moved plan's `plan`, `wave` and `depends_on`, and every plan number in its
  prose, changed in the same pass, and 13.2-04 now depends on 13.2-03.1. Threat identifiers
  such as `T-13.2-01` count the threats inside one plan's register and name no plan, so they
  kept their numbers.

## What the tree said, read 2026-10-14

- `main` at `dc592d07`, the merge of 14-08. `git rev-list origin/main..main --count` answered
  50, the merges of phase 14's plans, none pushed.
- `git log main..14-10-sitting-2` names one commit, `0eea8aec`, which keeps 14-10's before
  reading as `14-10-BEFORE-READING.md` and sitting 2's sheet corrected against build 1118, so the
  tree could go back to `main` for this phase. Its message says the sitting has not been done.
- `src/presentation/wx_app.rs` is 39,696 lines on `main`; plans that edit it read it one
  function at a time and commit after each task.
- No guard record names a file under `.planning`, read with the TOML reader over all 1,488
  records, so the planning files this phase changes move no anchor.
- Ledger rows a plan closes or narrows: 464 (13.2-24, corrected and closed), 616 (13.2-24,
  left open for Pratik), 637 (13.2-13, narrowed), 325 (13.2-25 says whether its refusal settles
  it). Nearby rows the verdicts name as different problems stay as they are; `GROUPING.md`
  lists them. New rows are numbered by the executor at commit time, after 830.

## What only a person or a real provider can settle

- Every sentence the phase changes and nobody has heard: the pull requests run the NVDA job,
  which hears only what its cases wait for, and 13.2-32 writes a listening line for each.
- Whether the four reports are fixed as Pratik meets them, in the view he uses: 13.2-01 to
  13.2-03.1 each push a branch so the NVDA job runs, and only his own use of the next build
  settles it.
- What Gmail, Microsoft and the CalDAV and CardDAV servers do with the corrected syncs, the
  login hint and the Trash detection: each plan ledgers its real-account half for phase 14.
- Whether WebView2 honours the reply editor's picture policy on a real reply (13.2-13, ledgered).
- Whether the release check passes, which only his dispatch shows.

## Handover

13.2-01 is next, on a branch cut from `main` at the commit landing these plans. Nothing needs
Pratik before 13.2-24's task 3, apart from the answers above, which he may give any time.

Phase 14 resumes after 13.2-32's merge, at 14-10, on a new build that holds this phase. **No
plan makes that build yet.** 14-10's precondition reads the build named by "the latest summary
that made a build", so the build is made by a plan of its own, the way 14-08 made the last one
with `scripts/build-installer.sh` on Pratik's ruling of 2026-10-06, and Pratik installs it.
Then 14-10's executor works from branch `14-10-sitting-2`, whose before reading and sheet were
taken against build 1118, and corrects the sheet again against the new build before the
sitting.
