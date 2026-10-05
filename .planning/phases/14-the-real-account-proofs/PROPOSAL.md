# Phase 14: The real-account proofs. Proposal

Written 2026-10-04 against `main` at `01ef4589`, version `1.0.0-alpha.1`, from the three
research files in this folder:

- `RESEARCH-1-what-waits-on-a-real-account.md`, the ledger's open entries that only a real
  account, server or person can close;
- `RESEARCH-2-the-four-write-paths-and-22.md`, the cause of #22, the write paths and the gate;
- `RESEARCH-3-the-build-and-reading-a-proof.md`, the build Pratik needs and how a proof is
  read back afterwards.

Nothing outside this file was written. Every figure about Pratik's profile is a reading the
researchers took on 2026-10-04 with Python, read-only. The program has not run on his profile
since 2026-09-20, so those readings hold until he next starts it, and no longer. Commit counts
were taken with git at `01ef4589` and carry their command.

## In short

- **#22 has a cause, found without his help.** His Gmail account signs in with an app
  password, Google gives calendars, contacts and tasks only to a browser sign-in, and no build
  made here carries the Google sign-in client a browser sign-in needs. So the three syncs skip
  Google without a word and say "Calendar sync: 0 created, 0 updated, 0 deleted", which sounds
  like success.
- **Every proof needs a new build.** The one he has installed is 720 commits old and carries
  none of phases 12, 13 or 13.1.
- **Today's log cannot show the proofs the roadmap asks for.** The calendar, contacts and tasks
  syncs write nothing unless they fail, and a move or copy between two accounts writes nothing
  when it works. His log is also at Info, not Debug.
- **Proposed shape:** seven plans of code and documents that need only his answers, then one
  build, then four sittings of his, then a closing plan. His part is four sittings of about 3
  to 4 hours together, about half an hour of answers, installing and reading the gate's
  wording, and up to an hour in Google's console if he has no sign-in client yet. These are
  estimates, derived in "Pratik's part" below.
- **Twelve questions.** The first two decide whether #22 can be proved at all and how.

## What the research changed

1. **The cause of #22.** Both research 1 and research 2 read it from the profile, the installed
   program and the code. His account row has `use_oauth` 0. There is no `oauth.toml` in his
   settings folder, no `WIXEN_` variable anywhere, and the installed program holds no Google
   client id (research 2 searched its bytes). With no client, `credentials_for("gmail")`
   answers nothing and the Calendar, Contacts and Tasks syncs skip their Google half silently.
   His own log of 2026-08-23 already said it: "No sign-in credentials are set up for gmail, so
   signing in through the browser cannot run."
2. **Three smaller defects on the same path.** The sync picks providers by which sign-in clients
   the build holds, not by what the account is. Held events are counted twice in the Google and
   Microsoft arms. And only Google's main calendar is ever read, while REAL-01 says "its
   calendars".
3. **The log cannot settle #22 at any level.** No line is written for a request, an answer or a
   skipped provider. The roadmap's premise that "the profile's log at the moment of the refresh
   is what the fix reads first" does not hold: there is nothing there to read.
4. **His log level is Info.** A profile that already holds a level keeps it, so the Debug lines
   added in 11-04 are not written on his machine.
5. **His profile already holds real-server evidence nobody has recorded.** Five deletes replayed
   against Gmail on 2026-09-20 ("A waiting delete was replayed: Moved to Trash"). Gmail refused
   118 sign-ins with "Too many simultaneous connections" on 2026-09-19. Gmail dropped the
   connection about 940 times during the download of everything over 18 to 20 September. Two of
   his deletes from 2026-09-20 still wait in the queue and go to Gmail at the next start.
6. **Two warnings are false today.** Every surface says sending has never been tried, and it was
   proven on 2026-09-18. The download-everything warning says it "has never been run against a
   real account", and it has. A third sentence, the first-run screen's "Changes to tasks,
   contacts and the calendar go up to your provider", is untrue for a Gmail account that signs
   in with a password, and for every build that holds no Google client, which is every build
   made so far.
7. **The gate has one switch for every change to mail.** `Allowed::mail` covers sending,
   moving, copying, deleting, marks, rules, filing in Sent, read receipts, subscriptions and
   emptying the Trash. "The default moves per path" is possible only by splitting it.
