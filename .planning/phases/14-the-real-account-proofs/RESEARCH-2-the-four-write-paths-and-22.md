# Phase 14: The real-account proofs. Research 2: the four write paths, #22, and the gate

**Researched:** 2026-10-04, against `main` at `01ef4589` (clean, pushed), version `1.0.0-alpha.1`
**Domain:** proving mail writes and the Google personal-information sync against Pratik's own account, by ear, in short sessions; and what moving `application::allowed` means
**Confidence:** HIGH for what the tree and the profile hold; MEDIUM for Gmail's server behaviour (secondary sources); the proofs themselves are, by design, not taken

<user_constraints>
## User Constraints (from CONTEXT.md)

No CONTEXT.md exists for phase 14; the phase folder was empty when this was written. The binding
constraints are the roadmap entry and requirements, quoted here as the planner must honour them:

- Roadmap, phase 14 goal (`.planning/ROADMAP.md:1798-1808`): each write path proved against
  Pratik's account "each with its own line", "re-taken after phase 11's queue made them complete
  here first (11-07.1, 11-07.2)"; a Gmail account added brings its calendars and contacts (#22,
  "the profile's log at the moment of the refresh is what the fix reads first"); then
  `application::allowed`'s default for each proven path moves, and the warning sentences on the
  settings screen, the first-run screen, the end of `--help` and the alpha page follow, "on
  Pratik's word".
- Order (`ROADMAP.md:1809-1812`): "#22 first, because it is a defect against his account with a
  log to read, then #63's four lines in the order copy, move, delete, since a copy is the least
  destructive." The order is "the planner's, for him to confirm".
- REQUIREMENTS `REAL-02` [D] (`REQUIREMENTS.md:7231-7235`): "A steps page per path against the
  build that carries phase 12, each run by him and recorded on the issue with the date and the
  build; a move made with the network off and replayed, one replayed after a restart, a message
  another client changed meanwhile."
- The delete line also owes ledgers 771, 784 and 787 (`REQUIREMENTS.md:7237-7240`).
- Hard rules of this research brief, which the planner should carry into every plan that reads
  the profile: never start the installed or release program, never sign in, never read a password
  or token, never send mail, never touch NVDA; read the profile only through Python, read-only;
  print counts and log lines about requests and errors, never bodies, subjects, or addresses
  beyond the domain.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research support |
|----|-------------|------------------|
| REAL-01 | Adding a Gmail account brings its calendars, contacts and tasks, and Refresh in each module brings what the account has | Cause found without his help (section 1): his account signs in with a password, every Google calendar, contacts and tasks request needs a browser sign-in, his build carries no Google client, and the sync skips Google silently and reports "0 created". The log cannot show it at any level, because no line is written for a request or a skip. Fix design and proof steps in sections 1 and 4 |
| REAL-02 | Copy within an account, copy across two, move within, move across two, and delete, each proved and recorded; the gate's default moves per path on his word | What 11-07.1 and 11-07.2 made complete here first (section 2), what the log and database already record and fail to record (section 3), one session design per line with the reading that decides it (section 4), and what "per path" can and cannot mean for a gate that has one mail switch (section 5) |
</phase_requirements>

## Summary

**#22 is explained, and the explanation is not the one the issue offers.** The issue names two
readings: nothing runs at account creation, or the sync runs and brings nothing. Read on
2026-10-04, his profile and his installed build show a third that contains the second: the
account signs in to Gmail with a password (`use_oauth` 0), Google's calendar, contacts and tasks
interfaces accept only a browser sign-in (OAuth 2.0), and the build he runs carries no Google
OAuth client at all, so `oauth_credentials::credentials_for("gmail")` answers nothing and
`spawn_calendar_sync`, `spawn_contacts_sync` and `spawn_tasks_sync` skip the Google branch without
a word. The finish then says "Calendar sync: 0 created, 0 updated, 0 deleted", which reads as
success. A program of his said this in August, in its own words: "No sign-in credentials are set
up for gmail, so signing in through the browser cannot run." Even with a client and a browser
sign-in, the calendar sync reads only Google's `primary` calendar, so "its calendars", plural, do
not arrive either.

