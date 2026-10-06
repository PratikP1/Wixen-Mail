# Phase 14: The real-account proofs - Research, part 3: the build and reading a proof

**Researched:** 2026-10-04, against `main` at `01ef4589`, version `1.0.0-alpha.1`,
`git rev-list origin/main..main --count` answering 0.
**Domain:** the build Pratik installs for the proofs, the tooling that reads a
proof back from his profile afterwards, and how the proofs share his sittings
with the manual accessibility pass.
**Confidence:** HIGH for what is installed and what the tree and his profile
hold (read this session); MEDIUM for the proposed session shape (a design,
not a measurement).

How the profile was read: with Python only, read-only (`sqlite3` with
`mode=ro`, `json.load`, plain file reads, `winreg`), counts and line shapes
only. No message subject, body, address beyond a server's domain, token or
password was printed. The installed program was not started.

## Summary

**The build Pratik has installed is 720 commits old.** Apps and Features and
the registry say `1.0.0-alpha.1+351.g5f363255`, installed 2026-09-20 under
`C:\Program Files\Wixen Mail\`, built from 11-11.1.3's closing commit. It
predates all of phase 12, phase 13 and phase 13.1 (104 merges). Every proof
under REAL-02 has to be re-taken on a new build anyway (#63's comments of
2026-09-19), and the roadmap says the steps are written against the build that
carries phase 12, so phase 14 needs a fresh build before any proof. The build
script works here today: Inno Setup is installed per user, and CI's Setup
Executable job also builds the same file on every push of `main`.

**The build counter has passed the file version's cap.** HEAD is 1,071
commits past `01ff57bf`, the commit that set `1.0.0-alpha.1`, and
`scripts/build-installer.sh` holds the counter to 999 in the Windows file
version. Every build from now on reads `1.0.0.14999` in Apps and Features
until the version moves; the version string still carries the true number,
and the script says so on the console. It still installs over `+351` cleanly
(14999 is above 14351). Whether to move to `alpha.2` is Pratik's call.

**Reading a proof afterwards works partly, and three gaps decide whether it
works without Pratik describing what happened.** What is there: the daily log
at Info records every server-side move, copy and delete the queue sends ("A
waiting move was replayed: ..."), every sentence handed to the screen reader
("Speaking: ..."), and every refusal and warning; the database shows the
queue (`moves_waiting`, `move_in_flight`, `outbox_queue`) and every message's
folder and server number. What is missing: (1) the PIM sync writes nothing at
all unless something fails, so success criterion 1's "the log says which
request the sync made and what Google answered" cannot be met by today's
code; (2) the replay lines carry no message number and no command path, so a
proof is matched to its log line by time alone; (3) `crash.log` in his real
profile holds 4,545 lines written by one unit test since 2026-09-22 and one
real panic, so it cannot be read as evidence. There is no script in the tree
that reads a profile; every reading so far was typed by hand into a session.

**Primary recommendation:** before the build, land three small plans test
first (the proof reader script, the log lines the proofs need, the crash log
fix), then build once, then run the proofs in four or five sittings, each a
sheet that interleaves the proof's steps with the manual-pass items that walk
the same screens, with the agent reading his profile before and after each
sitting so Pratik only reports what he heard.

## User Constraints

No `14-CONTEXT.md` exists; phase 14 has not been discussed. The constraints
in force are the brief's and the roadmap's:

- **The brief's hard rules (this research and any plan built on it):** never
  start the installed or release program; never sign in to any account; never
  enter or read a password or token; never send mail; never touch NVDA.
  Pratik's profile is read only through Python, read-only, never through bash
  or PowerShell (a bash view of AppData shows a stale copy); never print
  message bodies, subjects, addresses beyond the domain, tokens or passwords;
  counts and log lines about requests and errors are fine.
- **Roadmap, phase 14 entry:** the order inside the phase is the planner's,
  for Pratik to confirm; proposed order #22 first, then #63's lines copy,
  move, delete. `application::allowed`'s default moves per proven path, and
  the four warning surfaces follow, on Pratik's word.
- **REQUIREMENTS.md, REAL-01 [D]:** "The profile's log at the moment of a
  Refresh read first, with his agreement". REAL-02 [D]: "A steps page per path
  against the build that carries phase 12, each run by him and recorded on the
  issue with the date and the build".
- **Memory, 2026-09-24:** manual testing is not surfaced as pending and "not
  heard yet" lines are not relayed to Pratik. This research proposes combining
  sittings only because the brief asks for it.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REAL-01 | Adding a Gmail account brings its calendars, contacts and tasks, and Refresh in each module brings what the account has. | The profile today: 0 contacts, 0 events, 0 tasks, 0 `sync_state` rows, 2 local calendars; the account signs in with an app password (`use_oauth` 0); no PIM sync line in any log. The PIM sync logs only on failure, so the "or the log says which request" clause of criterion 1 needs new Info lines (Plan B). |
| REAL-02 | The five write paths are each proved against a real account and recorded, and the gate's default moves per path on Pratik's word. | Every server-side write goes through `replay_one` and logs one Info line; two deletes have waited in his queue since 2026-09-20 and replay at the new build's first check; manual items 66, 67 and 104 are the same walks as the proofs. The proof reader (Plan A) and the replay lines (Plan B) make each proof readable from the profile. |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Building the tester's setup file | Build tooling (`scripts/build-installer.sh`, CI Setup Executable job) | Installer (`installer/Wixen-Mail-Setup.iss`) | The script stamps the version, the counter and the file version; Inno only consumes them. |
| Saying what a write or a sync did | Application layer (`moves_waiting`, the PIM syncs) writing through `tracing` | Presentation (`Speaking:` lines from the accessibility layer) | The layer that knows the answer is the one that logs it; the log file is the record. |
| Reading a proof afterwards | Developer tooling outside the product (a Python script under `scripts/`) | none | Must not open the database through `MessageCache::new`, which writes; must run where bash cannot see the profile. |
| Recording a proof | GitHub issues #22 and #63, `docs/manual-accessibility-pass.md`, `.planning/WINDOWS.md` | the phase's summaries | REAL-02 [D] names the issue as the record. |

## What Pratik has installed

All read 2026-10-04 with Python, read-only.

| Fact | Value | Source |
|---|---|---|
| Uninstall key | `HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall\{9C2E6B41-0F7D-4A83-B5E1-27D4A9F3C608}_is1` | `[VERIFIED: winreg read]` |
| DisplayVersion | `1.0.0-alpha.1+351.g5f363255` | `[VERIFIED: winreg read]` |
| InstallDate | `20260920` | `[VERIFIED: winreg read]` |
| InstallLocation | `C:\Program Files\Wixen Mail\` (a machine-wide install) | `[VERIFIED: winreg read]` |
| `wixen-mail.exe` | 42,803,200 bytes, written 2026-09-20 08:56; FileVersion resource `1.0.0.0` | `[VERIFIED: os.stat, GetFileVersionInfoW]` |
| Its commit | `5f363255`, 2026-09-20 08:50 -0400, "docs(11-11.1.3): 11-11.1.3 complete, ..." | `[VERIFIED: git log]` |
| Distance to HEAD | 720 commits, 104 of them merges | `[VERIFIED: git rev-list]` |
| Builds he ran, from his logs | `+149.g744d05ef` (09-18), `+232.g6911018d` and `+266.g39537d13` (09-19), `+321.g4a09bfc2`, `+g59c5b6a4` and `+351.g5f363255` (09-20) | `[VERIFIED: "Starting Wixen Mail v..." lines]` |
| Last day the program ran | 2026-09-20; no daily log after `wixen-mail.2026-09-20.log` | `[VERIFIED: directory listing]` |
| A copy of that setup file | `dist/Wixen-Mail-Setup-1.0.0-alpha.1+351.g5f363255.exe`, the only file in `dist/` | `[VERIFIED: ls]` |

The exe's own file version reading `1.0.0.0` is expected: CLAUDE.md says
`winresource` stamps that resource from the crate version and nothing hands
it the build; only the setup file carries the counter.

## What his profile holds, as it bears on the proofs

`%LOCALAPPDATA%\wixen-mail`, read 2026-10-04.

| Item | Value |
|---|---|
| `cache\message_cache.db` | 775,663,616 bytes; `-wal` 16,051,552 bytes from 2026-09-20 16:57 |
| Accounts | 1: provider `Gmail`, protocol `imap`, `imap.gmail.com` / `smtp.gmail.com`, `use_oauth` 0, enabled, `allow_deleting_here` 1 |
| Messages | 99,014 rows, 50 folders (Inbox 1, Sent 4, Trash 2, Spam 2, Drafts 2, Archive 1, Outbox 1, Custom 37) |
| PIM tables | `calendars` 2 (both `source_provider` `local`), `calendar_events` 0, `contacts` 0, `tasks` 0, `task_lists` 2, `notes` 0, `sync_state` 0 |
| Queue | `moves_waiting` 2 rows, both `kind` `delete_to_trash`, asked 2026-09-20T20:56Z; `move_in_flight` 0; `outbox_queue` 0 |
| `config\app_config.json` | `log_level` `info`; `allowed_changes` mail true, personal_information true, reading true; `allowed_per_account` empty; `which_updates` `not_looking` |
| `logs\` | daily `wixen-mail.YYYY-MM-DD.log` files, the last three 1.2 MB, 2.8 MB, 1.2 MB; `crash.log` 460,735 bytes, written today; no `feedback\` folder, so no report was ever sent from Help, Send Feedback |

Sources: `[VERIFIED: sqlite3 mode=ro, json.load and os.walk over the profile, 2026-10-04]`.
The `accounts` table has a `password` column; it was not read.

Five things this changes for the plan:

1. **Two deletes will replay at the new build's first check.** The 09-20 log
   shows the same queue replaying five deletes that day ("A waiting delete was
   replayed: Moved to Trash"); two were still waiting when the program closed
   at 20:57Z. With mail allowed in Settings, the first check after the new
   build starts sends them. That is, unplanned, the "replayed after a restart"
   case of REAL-02's delete line, fourteen days late and by a different
   build. It is either the first piece of evidence or something to hold with
   `--read-only` on the first start; that is Pratik's choice (question 3).
2. **His Gmail account signs in with an app password.** `use_oauth` is 0, and
   no log line from 09-18 to 09-20 contains "OAuth" (the redirect server logs
   "OAuth redirect server listening on ..." at Info, `src/service/oauth.rs:612`).
   The Google tasks client takes an OAuth token on every call
   (`src/application/tasks_sync.rs:475-489`). Whoever researches REAL-01's
   cause should start there `[ASSUMED: that an app-password account has no
   token for the Google PIM APIs and that this is #22's cause; not traced
   here]`. Note also that #63's comment of 2026-09-18 calls the sending proof
   "(OAuth, SMTP through the outbox ...)", which the profile contradicts or
   predates (question 4).
3. **His log level is Info, not Debug.** The alpha page says a fresh profile
   starts at Debug under an alpha, and a profile that already holds a level
   keeps it (`docs/ALPHA_TESTING.md`, "What the log writes"). Every log line
   the proofs need should therefore be at Info, or Pratik moves the level by
   hand (question 6).
4. **No PIM sync has written a line.** Across every daily log, no line holds
   "Contacts sync", "Calendar sync", "Task sync" or "sync error"; the 09-15
   and 09-17 files are 0 bytes. The reading REAL-01 [D] asks for first has
   nothing to read until a build that logs the sync is run.
5. **Log time is UTC, crash time is local.** Log lines begin with an RFC 3339
   UTC stamp (`2026-09-20T03:54:43.758533Z`), the daily file rolls at UTC
   midnight, which is 20:00 in Pratik's evening, and `crash.log` stamps local
   time (`src/main.rs:457-461`). A sitting in his evening spans two files.

## The log, as a proof reader meets it

Format, read from his file and from `src/common/logging.rs`:

```
2026-09-20T03:54:43.758625Z  INFO wixen_mail: Starting Wixen Mail v1.0.0-alpha.1+321.g4a09bfc2
<UTC stamp>  <LEVEL> <module path>: <message>
```

The filter is "this crate at the chosen level and no library", overridable by
`RUST_LOG` (`src/common/logging.rs:133-136`), so no HTTP or IMAP library line
is ever written. `[VERIFIED: src/common/logging.rs:110-136 read this session]`

What 2026-09-20's 8,309 lines held, by module (Info 7,774, Warn 535, no Error):
`presentation::wx_app` 4,943 Info and 137 Warn; `service::protocols::imap`
1,370; `presentation::accessibility` 761 ("Speaking: ..."); `application::mail_controller`
662; `application::mail_sync` 397 Warn (390 of them "Could not fetch the text
of message N: Network error: The mail server closed the connection"); 
`application::moves_waiting` 5. `[VERIFIED: Python count over the file]`

What the log can already answer for a proof:

| Question | Line | Where written |
|---|---|---|
| Which build ran | `Starting Wixen Mail v<full build>` | `src/main.rs` |
| Which level | `Logging initialized at level: Info` | `src/common/logging.rs` |
| A move, copy or delete reached the server | `A waiting move was replayed: {spoken}`, `A waiting copy was replayed: copied to {into}`, `A waiting delete was replayed: {spoken}` | `src/application/moves_waiting.rs:561,567,573` |
| A crossing to another account waits | `A move of message {row} to another account waits: {why}` | `moves_waiting.rs:967` |
| A refused delete could not be undone here | Warn | `moves_waiting.rs:731` |
| What the program asked the screen reader to say | `Speaking: <sentence> topic=...`; muted message content appears as `Speaking N characters that are not written` | `presentation::accessibility` |
| A message was sent | `Sending email from <masked> to [<masked>]`, `Email sent successfully` | `src/service/protocols/smtp.rs:588,610` |
| A PIM sync failed | `Contacts sync error: ...`, `Calendar sync error: ...`, `Task sync: ...`, `Notes sync: ...` (Warn only) | `src/presentation/wx_app.rs:23687,23800,31031,31131` |

`[VERIFIED: grep and Read of each site this session]`

A check on the "Speaking" lines: their shape in his 09-20 file is
`Speaking: Conversation topic=Some("thread...")` and `Speaking: N messages, N
unread topic=...`; no row text or subject appeared in the shapes sampled. That
is a sample, not a proof that no spoken line ever carries a subject; the
reader script masks quoted strings and addresses anyway. A "Speaking" line
records what the program handed to the screen reader. It is not what NVDA
said: interruption, focus and the MSAA channel are only Pratik's ear.

### What the log cannot answer, and why each matters to a proof

1. **The PIM sync's request and answer.** The contacts and calendar
   completions log only `result.errors` (`wx_app.rs:23686-23688`,
   `23799-23801`); `google_api.rs` logs two warnings, neither about a
   request. A sync that runs, asks Google, and gets nothing writes no line;
   a sync that never runs writes no line either. REAL-01's criterion 1
   ("or the log says which request the sync made and what Google answered")
   needs an Info line per source per run: account, provider, which source it
   used or why none, what it asked, the HTTP status, how many items came and
   how many were written.
2. **Which message a replay line is about, and how the server took it.**
   "A waiting copy was replayed: copied to Work" carries no row id, no
   server number, no `COPYUID`, and does not say whether the server took
   `MOVE` or `COPY` then a delete. Two copies in one minute cannot be told
   apart, and ledger 187's questions (what Gmail makes of an appended
   message, what a duplicate identifier does) are answered by the new
   server number, which is not logged.
3. **How the account signed in.** `Signed in to imap.gmail.com`
   (`imap.rs:648`) does not say password or browser sign-in, which is the
   question item 2 above had to answer from the database.
4. **Which build crashed.** The panic hook writes `Wixen Mail v{CARGO_PKG_VERSION}`
   (`src/main.rs:437-442`), so a crash entry says `1.0.0-alpha.1` and never
   the `+N.g<hash>` build. The one real panic in his `crash.log`,
   2026-09-18 08:35:45, cannot be tied to a build except by its date.

## crash.log in his profile is written by the test suite

`[VERIFIED: Python count over %LOCALAPPDATA%\wixen-mail\logs\crash.log, 2026-10-04]`

| Count | Entry |
|---|---|
| 1,515 | `--show-page was given "mailto:somebody@example.com", which is not a page. Nothing was opened.` |
| 1,515 | `--show-page was given "not-a-page", which is not a page. Nothing was opened.` |
| 1,515 | `--show-page was given "", which is not a page. Nothing was opened.` |
| 1 | `PANIC at src\presentation\wx_app.rs:...` on 2026-09-18, the only entry from a real run |

Entries per day run from 72 to 882 between 2026-09-22 and today. The three
strings are exactly the three inputs of
`test_an_address_that_is_not_a_page_opens_no_window` in
`src/presentation/page_window.rs:525-536`, which calls `show()`, whose refusal
path calls `crash_log()` (`page_window.rs:451-470`), which writes to
`crate::common::logging::default_log_dir()`, which resolves to the real
profile through `AppPaths::resolve()` (`src/common/logging.rs:110-114`). So
every library test run on this machine appends three lines to the tester's
own crash file, breaking CLAUDE.md's "anything touching the filesystem uses
`tempfile` rather than real user directories". `WIXEN_MAIL_DATA`
(`src/common/paths.rs:51`) is the existing override the test could set, or
`crash_log` can take its folder. No ledger entry names this today
(`grep -n crash .planning/WINDOWS.md` finds only unrelated rows).

## The build a phase-14 proof needs

### How it is made

`scripts/build-installer.sh` (read in full this session):

- reads the version from `Cargo.toml` (`:12`), finds the commit that set it
  with `git log -S` (`:39`) and refuses a clone that does not hold it (`:40-51`);
- counts `LAG`, commits since that commit (`:55`), and names the build
  `$VERSION+$LAG.g<8-hex>`, with `.dirty` appended when the tree has
  uncommitted edits (`:81-88`); nothing is appended at a tag (`:74-75`);
- exports `WIXEN_BUILD` so the program and its log say the build (`:97`);
- encodes the file version as `stage * 13000 + step * 1000 + counter`, with
  step capped at 12 and counter at 999, each cap said on the console
  (`:133-158`);
- builds the release, then the search handler crate, then the setup file
  with ISCC (`:207-222`), and prints `Built dist/Wixen-Mail-Setup-<full>.exe`.

`[VERIFIED: scripts/build-installer.sh:1-225 read this session]`

At HEAD today that gives `1.0.0-alpha.1+1071.g01ef4589` and file version
`1.0.0.14999` (stage 1 times 13000, plus step 1 times 1000, plus the counter
capped at 999). `[VERIFIED: git rev-list --count 01ff57bf..HEAD = 1071; the
arithmetic is the script's own formula at :158]` The script will print
"== the build counter is 1071 and the file version holds it to 999 ==",
which is expected and not a failure.

### Two routes to the file, both the same script

| Route | What it takes | Notes |
|---|---|---|
| Built here | `bash scripts/build-installer.sh` on a clean tree at the merge commit | ISCC found at `%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe` (checked with Python, exists), cargo 1.98.1, about the release-build time the gate rows quote |
| CI's artifact | `gh run download <run id> -n setup-executable` from the CI run of the push of `main` | `ci.yml:243-292`, job "Setup Executable", `fetch-depth: 0`, uploads `dist/Wixen-Mail-Setup-*.exe` as `setup-executable`; run 37160119665 (13.1-01's merge) holds one, 15,010,845 bytes, expiring 2027-01-01 |