8. **Tests write into his real crash log.** `test_an_address_that_is_not_a_page_opens_no_window`
   appends three lines to `%LOCALAPPDATA%\wixen-mail\logs\crash.log` on every run: 4,545 lines
   since 2026-09-22, beside one real panic from 2026-09-18 that does not name its build.
9. **The record on #63 says "OAuth" for the sending proof.** The account row and every log line
   of 18 to 20 September say app password.

## Scope

### What phase 14 proves: the requirements' own lines

| Line | Requirement | Ledger entries it settles |
|---|---|---|
| Gmail's calendars, contacts and tasks arrive when the account is added and on Refresh | REAL-01, #22 | none exist for #22 itself; the new entries the plans open for the silent skip, the double count and the main calendar |
| Copy within one account | REAL-02, #63 | 546 (copy half) |
| Copy across two accounts | REAL-02, #63 | 547, 187 |
| Move within one account | REAL-02, #63 | 546 |
| Move across two accounts | REAL-02, #63 | 547, 187, 191 (rewritten: the question it describes was retired by 11-07.2) |
| Delete | REAL-02, #63 | 376, 546 (delete half), 771, 784, 787 |
| Sending | REAL-02, already proven 2026-09-18 | 148, which #63 says closes and the ledger still holds open |
| The gate's default and the four warning surfaces | REAL-02 | 377 when the sentences are rewritten |

### What rides along in the same sittings: recommended in

Each needs only his one Gmail account and a few minutes inside a sitting that is already
happening, or no hands at all.

| Item | Ledger | Where | His extra time |
|---|---|---|---|
| Undo after the server heard, read back in Gmail | 625 | Sittings 2 and 3 | 5 minutes |
| Undo of a mark or star at the server | 623 | Sitting 2 | 3 minutes |
| One event, one contact with all five name parts and one task moved between lists, each made here and read in Google's own app | 154, 597, 218; 203 and 216 seen in passing | Sitting 1 | 15 minutes |
| Google behaviours the code was read for and never run, recorded when they show themselves | 194, 434, 435, 671, 727, 732 | Sitting 1 | none |
| The details of a crossing: a byte-for-byte arrival, a dropped connection mid-append | 183, 188 | Sitting 4 | 10 minutes, 188 optional |
| What Gmail did to the download and the connection limit, written onto the entries from his logs | 11, 64, 65, 67, 72, 523, 525 | 14-07, no hands | none |

The calendar, contacts and tasks switch already starts on for every new tester, and nothing it
allows has ever reached Google. The fifteen minutes in Sitting 1 are its first proof.

### What waits for a later phase: recommended out

REAL-01 and REAL-02 name none of these, and each needs something he has not said he has.