**The log cannot settle #22 at any level, and the roadmap's premise that it can is wrong.** No
code in the Google calendar, contacts or tasks paths writes a log line for a request, an answer
or a skipped provider (`google_api.rs` has two `tracing::` calls, both about retries and
timeouts; `tasks_api.rs` and `tasks_sync.rs` have none). His stored log level is `info`, not the
`debug` a fresh profile now starts at, and the logs for 2026-09-15 and 2026-09-17, when #22 was
reported, are 0 bytes (#66). Across 35,758 lines on 2026-09-18 to 2026-09-20 there is no line from
any personal-information sync. The fix writes those lines first, at info, test-first, the way
11-04 wrote the lines a report needs.

**For #63, the queue is in place and has already met Gmail, but nothing has been proved on
purpose.** On 2026-09-20 the log records five "A waiting delete was replayed: Moved to Trash"
lines against his Gmail account, and two deletes still wait in `moves_waiting` from 20:56 that
day, so the first start of the next build replays them, one of which Gmail has very likely
already carried out. That is useful evidence and a free first reading; it is not a proof, because
nobody listened and no restart or network-off case was set up. A crossing between accounts writes
no log line when it succeeds, his profile holds one account, and the delete line's extra ledgers
need an IMAP account that is not Gmail or Microsoft (13-44.6 leaves Gmail's Trash to Gmail) and a
POP account. **The gate has one switch for all mail writes**, `Allowed::mail`, which also covers
flags, rules, Sent filing, read receipts, subscriptions and emptying the Trash; "the default
moves per path" is possible only by splitting that switch, and the warnings are already wrong
today about sending, proven on 2026-09-18.

**Primary recommendation:** make #22 a code plan first (say why nothing came, choose the provider
by the account, log each request and answer at info, fix the double count, decide the sign-in
route), then a short "proof kit" plan (crossing log lines, the reading script carried in the
plan, the steps pages), then two of Pratik's sessions, Gmail alone and then two accounts, each
recorded on #22 and #63 by date and build; reword the warnings per proven path as each line lands
and leave the default's shape to his answer.

## Architectural Responsibility Map

| Capability | Primary tier | Secondary tier | Rationale |
|------------|--------------|----------------|-----------|
| Which Google interfaces an account may ask (sign-in route, client present) | Application (`application::*_sync`, `service::oauth_credentials`) | Presentation (the sentence) | The decision is a rule about the account and the build; the window only says the answer |
| Saying why a sync brought nothing | Application (`what_the_calendar_sync_did` and siblings build the words) | Presentation (`a11y.signal(FeedbackEvent::SyncComplete, ..)`) | Words live where they can be tested; the window routes them |
| Request and answer log lines for Google | Service (`service::google_api`, `service::tasks_api`) | Application (one line per pass) | The one place every request goes through (`api_get`, `api_post`, `api_patch`, `api_delete`, `parse_response`) |
| Move, copy, delete made here first and replayed | Data (`data::message_cache::moves_waiting`) and Application (`application::moves_waiting`) | Presentation (`complete_here_then_tell_the_server`, `MovePutBack` arm) | Already built by 11-07.1 and 11-07.2; phase 14 proves it |
| The gate | Application (`application::allowed`, `data::config::default_allowed`) | Presentation (settings, first run, `--help`, the account editor) | One rule, read by every client before it is built |
| Reading a proof | A scratch Python reader over a copy of the profile | The tester's ear | The program must not be started by anybody but Pratik; the reader never signs in |

## 1. #22: what the profile, the build and the code say

### What his profile holds, read 2026-10-04 with Python, read-only

The profile is `%LOCALAPPDATA%\wixen-mail` (`src/common/paths.rs:108-119`: `config_dir()` is
`self.root.join("config")`, `cache_dir()` is `self.root.join("cache")`, `logs_dir()` is
`self.root.join("logs")`). It was last written on 2026-09-20 at 20:57Z. The database was copied
to the session scratchpad and read there, so nothing in the profile was opened for writing; the
copy is not present now.

| What | Value | Source |
|------|-------|--------|
| Accounts | 1: provider `Gmail`, `imap.gmail.com:993`, `smtp.gmail.com:465`, protocol `imap`, address at `gmail.com`, `use_oauth` 0, the database's `password` column empty (the secret is in the credential store, as it should be), `allow_deleting_here` 1 | `accounts` table [VERIFIED: profile copy] |
| Messages cached | 99,014 in 50 folders (12,872 when #22 was read on 2026-09-16) | `messages`, `folders` [VERIFIED: profile copy] |
| Calendars | 2, both `source_provider` `local`, both named "My Calendar", one filed under the Gmail account and one under `local`, created 2026-09-15; none from Google | `calendars` [VERIFIED: profile copy] |
| Events, contacts, address books, tasks | 0, 0, 0, 0; task lists 2 (`My Tasks`, local), note folders 2 (`General`, local) | [VERIFIED: profile copy] |
| `sync_state` | 0 rows, so no personal-information sync has ever finished a pass | [VERIFIED: profile copy] |
| `moves_waiting` | 2 rows, both kind `delete_to_trash`, `INBOX` to `[Gmail]/Trash`, asked 2026-09-20T20:56:35Z and 20:56:42Z; the two message rows sit in `[Gmail]/Trash` under uids 4294967295 and 4294967294 with `filed_here` 1 | [VERIFIED: profile copy] |
| `move_in_flight` | 0 | [VERIFIED: profile copy] |
| Settings | `"allowed_changes": {"mail": true, "personal_information": true, "reading": true}`, `"allowed_per_account": {}`, `"log_level": "info"`, `"version": "1.0.0-alpha.1"`, `"feedback_channels": "off=,edge_of_list=speech+braille+earcon"` | `config\app_config.json` [VERIFIED: profile] |
| OAuth client file | `config\oauth.toml` absent; no `WIXEN_*` variable in the process, user or machine environment | [VERIFIED: Python `os.path.exists`, `winreg`] |

Both answers in `allowed_changes` are on, so on his machine the gate has never been what stopped
anything; his own choice already allows every write.

### What the installed build carries

`C:\Program Files\Wixen Mail\wixen-mail.exe`, written 2026-09-20 08:56 local, searched by bytes:
no `apps.googleusercontent.com` and no `GOCSPX-`, so no compile-time Google client
(`option_env!("WIXEN_GMAIL_CLIENT_ID_DEFAULT")`, `src/service/oauth_credentials.rs:118-125`) was
baked in [VERIFIED: byte search]. Nothing under `scripts/` or `.github/` sets that variable
[VERIFIED: grep]. So no build made on this machine or by CI can sign a Gmail account in through
the browser unless an `oauth.toml` is placed in the profile's `config` folder.

The log of 2026-08-23 says the same thing in the program's words, four times:

```
Speaking: Account added. No sign-in credentials are set up for gmail, so signing in through the browser cannot run. See Setting up a provider in Help.
Speaking: Sign-in needs attention, Gmail <name>
```

and every September log shows the password route: "Signed in to imap.gmail.com" 426, 806 and
685 times on the three days, and no line naming OAuth. **Correction for #63's record:** its
comment of 2026-09-18 says sending was proven on "his Gmail account (OAuth, SMTP ...)". The
account and the log say password. The next comment on #63 should say so.

### Why nothing comes: the code path

`spawn_calendar_sync` (`src/presentation/wx_app.rs:31143-31367`), `spawn_contacts_sync`
(`:30810-30945`) and `spawn_tasks_sync` (`:30958-31045`) each do this for Google:

```rust
// src/presentation/wx_app.rs:31201-31235, abridged
if let Some(gmail_creds) = crate::service::oauth_credentials::credentials_for("gmail") {
    let auth = crate::service::oauth::AuthManager::new(aid, "gmail", ..);
    match handle.block_on(auth.get_valid_token()) {
        Ok(token) => { /* sync_google_calendar(..) */ }
        Err(e) => total_errors.push(format!("Google auth: {}", e)),
    }
}
```

With no client, the `if let` is false and nothing is recorded: no error, no log line, no
sentence. The Microsoft branch is the same shape and also skipped. The account's calendars are
`local`, so neither the server loop nor the feed loop runs. The finish (`UIUpdate::CalendarSyncComplete`,
`wx_app.rs:23771-23802`) builds its words with `what_the_calendar_sync_did`
(`src/application/calendar.rs:168-172`), which opens with
`"Calendar sync: {} created, {} updated, {} deleted"`, so he hears "0 created, 0 updated, 0
deleted" and logs nothing, since `errors` is empty.

Three more defects found on the way, each small and each the planner's to place:

1. **The provider is chosen by which clients exist, not by the account.** A Gmail account with
   both a Google and a Microsoft client configured would also try `get_valid_graph_token` and
   report "Microsoft auth: ..." as an error on every Refresh. The account editor already has
   the right question, `provider_of(account)` (`wx_account_manager.rs:3002`).
2. **`held_for_you_to_choose` is counted twice** in the Google and Microsoft arms:
   `wx_app.rs:31225-31226` and `:31258-31259` each hold
   `total_held_for_a_choice += result.held_for_you_to_choose;` on two consecutive lines; the
   calendar-server arm at `:31306` holds it once. A held event is reported as two.
3. **Only the primary calendar is read.** `sync_google_calendar` (`calendar.rs:737-767`) files
   everything under one provider calendar and asks `list_events` for `THE_MAIN_CALENDAR`
   (`google_api.rs:477`: `pub const THE_MAIN_CALENDAR: &str = "primary";`); nothing asks Google's
   calendar list. REAL-01 says "its calendars".

### Google's own rules (why a password can never reach these)

- CalDAV: "The CalDAV server refuses to authenticate a request unless it arrives over HTTPS with
  OAuth 2.0 authentication of a Google Account." [CITED: developers.google.com/workspace/calendar/caldav/v2/guide]
- CardDAV: "Google's CardDAV interface requires OAuth 2.0." [CITED: developers.google.com/people/carddav]
- The Calendar, People and Tasks REST interfaces the code calls take a bearer token; an IMAP app
  password is not one [ASSUMED from the code's own use of `get_valid_token`, consistent with the two quotes above].
- Installed applications: "it is assumed that these apps cannot keep secrets", so a client
  secret inside a desktop build is the expected shape, with PKCE recommended and the loopback
  redirect recommended for Windows desktop apps [CITED: developers.google.com/identity/protocols/oauth2/native-app].
- While Pratik's Google project is in testing, only listed testers can sign in and a sign-in
  expires after seven days (`docs/ALPHA_TESTING.md:164-170`; the account manager's own comment at
  `wx_account_manager.rs:265-267`) [CITED: in-repo pages; Google's rule itself not re-read this session].

The scopes the browser sign-in asks for already include all three
(`src/service/oauth.rs:72-84`: `"https://mail.google.com/"`,
`"https://www.googleapis.com/auth/contacts"`, `"https://www.googleapis.com/auth/calendar"`,
`"https://www.googleapis.com/auth/tasks"`) [VERIFIED: read]. The account editor's box is
`"Sign i&n with the provider in a browser (OAuth)"` (`wx_account_manager.rs:2089`) and the
account manager has `"&Sign In Again"` (`:271`) [VERIFIED: read].

### What the log shows and what it cannot

Read with Python over `logs\wixen-mail.2026-09-18.log` (8,478 lines), `-19` (18,970) and `-20`
(8,310); each begins "Logging initialized at level: Info".

**It shows:**

- The Calendar module opened four times ("2 calendars loaded", "0 calendar events loaded"), twice
  on 2026-09-18 and twice on 2026-09-20.
- No calendar, contacts or tasks sync result spoken on any of the three days. SyncComplete goes
  to every channel by default (`feedback.rs:655-661`) and his `feedback_channels` disables none,
  so a sync that finished would have left a `Speaking:` line. So he did not press Refresh or a
  Sync command in those three days, or the line was not written.
- Five delete replays at Gmail on 2026-09-20 (04:21:00, 13:33:05, 13:33:06, 13:34:51, 20:57:43):
  `A waiting delete was replayed: Moved to Trash`.
- The sending proof: on 2026-09-18 at 12:37:31 one SMTP send to one recipient at another domain,
  then "Email sent successfully".
- Faults the proofs will run into (section 6): on 2026-09-19, 118 sign-ins refused with Gmail's
  "Too many simultaneous connections"; on 2026-09-18 and 2026-09-20, the Trash folder unreadable
  at nearly every check (107 and 79 spoken errors "Trash: The connection to the mail server was
  lost ..."); and about 390 a day of "Could not fetch the text of message N: Network error: The
  mail server closed the connection while opening the folder".

**It cannot show:**

- Any request a personal-information sync made, or Google's answer: no such line exists in the
  code at any level.
- The refresh #22 was reported from: the 2026-09-15 and 2026-09-17 logs are 0 bytes (#66, since
  fixed).
- Anything at debug: his stored level is `info` and wins over the version's default
  (`src/main.rs:111-116`; `data/config.rs:697-701`). REQUIREMENTS' sentence "the log ... is at
  Debug since 11-04" is true of a fresh profile and not of his.
- A crossing between accounts that succeeded: `mail_across_accounts.rs` and the crossing replay
  write no line on success (section 3).
- What he heard. The log records what was sent to the screen reader, not what NVDA said.

### The fix, as a code plan before any session

Test-first throughout, in this order:

1. **Say why nothing could be asked.** Red: unit tests over a pure decision, "which providers may
   this account ask, and if none, why", taking the account's provider and sign-in route and
   whether a client exists. Three reasons with their own sentences: this build has no Google
   client; this account signs in with a password and Google gives calendars, contacts and tasks
   only to a browser sign-in; the browser sign-in has expired. Green: each `spawn_*_sync` asks
   that decision first and puts the sentence in the finish instead of "0 created".
2. **Choose the provider by the account**, not by which clients exist (defect 1).
3. **Log each pass at info**: which provider was asked, the request's method and path with no
   query values, the HTTP status, and Google's error reason; or which reason stopped the pass.
   Bounded: one line per request a pass makes. Red as source readings in
   `tests/the_log_carries_what_a_report_needs.rs`, whose lexical guard already refuses any
   `tracing::` call naming a subject, body, password or token (11-04-SUMMARY lines 18 and 88).
   Info, not debug, because his stored level is info.
4. **The double count** (defect 2): one red case, one line removed.
5. **On account creation**: REAL-01's [D] says the sync runs at creation as well as on Refresh.
   For an account that cannot ask, creation says the reason once instead. What is said while a
   first sync runs is a feedback decision (guardrail 5): one SyncComplete per module at most.
6. **Calendars beyond the primary** (defect 3): Google's calendar list read and each calendar
   filed under its own row. This one is the largest; it may be its own plan.

The sign-in route is Pratik's decision (question 1). Route A uses what exists: an `oauth.toml`
with his Google project's client in the profile's `config` folder, or the same client baked into
his builds by `scripts/build-installer.sh`. He ticks the browser box on the account, signs in
once, and mail moves to the browser sign-in as well, with the weekly expiry while the project is
in testing. Route B keeps mail on the app password and adds a second, browser-only sign-in for
calendars, contacts and tasks. That is new code and a new control, and the control's letter has
to be chosen against the account editor's letters already in use. `AuthManager::get_valid_token`
reads tokens by account and provider and does not look at `use_oauth`
(`oauth.rs:799-830`), so route B needs no change to the mail path.

## 2. What 11-07.1 and 11-07.2 made complete here first

From `11-07.1-SUMMARY.md` (merged `fa20d04a`, 2026-09-19) and `11-07.2-SUMMARY.md` (merged
`2526b31f`) [VERIFIED: read]:

- A move, a delete (to the Trash or outright) and a copy within one account change this computer
  first: the row leaves (or for a copy, a marked copy appears), "Moved to Archive: subject" is
  shown and not spoken, a copy's line is spoken, and a row is written to `moves_waiting` keeping
  the folder and number the server still holds the message under.
- The server is told once in the background on the account's held session
  (`complete_here_then_tell_the_server`, `wx_app.rs:25626-25741`), and again before the first
  listing of every check and download (`replay_the_moves_that_were_waiting`), so a restart replays
  and a check does not bring the message back. A server not reached ends that account's check.
- A refusal comes back as `UIUpdate::MovePutBack`, which puts the row back and speaks the reason
  at High. A refusal for a message already where the move wanted it is read as done (the server
  is asked where it is only after it said no).
- Enter on a folder in the Move window is the move.
- Across accounts (11-07.2): the row moves at once, the bytes are kept here under a 25 MB ceiling,
  the fetch at one server and the append at the other run in the background and at the next check
  of either account, a restart finishes from the kept bytes without asking, and a refusal at
  either server puts the row back. A message over the ceiling, or of unknown size, takes the old
  server-first path with a spoken line. The question on the next start that ledger 191 describes
  was retired ("why_it_cannot_be_finished_from_here in place of the retired question").

The stored kinds, verbatim (`src/data/message_cache/moves_waiting.rs:93-104`):

```
Self::Move { into_folder_path } => ("move", Some(into_folder_path), None),
Self::DeleteToTrash { trash_path } => ("delete_to_trash", Some(trash_path), None),
Self::DeleteOutright => ("delete_outright", None, None),
Self::Copy { into_folder_path } => ("copy", Some(into_folder_path), None),
} => ("move_across", Some(into_folder_path), Some(to_account)),
} => ("copy_across", Some(into_folder_path), Some(to_account)),
```

A row filed here takes a number counted down from the top (`src/data/message_cache/messages.rs:14`:
`const FIRST_RESERVED_UID: u32 = u32::MAX;`) and `filed_here` 1, which is why his two waiting
deletes sit in the Trash at 4294967295 and 4294967294.

The replay's log lines, verbatim (`src/application/moves_waiting.rs:561`, `:567`, `:573`):

```
tracing::info!("A waiting move was replayed: {}", moved.spoken(into));
tracing::info!("A waiting copy was replayed: copied to {into}");
tracing::info!("A waiting delete was replayed: {}", deletion.spoken());
```

and the wait (`wx_app.rs:25721-25724`): `"The change made here waits for the next check of {}: {why}"`
with the account's name. None carries a subject.

## 3. What the profile can and cannot record for each line

| Line | What the log says today | What the database says | Gap |
|------|-------------------------|------------------------|-----|
| Copy within | "A waiting copy was replayed: copied to {folder}"; a wait line when not reached | A `copy` row while waiting; afterwards the copy settles under the number the destination holds it under, `filed_here` 0, and the original stays | None for the reading |
| Move within | "A waiting move was replayed: {moved}" | A `move` row while waiting; afterwards the row in the destination under a real number, `filed_here` 0, or dropped for the next read to bring | None |
| Delete | "A waiting delete was replayed: Moved to Trash" or the outright form | `delete_to_trash` or `delete_outright` rows | Gmail's Shift+Delete answer (section 6) |
| Copy across, move across | Only the "waits" line (`moves_waiting.rs:967-970`) and failures (`mail_across_accounts.rs:508`, `:569`); **nothing on success** | `copy_across` and `move_across` rows; `move_in_flight` holds the kept bytes | A success, the append's answer (landed under a new number or not), and the removal at the source write no line. Ledger 187's third question is about exactly that answer |
| Refusal and put-back | The spoken refusal at High as a `Speaking:` line, whose text may carry the subject | The row back at its source | The reader must match the template, never print the line |

So one small code change belongs before the crossing session: an info line for each of the
crossing's three steps and its outcome, with no subject, red as readings in
`tests/the_log_carries_what_a_report_needs.rs`.

## 4. How each proof is taken

### The shape every session shares

- **The build.** Built from `main` with `scripts/build-installer.sh` and installed by Pratik.
  The build string is read by the reader from the log's first lines, "Starting Wixen Mail
  v1.0.0-alpha.1+N.gHASH", so he never has to read it out.
- **Before the first session, once:** Settings, the Advanced tab, Logging, "Log level:"
  (choices `["Error", "Warn", "Info", "Debug", "Trace"]`, `wx_settings.rs:2688`), set to Debug,
  OK, then quit with `Ctrl+Q` and start again, because "the level is set up once when the program
  starts" (`wx_settings.rs:2712-2716`). The lines the proofs need are at info, so this only adds
  the debug lines from 11-04.
- **Messages made for the proof, never his own mail.** He sends himself one message per step
  from the program (`Ctrl+Shift+M`; sending is proven), with subjects that begin
  "Wixen proof", for example "Wixen proof copy 1". The reader may print subjects that begin with
  that prefix and no others, which keeps every other subject out of the output.
- **A folder made for the proof**, for example "Wixen proof", so a move or copy never lands in a
  label he uses.
- **Network off** means Windows' Airplane mode (Windows key, type "airplane", Enter, Space; or
  Settings, Network and internet, Airplane mode). His phone keeps its own connection, which is
  what the "another client changed it" step needs [ASSUMED: he has Gmail on a phone or on the web
  he can use with a screen reader].
- **The session ends with `Ctrl+Q`.** The reader copies the database only after the program has
  closed, so the write log is settled.
- **The reader** is a scratch Python script whose full source the plan carries, because a plan
  that names a scratch probe and leaves out its source leaves the executor to reinvent it. It copies
  `cache\message_cache.db` and its `-wal` and `-shm` files to the session scratchpad, opens the
  copy, reads the profile's log lines after the session's start, and prints: the build string;
  every `moves_waiting` and `move_in_flight` row by kind, time and folder; for each "Wixen proof"
  message, its folder, whether its number is reserved (above 4294967000) and `filed_here`; the
  replay, wait and refusal lines by template; and for #22, the new sync lines and the counts of
  `calendars` by `source_provider`, `calendar_events`, `contacts`, `tasks` and `sync_state`. It
  never prints a subject without the prefix, a body, an address beyond the domain, or a `Speaking:`
  line whole. It deletes its copy when it finishes.
- **The decision.** A line is proven when four things agree: what he heard (his words, quoted);
  the replay line in the log with no refusal and no put-back; the database after the session (no
  waiting row for the proof message, its row in the destination under a real number with
  `filed_here` 0, and for a copy the original still in place); and a fresh listing from the
  server after the replay, taken by `F5` on the source and on the destination folder before
  `Ctrl+Q`, so the database reflects what Gmail lists rather than what this computer did. A line
  where any of the four disagree is recorded as what happened, not as proven.
- **The record.** One comment per line on #63 (and on #22 for REAL-01), in the shape of the
  sending comment: "**Copy within one account: proven.** 2026-10-NN, build
  `1.0.0-alpha.1+N.gHASH`, his Gmail account (password sign-in, IMAP): what was done, his words,
  the log lines, what the database held, after a restart and with the network off." Drafted by
  the executor from the reader's output and his words; posted on his word, since a comment is
  outward. The ledger entries it settles are closed or amended in the same plan.

### Session 0, free, at the first start of the new build

The two `delete_to_trash` rows from 2026-09-20 replay at the first check. At least one, and probably
the one logged at 20:57:43, was already carried out at Gmail, because the database's last write
came before that log line. So Gmail will probably answer the replay of that one with a refusal,
the program asks where the message is, finds it in the Trash, and reads it as done. That is the
"already done" branch meeting a real server for the first time. The reader records the two lines
and the table afterwards; nobody has to do anything but start the program. Record it under the
delete line as evidence, not as the proof.

### Session 1: Gmail alone (about 30 minutes)

In the roadmap's order, after the #22 code plan has landed in the build:

1. **#22 (REAL-01).** Depending on his answer to question 1: Tools, Account Manager
   (`Ctrl+Shift+A`), the account, the browser sign-in, then Tools, "Sync Calen&dar", "Sync
   Co&ntacts" and "Sync &Tasks" (`wx_app.rs:7790-7802`), listening to each finish. Without a
   sign-in, each finish should now say why nothing could be asked; that sentence heard is
   itself a recorded result for #22. With one, the reader's counts decide: `calendars` with a
   Google `source_provider`, events, contacts and tasks above 0, `sync_state` rows, and the
   new log lines showing each request answered 200.
2. **Copy within one account.** On "Wixen proof copy 1", Action, Copy to (`Ctrl+Shift+Y`,
   Somewhere Else), arrow to the proof folder, Enter. He should hear one line, "Copied to ..."
   once. Then Airplane mode on, copy "Wixen proof copy 2", `Ctrl+Q`, Airplane mode off, start
   the program, `F9`. Then `F5` in the source and in the proof folder.
3. **Move within one account**, the same three ways: online; offline then `F9`; offline,
   `Ctrl+Q`, restart, `F9`. Then one with another client: offline, move "Wixen proof move 4",
   then on the phone delete or relabel that message, then online and `F9`. He says what he heard;
   the reader says what the replay answered.
4. **Delete.** `Delete` on "Wixen proof delete 1" online, then offline with a restart. Then
   `Shift+Delete` on "Wixen proof delete 2", whose answer at Gmail depends on his Gmail IMAP
   setting (section 6).
5. **Undo, if he wants it in the same session**: `Ctrl+Z` after a move and after a copy, which
   is ledger 625. Not on the copy of a message he cares about (section 6).

### Session 2: two accounts (about 30 minutes)

Needs a second account added (question 2). Then:

1. **Copy across**, then **move across**, each online, then offline with a restart. The
   destination's next check should bring the message in; nothing removed at the source until
   the destination has answered.
2. **Ledger 187's three questions:** a message of about 10 MB (a proof message with a large
   attachment, sent to himself first) copied across; the same proof message copied across into
   Gmail twice, which asks what Gmail does with an append carrying an identifier it already
   holds, and what it makes of an appended message at all; the reader reports both arrivals and
   the new crossing log lines.
3. **Ledger 547:** a message over 25 MB, which should say it goes now and take the old path.
4. **The delete line's extra ledgers.** 771 and 784's first half need an IMAP account that is
   not Gmail or Microsoft, because 13-44.6 leaves Gmail's and Microsoft's Trash to the provider
   (commit `c9a34e6a`: "Gmail and Microsoft left to their provider"). 784's second half and 787
   need a POP account with "Leave mail on the server" on. If the second account is a plain IMAP
   provider that also offers POP, it can be added a second time as POP for this step.

### What each line closes or answers

| Line | Ledgers it settles | Notes |
|------|--------------------|-------|
| Sending | 148 (said to close on 2026-09-18 on #63, still `open` in `.planning/WINDOWS.md`) | Already proven; the ledger was never closed |
| Copy within | 546 (copy part), 625 (if undo is tried) | |
| Move within | 546, 625 | "Replayed after a restart" and "another client changed it" are 546's |
| Delete | 376, 546 (delete part), 771, 784, 787 | 771, 784 and 787 need the second and third accounts |
| Copy across | 183, 187, 547 | 187's three questions |
| Move across | 187, 188, 191, 547 | 191's seven steps describe a question that 11-07.2 retired; 191 should be rewritten to the resume, not answered as written |

## 5. The gate: what "the default moves per path" can mean

`Allowed` has three answers, one of which covers every mail write
(`src/application/allowed.rs:57-89`, `:135-158`), verbatim:

```rust
pub const NOTHING: Self = Self { mail: false, personal_information: false, reading: true, };
pub const EVERYTHING: Self = Self { mail: true, personal_information: true, reading: true, };
pub const FOR_TESTING: Self = Self { mail: false, personal_information: true, reading: true, };
```

`data::config::default_allowed` answers `Allowed::FOR_TESTING` (`config.rs:638-640`), the
first-run screen starts on `Choice::TasksAndContacts`, which also maps to `FOR_TESTING`
(`first_run.rs:56`, `:62`), and the alpha page lists what the mail answer covers
(`docs/ALPHA_TESTING.md:137`): "Sending, deleting, moving, copying, marking read on the server,
filing a copy in Sent, sending read receipts, changing subscriptions, what your rules do to
arriving mail ..., and emptying an account's Trash". 66 lines under `src` read `.mail`.

So:

- **One switch cannot move per path.** Turning the mail default on after four proofs would also
  turn on flags, rules, Sent filing, read receipts, subscriptions and the scheduled Trash
  emptying for every new install, none of which phase 14 proves. Moving per path means splitting
  `mail` into finer answers, which changes the settings screen, the first-run choices, the
  account editor's boxes, `--allow`, and every reader of `.mail`; and more boxes is more for a
  screen reader user to read past (principle 8).
- **The default reaches new installs only.** His own settings already allow everything. What a
  move changes is what the next tester's first-run screen starts on.
- **The personal-information half is the one on by default, and it is the one #22 shows has
  never reached Google.** The first-run choice's explanation, "Changes to tasks, contacts and the
  calendar go up to your provider" (`first_run.rs:92-95`), is untrue for every Gmail account on
  a password sign-in and for every build without a Google client.
- **The words are wrong now, independent of the switch.** Sending was proven on 2026-09-18, and
  every surface still says it has never run: the settings warning (`allowed.rs:336-341`, shown at
  `wx_settings.rs:2348-2352`), the first-run introduction (`first_run.rs:135-138`) and its third
  choice (`:98-102`), the end of `--help` (`command_line.rs:147-150`, held by the test at
  `:597-598`), the alpha page (`ALPHA_TESTING.md:117-121` and `:295-297`), and two more the
  roadmap does not list: `UNDOING_AT_THE_SERVER_IS_EXPERIMENTAL` (`allowed.rs:431-433`) and
  `EMPTYING_THE_TRASH_IS_EXPERIMENTAL` (`:443-449`). `test_anything_that_writes_says_it_is_experimental`
  (`allowed.rs:727-749`) requires "real account" in two of them, so a rewording starts red there.

**Recommendation for the question to him (question 3):** the sentences change per path as each
line is proven, starting with sending now; the switch's default stays as it is until every write
it covers is proven or he says otherwise; and splitting the switch is a decision for after the
phase, with its cost stated. He may prefer a combination, such as one split, sending apart from
changing mail.

## Standard Stack

No library is added. Everything in this phase uses what is in the tree: `tracing` for the new
lines, the existing `reqwest`-based Google and Graph clients, `rusqlite` behind `MessageCache`,
and Python 3.14 (`C:\Users\prati\AppData\Local\Python\pythoncore-3.14-64\python.exe`, the
standard library's `sqlite3`) for the scratch reader.

## Package Legitimacy Audit

Not applicable: no package is installed by this phase. Packages removed: none. Flagged: none.

## Architecture Patterns

### The flow of one proof

```
Pratik, with NVDA                  Wixen Mail (new build)                  Gmail / second server
  key: Copy, Move, Delete  ──>  row changes here at once  ──> moves_waiting row
                                 line shown or spoken                │
                                 push once on held session ──────────┼──> answer
                                         │ not reached: "waits" line │
  Airplane mode off, F9  ──>  check: replay before first listing ────┼──> answer
                                 "A waiting X was replayed: ..."      │
                                 or MovePutBack, spoken at High       │
  F5 on source and destination ─> fresh listing from the server <────┘
  Ctrl+Q ──> program closed
                                         │
Scratch reader (Python) ──> copy of the database and the log ──> per-line verdict
                                         │
Executor drafts the comment ──> Pratik's word ──> comment on #63 / #22, ledger closed or amended
```

### Patterns to follow

- **A sentence for every reason nothing happened.** The sync's finish names why a provider was
  not asked; "0 created" is said only when the provider was asked and answered with nothing.
- **Log lines tested by reading the source**, as `tests/the_log_carries_what_a_report_needs.rs`
  does, with its lexical guard against subjects, bodies, passwords and tokens.
- **The program's own words in the steps.** Every step names the menu item, the key and the
  sentence he should hear, read from the build, not from memory.

### Anti-patterns to avoid

- **Treating the 2026-09-20 replays as the delete proof.** They are evidence: no ear, no restart
  variant, no network-off variant.
- **Reading a `Speaking:` line whole.** A refusal line can carry the subject.
- **Opening the live database.** Copy first, after `Ctrl+Q`.
- **Cleaning up a copy on Gmail by deleting it** (section 6).

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---------|-------------|-------------|-----|
| A Google sign-in for calendars | A second OAuth flow | `AuthManager::authorize` and `get_valid_token` (`oauth.rs:749`, `:799`) | It already stores and refreshes per account and provider, and the scopes already cover all three |
| Knowing which provider an account is | Another match on the address | `provider_of(account)` (`wx_account_manager.rs:3002`) and 13-44.5's `WhoRunsTheMail` | One check, already used by the sign-in and the Trash emptying |
| Proving a log line exists | A capturing subscriber in tests | A reading in `tests/the_log_carries_what_a_report_needs.rs` | The project has no log-capture pattern and one reading file already guards every `tracing::` call |
| Reading the profile | Bash or PowerShell | Python | The shell's view of his AppData is a stale July copy (memory note) |

## Runtime State Inventory

Not a rename or migration phase. The runtime state this phase meets:

| Category | Items found | Action |
|----------|-------------|--------|
| Stored data | 2 `moves_waiting` rows (`delete_to_trash`) from 2026-09-20 that replay at the next start | Read at Session 0; nothing to migrate |
| Live service config | His Gmail account's IMAP setting "When a message is marked as deleted and expunged from the last visible IMAP folder" (unknown value); his Google Cloud project's test-user list and client (unknown) | Questions 1 and 4 |
| OS-registered state | The credential store holds his app password under the account's id | None; never read |
| Secrets and environment | No `oauth.toml` in the profile; no `WIXEN_*` variables anywhere | Route A needs a client placed, question 1 |
| Build artifacts | The installed build of 2026-09-20 carries no Google client and none of phases 12 to 13.1 | A new build is installed before Session 1 |

## Common Pitfalls

### Pitfall 1: Deleting a copy on Gmail deletes the original

**What goes wrong:** on Gmail a folder is a label and a copy is a second label on the same
message. A delete here replays as a move into `[Gmail]/Trash` (`imap.rs:1647-1652`), and moving a
message to the Trash removes every label, so the original leaves the Inbox too.
**Source:** "When you delete a message from the Inbox, the message is moved to Gmail/Trash, causing
it to be removed from ALL other labels/folders as well" [CITED: support.postbox-inc.com, Delete
Behavior for Gmail Accounts; MEDIUM].
**How to avoid:** proof messages only; clean up with Undo or not at all; and record what
the copy's removal does at Gmail as part of the copy line (ledger 625 asks the same of an undone
copy).

### Pitfall 2: Delete Permanently on Gmail archives by default

**What goes wrong:** `Shift+Delete` sends a `\Deleted` flag and `UID EXPUNGE`
(`imap.rs:1655-1670`) and the program reports the message removed. Gmail's IMAP setting decides
what an expunge does, and its default is to archive the message, which stays in All Mail
[CITED: aiemaily.com and limilabs.com; the setting is the Gmail API's `ImapSettings`
expunge behaviour, developers.google.com; MEDIUM].
**How to avoid:** ask Pratik which value his account has before the step, and record the
sentence he heard beside what Gmail kept. If the sentence is untrue on Gmail's default, that is a
finding for a ledger entry, not a failed proof.

### Pitfall 3: Too many connections

**What goes wrong:** on 2026-09-19 Gmail refused 118 sign-ins with "Too many simultaneous
connections". A replay that cannot get a session reads as not reached and waits, which looks
like the network-off case when the network was on.
**How to avoid:** the reader reports the connection refusals beside each replay; a session that
meets them says so and is taken again, not recorded as a result.

### Pitfall 4: The Trash that cannot be read

**What goes wrong:** on 2026-09-18 and 2026-09-20 nearly every check spoke "Some folders could
not be read. Trash: The connection to the mail server was lost ...". The delete line is read from
the Trash.
**How to avoid:** read the first log of the new build for it before Session 1; if it is still
there, it is a defect to fix before the delete line, not a step to work around.

### Pitfall 5: The log level

**What goes wrong:** his stored `"log_level": "info"` wins over the version's debug default, and
a change takes effect only at the next start.
**How to avoid:** the new lines are written at info; the debug step is done once, with a restart,
before the first session.

### Pitfall 6: The roadmap's "#22 first because there is a log to read"

**What goes wrong:** a session spent pressing Refresh on the current build produces no line to
read.
**How to avoid:** #22's code plan comes first; the session for #22 runs on the build that carries
it.

### Pitfall 7: A stale sentence on the shortcuts page

`docs/KEYBOARD_SHORTCUTS.md:651-652` says "A move to a folder on another account still waits for
both servers before the row leaves", which stopped being true at 11-07.2. Session 2's steps must
not be written from it; the line wants a dated correction in the proof-kit plan.

### Pitfall 8: Tests write into his crash log

`logs\crash.log` in his profile holds 4,546 lines, nearly all `--show-page was given "...", which
is not a page. Nothing was opened.` from 2026-09-22 to 2026-10-04, three per test run
(`src/presentation/page_window.rs:394`). A test reaches the real profile, and a real crash during
a proof session would be buried in them. Not ledgered as of this reading; a ledger entry and a
temporary folder for that test are the fix.

## Code Examples

### The decision #22's fix makes testable (shape only)

```rust
// Illustrative; names are the executor's. The point is a pure answer the sync asks first.
enum MayAskGoogle {
    Yes,
    NoClientInThisBuild,
    SignsInWithAPassword,
    BrowserSignInExpired,
}
```

### Reading a proof without opening the live database (Python, scratch)

```python
# Illustrative. Run after Ctrl+Q. Never prints a subject without the proof prefix.
import os, shutil, sqlite3
root = os.path.join(os.environ["LOCALAPPDATA"], "wixen-mail")
scratch = os.environ["PROOF_SCRATCH"]          # the session scratchpad
for name in ("message_cache.db", "message_cache.db-wal", "message_cache.db-shm"):
    shutil.copy2(os.path.join(root, "cache", name), os.path.join(scratch, name))
db = sqlite3.connect(os.path.join(scratch, "message_cache.db"))
for row in db.execute("select kind, from_folder_path, into_folder_path, asked_at from moves_waiting"):
    print(row)
for row in db.execute(
    "select f.path, m.uid > 4294967000, m.filed_here, m.subject from messages m "
    "join folders f on f.id = m.folder_id where m.subject like 'Wixen proof%'"):
    print(row)
```

The column names in that query were read from the profile's own tables on 2026-10-04:
`moves_waiting` (`message_row_id, account_id, from_folder_path, uid, kind, into_folder_path,
asked_at, to_account_id, to_account_name`), `messages` (`id, uid, folder_id, ..., subject, ...,
filed_here, ...`), `folders` (`id, account_id, name, path, ...`) [VERIFIED: `pragma table_info` on the copy].

## State of the Art

| Old approach | Current approach | When | Impact |
|--------------|------------------|------|--------|
| Move and delete waited for the server | Made here first, replayed from `moves_waiting` | 11-07.1, 2026-09-19 | The proofs must include restart and network-off replays |
| Crossing waited for both servers, asked a question at the next start | Made here first, finished from kept bytes without asking | 11-07.2, 2026-09-19 | Ledger 191's question no longer exists |
| Log level fixed at info | Default follows the version, stored level wins | 11-04, 2026-09-18 | His profile still logs at info |

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | Google's Calendar, People and Tasks REST interfaces cannot be reached with an IMAP app password | 1 | Low: CalDAV and CardDAV are quoted; the REST interfaces take bearer tokens in the code itself |
| A2 | Pratik has a Google Cloud project with a client and a tester list | 1, question 1 | Route A cannot start without one |
| A3 | He can use Gmail on a phone or the web with a screen reader for the "another client" step | 4 | The step needs a different second client |
| A4 | The 20:57:43 replay was carried out at Gmail and its row was not cleared | 4, Session 0 | Session 0 then reads two ordinary replays, which is still evidence |
| A5 | Gmail's expunge default is archive | 6, pitfall 2 | Read from secondary sources; his own setting is what matters |
| A6 | Airplane mode is the simplest network-off step for him | 4 | He may prefer turning Wi-Fi off |

## Open Questions

1. **How should Gmail calendars, contacts and tasks sign in?** Known: a password can never reach
   them, his builds carry no Google client, and the browser box exists. Unclear: whether he wants
   mail moved to the browser sign-in (one sign-in, weekly expiry while the project is in testing),
   or mail kept on the app password with a second sign-in for the rest (new control); and whether
   the client goes into his profile only or into every build. Recommendation: ask (question 1).
2. **Which second account, and a POP account?** Needed for both crossings and for ledgers 771,
   784 and 787. Recommendation: ask (question 2).
3. **What the gate's default does** (question 3, section 5).
4. **His Gmail IMAP expunge setting** (question 4).
5. **Whether the Trash read failures and the connection limit still happen** on the new build:
   read from its first log before Session 1.

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Python | The reader | Yes | 3.14 (`pythoncore-3.14-64`) | None needed |
| `gh` | Reading and commenting on #22 and #63 | Yes | Used this session | None |
| Installed Wixen Mail | Sessions | Yes, but the 2026-09-20 build | `1.0.0-alpha.1` | A new build from `scripts/build-installer.sh` |
| Google OAuth client | REAL-01 with a sign-in | No | None | The sentence saying why nothing comes is still a recorded result |
| Second IMAP account, POP account | Crossings, 771, 784, 787 | No (one account in the profile) | None | Those lines wait on question 2 |
| A second mail client | The "changed meanwhile" step | Unknown | None | Ask |

## Validation Architecture

### Test framework

| Property | Value |
|----------|-------|
| Framework | `cargo test`, unit tests beside the code and integration targets under `tests/` |
| Config file | `Cargo.toml`; `scripts/check.sh` |
| Quick run | `cargo test --lib application::calendar::` (one module per command; `cargo test` takes one `--lib`, join several with `&&`) |
| Full suite | `bash scripts/check.sh all`, once, by the phase's closing plan |

### Requirements to tests

| Req | Behaviour | Type | Command | Exists? |
|-----|-----------|------|---------|---------|
| REAL-01 | A sync that cannot ask Google says why, not "0 created" | unit | `cargo test --lib application::calendar::` and the contacts and tasks modules | No: Wave 0 |
| REAL-01 | The provider asked follows the account | unit | as above | No |
| REAL-01 | Each pass writes its request and answer at info, naming no subject, body, password or token | source reading | `cargo test --test the_log_carries_what_a_report_needs` | File exists; cases do not |
| REAL-01 | A held event is counted once | unit or reading | `cargo test --lib presentation::wx_app::` | No |
| REAL-02 | A crossing's steps write their outcome at info | source reading | `cargo test --test the_log_carries_what_a_report_needs` | No |
| REAL-02 | Warnings say sending was proven | unit | `cargo test --lib application::allowed::` and `presentation::first_run::` and `presentation::command_line::` | Tests exist and pin the old words; they go red first |
| REAL-01, REAL-02 | Proven against his account | manual, his ear and his account | The reader, then a comment on #22 or #63 | Manual only: a real account cannot be reached from a test, by the project's rule |

### Sampling rate

- Per commit: the hook's scoped run.
- Per merge: the hook (`git merge --no-ff`).
- Phase gate: `scripts/check.sh all` by hand in the closing plan, output to a file, exit status
  read directly.

### Wave 0 gaps

- The decision type and its cases in the three sync modules.
- The new readings in `tests/the_log_carries_what_a_report_needs.rs`.
- Guard records for the new tests, and re-measurement of the records naming `wx_app.rs`,
  `allowed.rs`, `first_run.rs` and `command_line.rs`, by both readings of the register (the
  files they name, and the anchors inside any file a plan edits).

## Security Domain

| ASVS category | Applies | Control |
|---------------|---------|---------|
| V2 Authentication | Yes | OAuth 2.0 installed-app flow with loopback redirect, as `service::oauth` already does; tokens only in the credential store |
| V3 Session management | Yes | Refresh through `get_valid_token`; expiry said, not hidden |
| V5 Input validation | Yes | Google's answers parsed into typed values at the service boundary, as today |
| V6 Cryptography | No new code | |
| V7 Logging | Yes | New lines carry method, path without query values, status and reason; the lexical guard refuses subjects, bodies, passwords and tokens |

| Threat | STRIDE | Mitigation |
|--------|--------|------------|
| A token or address written to the log by a new line | Information disclosure | The existing lexical guard; path without query values |
| The reader printing his mail | Information disclosure | The prefix rule; never print `Speaking:` lines whole; delete the copy |
| A client secret in a public build | Information disclosure | Google treats installed-app secrets as not confidential [CITED]; still Pratik's call (question 1) |
| A proof deleting real mail | Tampering | Proof messages only; pitfalls 1 and 2 read before the delete line |

## Project Constraints (from CLAUDE.md)

- Red, green, refactor on every behaviour change; `workflow.tdd_mode` is true; a red commit names
  its failing tests with `Fails-until-green:` and is made on a branch.
- Guardrail 1: the #22 fix is done when Refresh reaches the new sentence in the running program,
  not when its tests pass.
- Guardrail 3: no sentence may claim a path works that has not been proven; the warning rewording
  follows the proofs, not the other way round.
- Guardrail 7: a comment on an issue and any push are made on purpose, on his word.
- Guardrail 9: the sentence for a password-signed Gmail account names the cause plainly.
- Secrets stay out of the tree and the database; never log a token, password or body.
- A setting nobody can reach is not a setting; a new sign-in control is reachable from the
  account editor, with its mnemonic letter chosen against the letters that dialog already uses.
- Every user-visible change gets a `docs/changelog.md` entry; every shortcut change is in
  `docs/KEYBOARD_SHORTCUTS.md` in the same commit.
- Plans that write `guards/guards.toml`, `docs/changelog.md` and `.planning/WINDOWS.md` share a
  file, so they go in separate waves.
- The phase's closing plan runs `scripts/check.sh all` once, by hand, before its merge.
- Profile reads by Python only; the reader prints nothing private.
- Do not relay "not heard yet" lines to Pratik; raise decisions.

## Proposed plans

| Plan | What | Depends on |
|------|------|------------|
| 14-01 | #22 in code: why nothing was asked, provider by account, request lines at info, the double count, what account creation says | Nothing; first |
| 14-02 | The Google sign-in route for calendars (route A: client in profile or builds; route B: a second sign-in) and, if wanted, calendars beyond the primary | Question 1 |
| 14-03 | The proof kit: crossing log lines, the shortcuts page's stale line, the steps pages, the reader carried in the plan, the crash log test moved to a temporary folder | 14-01 |
| 14-04 | Session 0 and Session 1 with Pratik: #22, copy, move, delete on Gmail; comments on #22 and #63; sending's warning reworded and ledger 148 closed | 14-01, 14-03, a build |
| 14-05 | Session 2: crossings, 187's three questions, 547, and the delete line's 771, 784, 787 | Question 2, 14-04 |
| 14-06 | The gate, on his word, and the phase's closing gate | Question 3, 14-04, 14-05 |

## Sources

### Primary (HIGH)

- The tree at `01ef4589`: `src/application/allowed.rs`, `src/data/config.rs`,
  `src/presentation/first_run.rs`, `src/presentation/command_line.rs`,
  `src/presentation/wx_app.rs`, `src/presentation/wx_settings.rs`,
  `src/presentation/wx_account_manager.rs`, `src/service/oauth.rs`,
  `src/service/oauth_credentials.rs`, `src/service/google_api.rs`,
  `src/application/calendar.rs`, `src/application/moves_waiting.rs`,
  `src/data/message_cache/moves_waiting.rs`, `src/data/message_cache/messages.rs`,
  `src/service/protocols/imap.rs`, `src/main.rs`, `src/common/paths.rs`.
- `.planning/ROADMAP.md:1796-1828`, `.planning/REQUIREMENTS.md:7198-7240`,
  `.planning/WINDOWS.md` entries 148, 183, 187, 188, 191, 376, 546, 547, 625, 771, 784, 787.
- `11-07.1-SUMMARY.md`, `11-07.2-SUMMARY.md`, `11-04-SUMMARY.md`, phase 13 and 13.1 READMEs.
- `gh issue view 22`, `63`, `66`, `86`, with comments.
- Pratik's profile, read with Python on 2026-10-04: `config\app_config.json`, a copy of
  `cache\message_cache.db`, `logs\*.log`, `logs\crash.log`; the installed binary by byte search.
- Google: CalDAV guide, CardDAV page, OAuth 2.0 for iOS and desktop apps.

### Secondary (MEDIUM)

- Gmail delete behaviour: support.postbox-inc.com, "Delete Behavior for Gmail Accounts".
- Gmail expunge setting: aiemaily.com, "Gmail Delete Only Archives?"; limilabs.com, "Delete
  email permanently in Gmail"; the Gmail API `ImapSettings` reference.

## Metadata

**Confidence breakdown:**

- #22's cause: HIGH. The profile, the binary, the August log and the code agree.
- What the log can and cannot show: HIGH. Counted over every line.
- The write paths' shape: HIGH. Read from the code and the summaries.
- Gmail's behaviour on a copy's delete and on expunge: MEDIUM. Secondary sources, to be settled
  by the proofs themselves.
- Session timings: LOW. Estimates, not measurements.

**Research date:** 2026-10-04. **Valid until:** the next build Pratik installs, since Session 0's
reading and the log findings are about the 2026-09-20 build and profile.