Both builds are unsigned: the certificate is Pratik's and not raised
(memory, 2026-09-11). A local build needs nothing pushed; the CI route needs
`main` pushed, which is Pratik's word each time. Recommend the local build at
the last code plan's merge commit, with a clean tree so no `.dirty` is
stamped.

### Installing it over his copy

The setup keeps one `AppId` (`installer/Wixen-Mail-Setup.iss:24`) and allows
a per-user or machine-wide install (`:39-40`). Inno Setup's
`UsePreviousPrivileges` defaults to yes: "at startup Setup will look in the
registry to see if the same application is already installed in one of the
two install modes, and if so, it will use that install mode and not ask the
user." `[CITED: jrsoftware.org/ishelp/topic_setup_usepreviousprivileges.htm]`
The `.iss` does not set it, so the new setup will take the machine-wide mode
of the `+351` install and ask for elevation through UAC; it will not create a
second, per-user copy. `CloseApplications=yes` (`:103`) asks to close a
running copy. His profile and its settings stay where they are.

### Versioning: no rule requires a bump, one cap now bites

CLAUDE.md: the tree stays at `1.0.0-alpha.1`; after a build is *cut* the first
behaviour change moves the prerelease counter; releases are cut deliberately
through the Release workflow. `git tag` lists no tags, so no build has been
cut, and the builds handed to Pratik are tester builds ordered by the
metadata counter (Pratik's decision of 2026-09-17, memory). So no bump is
owed. What has changed is the cap: past 999 the file version stops ordering
builds in Apps and Features, so every phase-14 fix build reads `1.0.0.14999`.
The version string, the About box, `--version` and the log's first line all
still order correctly. Moving to `1.0.0-alpha.2` resets the counter (file
version `1.0.0.15000` plus the new counter); `patch` must never be dispatched
on a suffixed version (CLAUDE.md). This is a question for Pratik, not a
planner's decision (question 2).

### What the alpha page owes the build

`docs/ALPHA_TESTING.md` lists "What would help most to hear about", items 1
to 22, and item 8 (deleting, moving and copying), item 17 (a report from a
real account) and item 20 (an undo naming the item) are the proofs' walks in
a tester's words. Its "Everything that writes is experimental" paragraph and
the Allow Changes table are two of the four warning surfaces REAL-02 rewords
on Pratik's word. Its "How to report something" section names the build
format (`1.0.0-alpha.1+114.g44bff634`) and says log files sit under `logs`.
Nothing on the page says what changed between `+351` and the phase-14 build;
the changelog's `[Unreleased]` section carries it, and it is 720 commits long.
A sitting sheet should name only what that sitting touches.

## Package Legitimacy Audit

No external package is proposed. The proof reader uses only the Python
standard library (`sqlite3`, `json`, `re`, `hashlib`, `argparse`, `pathlib`,
`datetime`); the log lines use the `tracing` crate already in the tree.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| none | | | | | | |

**Packages removed due to [SLOP] verdict:** none.
**Packages flagged as suspicious [SUS]:** none.

## Architecture Patterns

### How a proof is read, end to end

```
Pratik's sitting                         this machine, after the sitting
----------------                         -------------------------------
install build ──> first start ──┐
                                │  writes   %LOCALAPPDATA%\wixen-mail\
steps from the sheet ───────────┼────────> logs\wixen-mail.<UTC date>.log
  (move, copy, delete,          │            (Starting ... v<build>,
   network off, restart,        │             A waiting ... was replayed,
   Refresh in a module)         │             Speaking: ..., Warn lines,
                                │             [new] sync per source)
what he heard ──> one line      │          cache\message_cache.db (+ -wal)
  per item, said to the agent   │            moves_waiting, move_in_flight,
                                │            messages.folder_id / uid,
                                │            calendars, contacts, sync_state
                                v
            agent runs scripts/read-a-proof.py --since <UTC> [--message <row>]
            (Python, mode=ro, masks addresses and quoted text, prints counts)
                                │
                                v
            the record: #63 or #22 comment with date + build + what the
            reader printed; manual-pass items dated; ledger rows closed
```

### Pattern 1: read the profile with the format's own reader, read-only

**What:** open the database with `sqlite3.connect("file:<path>?mode=ro", uri=True)`
and the settings with `json.load`; read logs as text with
`errors="replace"`.
**Why:** `MessageCache::new` runs additive migrations and sets pragmas, so any
reader built on the crate's own store writes to his database; and bash on
this machine sees a stale July copy of AppData (memory). `mode=ro` includes
the `-wal` contents, which matter: his `-wal` is 16 MB of the last day's
writes. Do not use `immutable=1`, which skips the write log.
**Example (the shape used for every reading in this research):**

```python
# Source: this session's readings, 2026-10-04
import os, sqlite3
db = os.path.join(os.environ["LOCALAPPDATA"], "wixen-mail", "cache", "message_cache.db")
con = sqlite3.connect("file:" + db.replace("\\", "/") + "?mode=ro", uri=True)
for kind, asked in con.execute("SELECT kind, substr(asked_at, 1, 16) FROM moves_waiting"):
    print(kind, asked)
```

The column names in that example are quoted from `PRAGMA table_info` this
session: `moves_waiting` holds `message_row_id, account_id, from_folder_path,
uid, kind, into_folder_path, asked_at, to_account_id, to_account_name`.
`[VERIFIED: PRAGMA table_info(moves_waiting), 2026-10-04]`

### Pattern 2: test a log line with the capture the tree already has

`CapturedLogs` in `src/presentation/accessibility/screen_reader.rs:723-771`
is a `tracing::Subscriber` that keeps each event's level and message, with
`has(level, contains)`. It is `pub(crate)` inside that module's `tests`, and
`feedback.rs` already reads it. A new Info line is driven red by asserting
`captured.has(tracing::Level::INFO, "...")` under
`tracing::subscriber::with_default`. `[VERIFIED: read this session]`

### Pattern 3: a sitting sheet

One page per sitting, written against the installed build, listing in order:
the proof's steps (from REAL-02 [D] and ledgers 546, 547), the manual-pass
items that walk the same screen at the same moment, and where the agent reads
before and after. Pratik reads the sheet once, works it, and reports one line
per item ("heard as written", or what he heard instead). The agent's reading
supplies everything the log and database can say.

### Anti-Patterns to Avoid

- **Running any test or gate while Pratik is in a sitting.** Window tests
  build live windows on the same desktop and can take focus from NVDA; the
  library tests write to his `crash.log`; a start of the product binary can
  meet the single-instance handover (`src/service/handover.rs`, a pipe named
  per logon session) `[ASSUMED: whether a test child reaches the handover;
  not traced]`. Hold every cargo run until he says the sitting is over.
- **Turning the network off while an agent is working.** The "network off"
  steps take this machine off the network, and an agent session on it stops
  with it. The agent reads after the network is back.
- **Reading the profile with bash or PowerShell.** It shows a stale copy.
- **Matching a proof to its log line by memory.** Use the UTC stamp and,
  once Plan B lands, the message row id.
- **Quoting a "Speaking" line as what was heard.** It is what was sent.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Opening his database | anything on `MessageCache` or a writing connection | `sqlite3` with `mode=ro` | The crate's open migrates and sets pragmas; `secure_delete` is on for every connection since 13.1-03 |
| Masking addresses in the reader | a new regex from memory | the rule `feedback_report::redact` and `logging::mask_email` already use (`src/application/feedback_report.rs:406-475`, `src/common/logging.rs:163`), mirrored in the script and held to the same cases | One masking rule, two languages; a test feeds both the same lines |
| Capturing log lines in tests | a new subscriber | `CapturedLogs` (`screen_reader.rs:723`) | It exists and two modules use it |
| Naming the build | a version string in a sheet typed by hand | the log's `Starting Wixen Mail v...` line, read by the script | The installed build is what ran, not what was planned |
| A second install mode | `/CURRENTUSER` or a custom installer path | the default; `UsePreviousPrivileges` follows the existing machine-wide install | A per-user install beside the machine-wide one is two programs |

## Common Pitfalls

### Pitfall 1: the first start sends two old deletes
**What goes wrong:** the new build's first check replays the two
`delete_to_trash` rows queued on 2026-09-20 before any planned step.
**Why it happens:** mail is allowed in his Settings and the queue is replayed
at a check by design.
**How to avoid:** decide beforehand (question 3). Either start once with
`--read-only` ("Changes nothing at any server for that run, whatever the
settings say", `docs/ALPHA_TESTING.md`) for the sittings' listening-only
items, or treat the replay as the delete line's restart case and read
`moves_waiting` before and after.
**Warning signs:** two "A waiting delete was replayed" lines within a minute
of the first `Signed in to imap.gmail.com`.

### Pitfall 2: a proof read from the wrong day's file
**What goes wrong:** a sitting at 21:00 his time lands in the next UTC day's
file and the reader finds nothing.
**How to avoid:** the reader takes a UTC window and reads every daily file
the window touches.

### Pitfall 3: crash.log read as evidence
**What goes wrong:** thousands of test lines bury a real crash, and the real
one names no build.
**How to avoid:** Plan C before the build; until then the reader filters the
three `--show-page` lines and says how many it dropped.

### Pitfall 4: a new script the gate cannot place
**What goes wrong:** a new file under `scripts/` that no suite list names
makes `check.sh` run every shell suite on every commit touching it
(`scripts/check.sh:135-141`, "a path under `scripts/` ... that no list
places").
**How to avoid:** add the script to `what_no_suite_reads` or to the list of
the suite that tests it, in the commit that adds it, and give the Rust target
that tests it a `guards/guards.toml` record whose `file` is the script, so
the scoped gate runs the target when the script changes.

### Pitfall 5: a capture that misses events from another thread
**What goes wrong:** `tracing::subscriber::with_default` sets a thread-local
default; an Info line written on a tokio worker thread is not captured, and
the red test stays red for the wrong reason or a green one passes over
nothing.
**How to avoid:** drive the replay under a current-thread runtime in the
test, as the existing `moves_waiting` tests' shape allows, and assert one
negative case (no line when nothing replayed).

### Pitfall 6: the counter cap read as a failure
**What goes wrong:** the build prints the 999 cap and somebody stops.
**How to avoid:** the sheet and the plan say the cap is expected at 1,071
and what it costs.

## Code Examples

### The Info line a replay should write (shape for Plan B)

```rust
// Shape only. Names quoted from src/application/moves_waiting.rs:559-575,
// read this session: move_it, copy_it, delete_it, moved.spoken(into),
// deletion.spoken(). The row id and the new server number are the additions,
// and how they reach these functions is the plan's to decide.
tracing::info!("A waiting copy was replayed: copied to {into}");
```

The current three lines are the ones quoted above; the plan adds the message
row id and what the server answered with (its new number when it gives one),
never a subject.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Build named by commit alone (`+g59c5b6a4`) | `+<counter>.g<hash>`, counter in the file version | 2026-09-17 | Builds ordered by reading; capped at 999 in the file version, now passed |
| Profile read by hand in a session | (proposed) one read-only script in the tree | this phase | A proof is read the same way each time and can be re-read |

## Combining the proofs with the manual pass

`docs/manual-accessibility-pass.md` holds 218 numbered items: 183 under A
(screen readers) and 35 under B to F. Items 84 to 183, written for phases 12,
13 and 13.1, split 39 NVDA, 21 Both, and 40 that need an account, a
certificate, a directory or Outlook. `[VERIFIED: Python parse of the page,
2026-10-04]` (The page's closing section still says "Seventy-six items", its
count of 2026-09-14; the A section's own paragraph says one hundred and
eighty-three.)

The items that walk the proofs' own screens:

| Sitting | Proof | Manual-pass items in the same walk | Alpha page list |
|---|---|---|---|
| 0. Install and first start | the two waiting deletes, if Pratik lets them go; the build name in About and the log | 64 (Tab into the list), 81 (All Inboxes on a profile), anything on the main window he wants | 1, 2 |
| 1. Calendar, contacts, tasks (REAL-01) | Refresh in each module; adding the account again if the cause is the sign-in | 92 (five name parts sync), 106 (undo on Google, its Google part), 110 (accept on Google), 153 (free time on Google) | none yet |
| 2. Copy, then move, in one account | copy; move; each with the network off and back; each after a restart; one changed in Gmail on the web meanwhile | 66 (all of it), 104 (undo a move, a delete and a copy, before and after the server heard), 65 (selection, Delete over several) | 8, 14, 20 |
| 3. Delete | delete; network off; restart; Trash at the server | 104's delete half if not done, 172 (Trash after days, Gmail has UIDPLUS), 182 needs a POP account | 8 |
| 4. Across two accounts | copy and move to the other account; restart with a crossing waiting; a message over 25 MB | 67 (all of it), ledger 187's three questions | 8 |
| (optional) Send again | sending on the new build | 89 (a report from a real account), 162 (send from another address) | 17 |

Items 174 and 175 (a POP account and a server without UIDPLUS, ledgers 784
and 787) need an account Pratik may not have; they stay open unless he names
one (question 5).

What the agent does around each sitting, so Pratik's report is one line per
item:

1. Before: the reader's snapshot (build, level, queue rows, folder counts for
   the folders the sheet uses, PIM counts by provider).
2. Pratik works the sheet with NVDA. The agent does nothing on this machine,
   and nothing at all while the network is off.
3. After: the reader over the sitting's UTC window: every replay line, every
   "Speaking" line in order, every Warn and Error, the queue again, and for
   each message the sheet names, its folder type and server number before
   and after.
4. The agent drafts the record (issue comment with date, build and the
   reader's lines; each manual item with date, "NVDA <version>", and his one
   line), Pratik confirms, and only then is anything posted or ticked.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | An app-password Gmail account has no OAuth token for the Google PIM calls, and this is #22's cause | What his profile holds, item 2 | REAL-01's fix aims at the wrong thing; the REAL-01 research should settle it |
| A2 | A test child starting the product binary can reach the single-instance handover while Pratik's copy runs | Anti-Patterns | Low: the rule (no test runs during a sitting) holds anyway for focus reasons |
| A3 | `--read-only` holds the queue's replay as well as new writes, and keeps the rows waiting | Pitfall 1 | Partly read: the IMAP client's writes go through `outward::permitted`, which refuses with `Error::Security` (`src/service/outward.rs:292-297`), and `why_the_push_failed` reads `Security` as "this computer refused it", which calls for keep and wait, not put back (`src/application/flag_changes_waiting.rs:134-163`). Not traced: that `--read-only` reaches the replaying session's `may_change`. If it does not, the two deletes go on the first start |
| A4 | Python is on CI's Windows runner for a Rust test that runs the reader script | Validation Architecture | The test cannot run in CI; fall back to a shell suite or mark it local |
| A5 | Four or five sittings is about right | Combining | A planning estimate, not a measurement; Pratik's time decides |

## Open Questions

1. **Which build route** (local at the merge commit, or CI's artifact after a
   push)? Recommendation: local, clean tree.
2. **`alpha.1` or `alpha.2`** for the phase-14 build, given the 999 cap?
3. **The two waiting deletes:** let them replay as evidence, or first start
   with `--read-only`? Recommendation: let them replay, read before and after;
   they are deletes he asked for on 2026-09-20.
4. **Sign-in:** will he sign in to Gmail through the browser for REAL-01 (the
   seven-day expiry for test users applies, `docs/ALPHA_TESTING.md`), and was
   the send proof of 2026-09-18 through OAuth, as #63 says, or the app
   password the profile shows?
5. **A second account** for the across-account lines, and whether any POP or
   non-UIDPLUS account exists for ledgers 784 and 787.
6. **Log level:** new lines at Info (recommended, nothing for him to change),
   or Debug for the sittings?
7. **Reading his profile before and after each sitting**, with his agreement
   as REAL-01 [D] puts it; and whether the sittings carry the manual items
   above.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Inno Setup 6 (ISCC) | the setup file | yes, per-user install (Python `os.path.exists` true) | 6 | CI's Setup Executable job |
| cargo / rustc | the release build | yes | 1.98.1 | none needed |
| Python | the proof reader; every profile read | yes | 3.14.3 | none: bash cannot see the profile |
| gh | issue records, CI artifact | yes | 2.88.1 | none needed |
| A browser sign-in to Google | REAL-01, if the cause is the sign-in | Pratik's | | none |
| A second mail account | REAL-02 across two | unknown | | lines stay open |
| A POP account, a server without UIDPLUS | ledgers 784, 787 | unknown | | items 174, 175 stay open |

**Missing dependencies with no fallback:** the second account, if Pratik has
none.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (unit tests beside code, integration targets in `tests/`); shell suites `scripts/*.test.sh` through `scripts/shell-suite.sh` |
| Config file | none beyond `Cargo.toml`; suites listed in `scripts/check.sh:135-141` |
| Quick run command | `cargo test --lib application::moves_waiting::` (one `--lib` per run; join several with `&&`) |
| Full suite command | `bash scripts/check.sh all`, once, in the phase's closing plan |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REAL-01 | A PIM sync run writes one Info line per source with what it asked and what came back, or why it did not run | unit, `CapturedLogs` | `cargo test --lib application::contacts_sync::` and the calendar and tasks modules, one run each | no, Wave 0 |
| REAL-02 | A replayed move, copy or delete writes the row id and the server's answer at Info | unit, `CapturedLogs` | `cargo test --lib application::moves_waiting::` | no, Wave 0 |
| REAL-02 | The reader prints a sitting's replay lines and a message's folder before and after, and never a subject or address | integration over a fixture profile in a temp directory | `cargo test --test <new target>` | no, Wave 0 |
| (crash log) | The page window's refusal writes to the folder it is given, not the real profile | unit with `tempfile` | `cargo test --lib presentation::page_window::` | no, Wave 0 |
| REAL-01, REAL-02 | The proofs themselves | manual, Pratik with NVDA and his account | none | not automatable |

### Sampling Rate
- **Per task commit:** the scoped `--lib` run the hook chooses.
- **Per wave merge:** the hook's branch-diff run at `git merge --no-ff`.
- **Phase gate:** `scripts/check.sh all` once in the closing plan.

### Wave 0 Gaps
- [ ] a fixture-profile builder for the reader's test: a temp directory with a
  database made by the real schema and a two-file log spanning UTC midnight
- [ ] a guard record for the reader script and its target, and its place in
  `check.sh`'s lists

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes, indirectly | the reader never reads `accounts.password`, the credential store or `oauth.toml`; the sign-in method is logged as a word, never a token |
| V3 Session Management | no | |
| V4 Access Control | yes | `application::allowed` stays the gate; no plan moves a default without Pratik's word |
| V5 Input Validation | yes | log text and folder names are untrusted; the reader masks addresses and quoted strings before printing |
| V6 Cryptography | no | |
| V7 Error Handling and Logging | yes | new Info lines carry ids and counts, never subjects, bodies, addresses or tokens (CLAUDE.md, "Never log a token, password, or message body") |

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| A diagnostic line leaking a subject or address | Information disclosure | masking at the writer (`mask_email`) and again in the reader; a test feeds the reader a subject and an address and asserts neither is printed |
| A reader that writes to the profile | Tampering | `mode=ro`; never the crate's store |
| A test writing into the real profile | Tampering | `tempfile` or `WIXEN_MAIL_DATA` in every test that writes (Plan C) |

## Sources

### Primary (HIGH confidence)
- The tree at `01ef4589`: `scripts/build-installer.sh`, `installer/Wixen-Mail-Setup.iss`,
  `.github/workflows/ci.yml:243-292`, `src/application/moves_waiting.rs`,
  `src/presentation/page_window.rs`, `src/common/logging.rs`, `src/main.rs`,
  `src/application/allowed.rs`, `src/presentation/accessibility/screen_reader.rs`,
  `src/application/feedback_report.rs`, `scripts/check.sh`
- `CLAUDE.md`, `.planning/ROADMAP.md` (phase 14), `.planning/REQUIREMENTS.md`
  (REAL-01, REAL-02), the phase 13 and 13.1 READMEs, `docs/ALPHA_TESTING.md`,
  `docs/manual-accessibility-pass.md`, `.planning/WINDOWS.md` ledgers 187,
  191, 376, 546, 547, 771, 772, 784, 787
- Pratik's profile and the registry, read-only with Python, 2026-10-04
- `gh issue view 22 --comments`, `gh issue view 63 --comments`,
  `gh run view 37160119665`, the artifacts API for that run

### Secondary (MEDIUM confidence)
- Inno Setup help, `UsePreviousPrivileges`:
  https://jrsoftware.org/ishelp/topic_setup_usepreviousprivileges.htm

## Metadata

**Confidence breakdown:**
- Installed build and profile: HIGH, read directly this session.
- Build tooling: HIGH, script and workflow read in full; the counter computed.
- Gaps in the log: HIGH for what is absent (every log call in the named
  modules listed); the PIM cause is LOW and belongs to another research.
- Sitting design: MEDIUM, a proposal.

**Research date:** 2026-10-04
**Valid until:** the next merge that touches `moves_waiting.rs`, the PIM
syncs or `build-installer.sh`, or the next time Pratik starts the program,
which changes his profile.