| Group (research 1's numbering) | Needs | Entries, roughly |
|---|---|---|
| 7. Microsoft: Outlook.com, Microsoft 365, OneNote, To Do | A Microsoft app registration, a Microsoft account | 14 |
| 8. Calendar and address book servers (CalDAV, CardDAV) | A Fastmail, iCloud, Nextcloud or similar account | 20 |
| 9. Invitations and time zones | An organiser who is not him, or a second account of his on another provider | 15 |
| 10. S/MIME and PGP with a real correspondent | A certificate or key pair and somebody using another mail program | 12 |
| 11. Junk, block, rules and Quick Steps at a server | His Gmail account, plus a non-Gmail IMAP server for the keyword halves | 10 |
| 12 and 13. Directory, people search, free/busy | A work directory, a Workspace or Microsoft 365 colleague | 9 |
| 14. Other addresses, shared mailboxes, delegation | Gmail's "Send mail as", a shared mailbox granted by somebody | 3 |
| 5, in part. Gmail threads against Gmail's own view, All Mail, a 100,000-message mailbox's speed, STARTTLS, CONDSTORE | His eye on Gmail's web reader; a harness run against his profile; another provider | 14 |
| 15. What real mail looks like, counted from his database | His agreement only; no hands | 8 |
| 1, in part. Send Later hours later, alt text surviving to Gmail, a report reaching support@ and security@ | His hands; the support addresses exist by public testing | 3 |

The counts are research 1's, for sizing only: an entry can sit in two groups. Open entries that
need no account, only his ear or his files, stay where the ledger already has them and are not
in this table.

Group 11 matters to the gate. By research 2's list the mail switch also covers rules and marks
on arriving mail, so the switch's starting position cannot honestly move for those until they
are proven at a real server.

### Not phase 14 by their own words

Imports and exports, ledgers 499, 509, 789, 792, 795, 796, 799, 801, 805 and 806, each say
"Nothing here touches a server, so nothing goes to phase 14". They need his files and his ear,
not an account.

**Recommendation:** phase 14 is REAL-01, REAL-02 and the ride-along items above. Everything in
"waits for a later phase" goes to a phase grouped once he says which accounts and people he can
use (question 11).

## The plans, in order

One plan per wave, because every plan writes `guards/guards.toml` or `.planning/WINDOWS.md`
and most write `docs/changelog.md`, and two plans that share a file cannot share a wave.

All code lands before the build, because every sitting needs the new build and the build
should carry the log lines the proofs are read from. The roadmap's order holds for the
sittings: #22 first, then copy, move and delete within one account, then across two accounts,
since a copy is the least destructive. If his Google client is not ready when the build is,
Sittings 2 and 3 can go first; the build carries everything they need.

"Pushed" is whether the plan's branch goes out with a pull request under his standing OK of
2026-09-23, for plans that change what is spoken or shown.

| Plan | Wave | What | Requirement | Size | Needs from Pratik | His time | Pushed |
|---|---|---|---|---|---|---|---|
| 14-01 | 1 | Calendar, Contacts and Tasks say why nothing came | REAL-01 | S to M | nothing | none | yes |
| 14-02 | 2 | Each request and Google's answer in the log; the provider follows the account; a sync when an account is added | REAL-01 | M | nothing | none | yes |
| 14-03 | 3 | The Google sign-in for calendars, contacts and tasks | REAL-01 | M for route B, S for route A | questions 1 and 2 | none beyond the answer | yes |
| 14-04 | 4 | Every calendar on the account, not only the main one | REAL-01 | M | question 3 | none | yes |
| 14-05 | 5 | The write paths say what they did | REAL-02 | M | nothing | none | no |
| 14-06 | 6 | The proof reader, and tests out of his crash log | REAL-01, REAL-02 | M | nothing | none | no |
| 14-07 | 7 | True words now, and the records put straight | REAL-02 | S to M | questions 4 and 8 | none beyond the answer | yes |
| 14-08 | 8 | The sitting sheets and the build | REAL-01, REAL-02 | S to M | install it; place `oauth.toml` | 10 to 15 minutes, plus Google setup | no |
| 14-09 | 9 | Sitting 1: first start and #22 | REAL-01 | checkpoint | his hands, questions 5 and 9 | 40 to 60 minutes | no |
| 14-10 | 10 | Sitting 2: copy and move within Gmail | REAL-02 | checkpoint | his hands | 45 to 60 minutes | no |
| 14-11 | 11 | Sitting 3: delete within Gmail | REAL-02 | checkpoint | his hands, his Gmail IMAP setting | 25 to 35 minutes | no |
| 14-12 | 12 | Sitting 4: two accounts, Trash emptying and POP | REAL-02 | checkpoint | his hands, question 6 | 60 to 80 minutes, plus 2 minutes some days later | no |
| 14-13 | 13 | The gate, the warnings, the closing read and the phase's full gate | REAL-01, REAL-02 | M | question 7, his word on the wording | about 10 minutes | yes |

When a sitting finds a defect, a fix plan is inserted after it (14-10.1 and so on): red and
green, a new build, and that line taken again. Each costs him an install (5 to 10 minutes) and
10 to 20 minutes per line taken again.

### 14-01: Calendar, Contacts and Tasks say why nothing came

- **Task 1, the tracer.** A pure answer to "may this account ask Google, and if not, why",
  built from the account's provider, how it signs in, whether this build holds a Google client,
  and whether a browser sign-in has expired. The Calendar sync asks it first, says the reason in
  place of "0 created", and writes one Info line naming the reason. One path, end to end through
  the Calendar module, red first.
- **Task 2.** Contacts and Tasks ask the same answer. Held events are counted once.
- A sentence the program might say, as a draft for the executor's own words: "Calendar: nothing
  was asked of Google. This account signs in with a password, and Google gives calendars only to
  a browser sign-in."

### 14-02: Each request and Google's answer in the log, the provider by account, and a sync when an account is added

- One Info line per request a pass makes: the provider, the method, the path without query
  values, the HTTP status, and Google's reason when it refuses. The counts at the end of every
  pass, whether or not they are spoken. Held by the reading in
  `tests/the_log_carries_what_a_report_needs.rs`, which already refuses any log call naming a
  subject, body, password or token, and shown against the loopback stand-ins first, as REAL-01's
  `[D]` line asks.
- The provider asked follows the account (`provider_of`), not which clients the build holds.
- The three syncs run once when an account is added, as REAL-01's `[D]` line asks, saying at
  most one sentence per module.

### 14-03: The Google sign-in for calendars, contacts and tasks

- **Route B (recommended, question 2):** a browser sign-in in the account editor that asks
  Google for the calendar, contacts and tasks permissions only, not mail, stored per account the
  way 13-28 gives each new permission a token of its own. Its letter is chosen after a grep of
  the letters the account editor already uses. 14-01's answer learns the new state, so an
  expired sign-in is said, not hidden.
- **Route A:** no code. The sheet says which box to tick and that Google asks again every week.
- Either route: the sheet tells Pratik how to place `oauth.toml` himself, from the tracked
  template `oauth.toml.example`, so no agent handles the secret.

### 14-04: Every calendar on the account

Google's calendar list is read and each calendar is filed under its own row, shown against the
loopback stand-in. Dropped if his answer to question 3 is that the main calendar is enough.

### 14-05: The write paths say what they did

- The three replay lines ("A waiting move was replayed", and the copy and delete forms) carry the
  message's row number and the server's answer, its new number when it gives one. Never a
  subject. Today two copies in one minute cannot be told apart in the log.
- A move or copy between two accounts writes a line for the fetch, the append and the removal,
  and its outcome. Today a success writes nothing.
- "Signed in to" says whether the account used a password or a browser sign-in.
- The crash entry names the build, `1.0.0-alpha.1+N.gHASH`, not only `1.0.0-alpha.1`.

### 14-06: The proof reader, and tests out of his crash log

- `scripts/read-a-proof.py`, Python standard library only. It opens the database read-only
  (`mode=ro`, never the program's own store, which writes on opening), reads every daily log a
  UTC window touches, and prints: the build, the log level, the waiting queue, counts for the
  folders a sheet names, calendars by provider, events, contacts, tasks and sync state, and the
  replay, wait, refusal, sync and warning lines with addresses and quoted text masked. It prints
  no subject except one beginning "Wixen proof".
- Held by a Rust integration target over a fixture profile built with the real schema in a
  temporary folder, which includes a subject and an address the reader must not print. Placed on
  `scripts/check.sh`'s lists and given a guard record, so the gate can place it.
- The page window test stops writing into the real profile, and a ledger entry records that it
  did.

### 14-07: True words now, and the records put straight

On his answers to questions 4 and 8:

- Sending's sentences on the settings screen, the first-run screen, the end of `--help`, the
  alpha page, and the undo-at-the-server and Trash-emptying warnings where they say sending never
  ran. The tests that pin the old words go red first.
- The download-everything warning says what Gmail did on 18 to 20 September.
- The first-run sentence about changes going up to the provider says a Google account needs the
  browser sign-in for that.
- The shortcuts page's line saying a move to another account waits for both servers, false since
  11-07.2, corrected by dating.
- Ledger 148 closed on #63; 191 rewritten to the resume; Gmail's behaviour written onto 11, 64,
  65, 67, 72, 523 and 525; an entry for each finding above that has none.
- A comment on #63 correcting "OAuth" to the app password, posted on his word.

### 14-08: The sitting sheets and the build

- One sheet per sitting, kept in this folder. Each step names the menu item, the key and the
  sentence he should hear, read from the tree at the merge, with the listening items of
  question 10 at the moments they fit.
- The build, made with `scripts/build-installer.sh` on a clean tree at 14-07's merge.
- A checkpoint: he installs it, and places `oauth.toml` if question 1's answer is yes.

### 14-09 to 14-12: the four sittings

Each sitting plan has the same three steps: the reader's snapshot before; his sitting, a
checkpoint; the reader after, the record drafted, and posted on #22 or #63 on his word.

| Sitting | What he does | What it also reads |
|---|---|---|
| 1 | Starts the new build and lets the first check run. Signs in through the browser by the chosen route. Sync Calendar, Sync Contacts, Sync Tasks, then Refresh in each module. Makes one event, one contact with five name parts and one task moved between lists, and reads each in Google's own app. Quits with Ctrl+Q. | The two old deletes replaying (or held, question 5); whether the new build still meets Gmail's connection limit and the unreadable Trash of 18 and 20 September. If the Trash is still unreadable, a fix comes before Sitting 3. |
| 2 | Copy within Gmail: online; with the network off and then back; with the network off, a quit and a restart. Move: the same three, and one more where he changes the message in Gmail on the web or his phone while Wixen Mail is offline. Undo after the server heard; undo of a mark. F5 on both folders, then Ctrl+Q. | The replay line, the waiting queue empty, the message under a real server number in its new folder, the original still in place for a copy. |
| 3 | Delete online; with the network off, a quit and a restart; Delete Permanently on a proof message, after reading his Gmail setting for what an expunge does; one account's Allow Changes box unticked and a delete refused (376); undo of a delete, read back in Gmail's Trash. | What Gmail kept against what he heard. By the secondary sources research 2 read, Gmail's default turns an expunge into an archive, which the program would describe as removed; a sentence that is untrue on that default is a ledger finding, not a failed proof. |
| 4 | Adds the second account (and the others, question 6). Copy across and move across, online and with the network off and a restart. A message of about 10 MB; the same message copied into Gmail twice; one over 25 MB. Emptying the Trash at the non-Gmail IMAP account. A POP message taken off this computer, and a look some days later. | Ledger 187's three questions, 547, 771, 784 and 787. |

## Pratik's part, and how long it takes

Every time below is an estimate, not a measurement. Research 2's own estimates were marked low
confidence, and these are longer than its 30 minutes a session because each variant means
finding a proof message, acting, switching the network, quitting and starting again with a
screen reader.

| Step | What he does | Likely time |
|---|---|---|
| Answer the questions below | One reply | 15 to 20 minutes |
| A Google sign-in client, if he has none | Create a Google Cloud project, a consent screen in Testing with himself as a tester, a desktop client, the Calendar, People and Tasks interfaces turned on, and the two values in `oauth.toml` | 30 to 60 minutes; 5 minutes if he already has one |
| Install the build | Run the setup file and accept the Windows prompt | 5 to 10 minutes |
| Sitting 1 | First start and #22 | 40 to 60 minutes |
| Sitting 2 | Copy and move within Gmail | 45 to 60 minutes |
| Sitting 3 | Delete within Gmail | 25 to 35 minutes |
| Sitting 4 | Two accounts, Trash emptying, POP | 60 to 80 minutes, plus 2 minutes some days later |
| The gate | Read the drafted sentences, say yes per path | About 10 minutes |
| Listening items, if question 10 is yes | 2 to 5 per sitting | 5 to 10 minutes per sitting |

Derived from the rows above: the four sittings come to about 170 to 237 minutes, roughly 3 to 4
hours; the answers, the install and the gate add 30 to 40 minutes; Google setup adds up to an
hour; the listening items add 20 to 40 minutes if they ride along. Sittings 2 and 3 can be one
longer sitting if he prefers fewer.

## How a sitting works

- **Proof messages only, never his own mail.** He sends himself one message per step, titled
  "Wixen proof copy 1", "Wixen proof move 2" and so on, and uses a folder named "Wixen proof".
  Sending is proven, so this costs nothing new. On Gmail, deleting a copy removes every label,
  the original's included, so proof messages are the only ones touched.
- **A line is proven when four readings agree** (research 2's rule): what he heard, in his
  words; the replay line in the log with no refusal and no put-back; a fresh listing from the
  server, taken with F5 on the source and the destination folder before Ctrl+Q; and the
  database afterwards, with no waiting row for the proof message, its row in the destination
  under a real server number, and for a copy the original still in place. Where a ledger asks
  for it (625, the delete line's Trash), he also reads the message in Gmail itself. A line where
  any reading disagrees is recorded as what happened, not as proven.
- **The record** is one comment per line on #63, or on #22 for REAL-01, in the shape of the
  sending comment: the date, the build, the account and how it signs in, what was done, his
  words, the reader's lines. Drafted by the agent, posted on his word.
- **Nothing runs on this machine during a sitting.** Window tests take focus from NVDA, and until
  14-06 lands the library tests write into his crash log.
- **No agent can work while the network is off.** Turning the network off takes this machine
  offline, and an agent session on it stops too. He works the sheet alone and tells the agent
  when he is done; the reading happens afterwards.
- **Gmail's limit on connections.** A second mail program reading the same account while Wixen
  Mail runs can meet "Too many simultaneous connections" and look like a failure of the path
  under test. Gmail on the web or on his phone is the other client for the "changed meanwhile"
  step. A sitting that meets the limit says so and is taken again.
- **Google's weekly sign-in.** While his Google project is in Testing, a browser sign-in lasts
  seven days. Under route B only calendars, contacts and tasks are affected. A failure on day
  eight is Google's rule, not a defect.

## The listening pass in the same sittings

His listening pass stays where it is unless he says otherwise. If question 10's answer is yes,
each sheet carries only the items from `docs/manual-accessibility-pass.md` that walk the same
screen at the same moment, and nothing else from the pass is added.

| Sitting | Items of the listening page | From the alpha page's "what would help most" list |
|---|---|---|
| 1 | 64 (Tab from the folder tree into the list), 92 (a synced contact's five name parts), 106 (undo of a contact, event and task, its Google half) | none |
| 2 | 65 (growing and shrinking a selection), 66 (a move within one account), 104 (undo of a move and a copy, before and after the server heard) | 8, 14, 20 |
| 3 | 104 (its delete half) | 8, 20 |
| 4 | 67 (a move to another account), 172 (Trash emptying after days, on the non-Gmail IMAP account), 174, 175 and 182 (the POP items) | 8 |

Items 89 (a report reaching support@ and security@), 110 and 153 (invitations and Workspace
free time) and 162 (Send mail as) stay out: the addresses are live by public testing, and the
rest need a person or setup phase 14 does not include.

## The build

- **What:** made on this machine with `scripts/build-installer.sh` on a clean tree at 14-07's
  merge commit, so no `.dirty` is stamped. It will be named `1.0.0-alpha.1+N.gHASH` with N a
  little above 1,071 (`git rev-list --count 01ff57bf..HEAD` answered 1071 at `01ef4589`).
  Nothing needs pushing for it. CI's Setup Executable job builds the same file, but only after a
  push of `main`, which is his word each time.
- **When:** after the last code plan merges and before Sitting 1. Again only if a sitting finds a
  defect.
- **How it installs:** over his machine-wide copy in `C:\Program Files\Wixen Mail\`, after the
  Windows prompt. Inno Setup's `UsePreviousPrivileges` defaults to the existing install's mode
  (jrsoftware.org, read by research 3 on 2026-10-04), and every file the setup installs carries
  `ignoreversion` (`grep -n '^Source:' installer/Wixen-Mail-Setup.iss` at `01ef4589` answers
  eight lines, each with it), so a later fix build replaces it too. His settings and mail stay
  where they are.
- **Its number:** the build counter has passed the 999 the Windows file version holds, so Apps
  and Features shows `1.0.0.14999` for every build until the version moves. About, `--version`
  and the log's first line still carry the true number, and the reader prints it. No bump is
  owed: `git tag` lists nothing, so no build has been cut.
- **How far it moves him:** the installed build is `1.0.0-alpha.1+351.g5f363255`, 720 commits
  behind `01ef4589` (`git rev-list --count 5f363255..HEAD`), 97 of them merges by
  `git rev-list --merges --count 5f363255..HEAD`, where research 3 gave 104.
- **Before it starts:** two deletes from 2026-09-20 replay at its first check (question 5).

## Decisions the planner took, for Pratik to overrule

- **New log lines at Info.** His profile is at Info and keeps it, so lines at Info need nothing
  from him. Research 1 suggested asking him to move to Debug; Debug stays his choice.
- **The proof reader lives in the tree with a test,** rather than as a scratch script typed into
  each plan (research 2's shape). It is used at least eight times in this phase, once before and
  once after each sitting, and its job of keeping his mail out of the output deserves a test.
- **The sitting sheets live in this folder, not under `docs/`.** Everything under `docs/` ships
  inside the installer, and these are one phase's steps.
- **One build, and a fix build only when a sitting finds a defect.**
- **Route B's sign-in asks for calendars, contacts and tasks only,** not mail, following 13-28's
  token per permission.
- **When an account is added, each module says at most one sentence** about its first sync.
- **Sitting 0 of research 2 and research 3 is folded into Sitting 1**: the first start is the
  first thing Sitting 1 does.

## Where the research files disagreed, and what this proposal takes

| Point | Research says | Taken here |
|---|---|---|
| Log level | 1: set Debug for the phase. 2: set Debug once. 3: write the new lines at Info | Info |
| The reader | 2: a scratch script carried in the plan. 3: a tested script in the tree | In the tree, tested |
| Session length | 2: two sessions of about 30 minutes. 3: four or five sittings | Four sittings, times re-estimated above |
| Item 172 | 3: with the Gmail delete. 1: ledger 770 says a Gmail account gets a sentence in place of emptying | Sitting 4, on the non-Gmail IMAP account; the sheet reads from the build whether Gmail offers emptying at all |
| Holding the two deletes with `--read-only` | 3: possible. 3's own assumption A3: whether `--read-only` reaches the replay is not traced | Let them go (question 5) |
| Merges behind | 3: 104 | 97 by the command above |

## Questions only Pratik can answer

Each has a recommendation and the honest cost of taking it. Combinations are welcome.

1. **A Google sign-in client.** To bring your calendars, contacts and tasks, Wixen Mail needs a
   sign-in client from a Google Cloud project with you on its tester list. No build here has
   one, which is why #22 brings nothing. Do you have one, or will you make one? And should it
   live only on your machine or go into every build?
   **Recommended:** make one, or use yours, keep it in Testing with yourself as a tester, and
   put its two values in `oauth.toml` in your own settings folder yourself, so no agent handles
   them. Your machine only, for this phase. **Cost:** 30 to 60 minutes in Google's console if
   you have none; other testers' builds keep saying why Google calendars cannot come. Google's
   page (developers.google.com/identity/protocols/oauth2, read 2026-10-04) ties the seven-day
   sign-in to the Testing status and does not say what an unverified project in production
   gets, so nothing here relies on that. If the answer is
   no, REAL-01 stays open and phase 14 proves only that the program says why nothing came.
2. **How your Gmail account signs in for calendars, contacts and tasks.** (A) Move the whole
   account to the browser sign-in: no new code, but mail stops every week until you sign in
   again, and mail through the browser sign-in has never been tried against Gmail. (B) Keep mail
   on the app password, which is how sending was proven, and add a separate browser sign-in for
   calendars, contacts and tasks only.
   **Recommended:** B. It matches the alpha page, which already calls the app password the
   steadier choice for mail. **Cost:** one plan of new code, one more control in the account
   editor, and two ways of signing in for a tester to understand.
3. **All your Google calendars, or only the main one?** Today only the main one is ever read.
   **Recommended:** all of them, since REAL-01 says "its calendars". **Cost:** one more plan
   before the build.
4. **The sending proof of 18 September.** Your account and the log say it went with the app
   password; the comment on #63 says the browser sign-in. Which was it?
   **Recommended:** unless you remember a browser sign-in, a comment on #63 corrects it.
5. **The two deletes waiting since 20 September.** They go to Gmail at the new build's first
   check, before anything else. Let them go, or hold them?
   **Recommended:** let them go. They are your own deletes, and one was probably already done at
   Gmail, which tests the "already done" answer against a real server for the first time.
   **Cost:** they happen before any planned step. Holding them by starting with `--read-only`
   has not been traced and may not hold them.
6. **Other accounts.** Copy and move across two accounts need a second account. Emptying the
   Trash at a real server needs an IMAP account that is not Gmail or Microsoft, because the
   program leaves their Trash to them. The POP lines need a POP account. Which can you add?
   **Recommended:** a second Gmail account for the two crossings, which is free and quick; any
   other IMAP account you already have for the Trash line; and a POP account for the POP lines.
   **Cost:** each line without an account is recorded as still open, not dropped.
7. **The safety switch for changes to mail.** One switch covers sending, moving, copying,
   deleting, marks, rules, filing in Sent, read receipts, subscriptions and emptying the Trash,
   and it starts
   off for new testers. Phase 14 proves five of those. Options: (a) change only the warning
   sentences, path by path as each is proven, and leave the switch's starting position as it
   is; (b) split the switch, for example sending apart from the rest, or one box per path;
   (c) start it on once all five are proven, with the rest riding along unproven.
   **Recommended:** (a) in this phase, and (b) decided after it with its cost in front of you.
   Not (c). **Cost of (a):** new testers still start with changes to mail off. Your own settings
   already allow everything, so nothing changes for you. **Cost of (b):** the settings screen,
   the first-run screen, the account editor, `--allow` and every place the program reads the
   switch, and more boxes for a screen reader user to read past.
8. **Three sentences that are false today.** Every screen says sending has never been tried;
   the download-everything warning says it has never run against a real account; the first-run
   screen says changes to tasks, contacts and the calendar go up to your provider, which cannot
   happen for a Gmail account on an app password or on any build made so far. Correct all three
   now, before the build?
   **Recommended:** yes, each saying what happened and when, so the build you install says true
   things.
9. **Reading your profile, and posting the records.** May the agent read your settings, log and
   mail database before and after each sitting, with Python, read-only, printing counts, the
   build's name and log lines with addresses and quoted text masked, never a message body and
   no subject except those beginning "Wixen proof"? And may your one-line confirmation of a
   sitting's results be the word to post them on #22 and #63?
   **Recommended:** yes to both.
10. **Listening checks in the same sittings.** Each sheet can carry the two to five listening
    items that walk the same screens at the same moment, listed above, and nothing else.
    **Recommended:** yes, for those only. **Cost:** 5 to 10 minutes a sitting.
11. **What phase 14 covers, and in what order.** Microsoft, calendar and address book servers,
    invitations from a real organiser, encrypted mail with a real correspondent, a work
    directory, free time with colleagues, shared mailboxes and other sending addresses: phase 14
    or later? And the order: #22 first, then copy, move and delete within Gmail, then across two
    accounts?
    **Recommended:** later, in a phase grouped once you say which accounts and people you can
    use; phase 14 keeps REAL-01, REAL-02 and the Gmail items the same sittings touch. Confirm the
    order, with Sittings 2 and 3 moving first if the Google client is not ready.
12. **The build.** Made on this machine from the last code plan's merge, or downloaded from CI
    after a push of `main`? Keep `1.0.0-alpha.1`, or move to `1.0.0-alpha.2`?
    **Recommended:** made here, no push needed, and keep `1.0.0-alpha.1`, since no build has
    been cut. **Cost:** Windows' Apps and Features shows `1.0.0.14999` for every phase-14
    build; About and the log tell them apart. `alpha.2` resets that number but renames the
    round you chose on 2026-09-17.

## Requirements and criteria

| Requirement or criterion | Plans |
|---|---|
| REAL-01, criterion 1 (#22) | 14-01, 14-02, 14-03, 14-04, 14-06, 14-08, 14-09, 14-13 |
| REAL-02, criterion 2 (#63): the five lines, after a restart and with the network off | 14-05, 14-06, 14-08, 14-10, 14-11, 14-12 |
| Criterion 2: ledger 187's three questions and 547 answered or said still open | 14-12 |
| Criterion 2: the gate's default moves per path on his word | 14-07 (sending's words), 14-13 |
| The phase's full gate, `scripts/check.sh all` once by hand before the closing merge | 14-13 |

## Handover

Nothing in phase 14 has run. The next step is Pratik's answers; with them the planner writes
the thirteen plans, and 14-01 starts. Nothing needs his time until the build in 14-08 is ready
to install, apart from the Google client if question 1's answer is to make one, which he can do
any time before Sitting 1.
