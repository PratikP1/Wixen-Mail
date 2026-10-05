# Phase 14, research 1: what waits on a real account

**Researched:** 2026-10-04, against `main` at `01ef4589`, version `1.0.0-alpha.1`
**Domain:** the ledger's open entries that only a real account, server, organiser or correspondent can close
**Confidence:** HIGH for what the ledger and the code say; MEDIUM for the #22 cause, which is read and not yet run

## Summary

`.planning/WINDOWS.md` holds 806 entries, 692 open and 114 fixed, by its own frontmatter
and by a count of both halves. The table and the JSON block agree entry for entry: same ids,
same status, same words, checked by script. 46 open entries say "phase 14" in words, which is
the "about 90" in the brief counted once per half (`grep -c -i "phase 14"` answers 92). Read
widely, about 150 open entries wait on something outside this computer: a real account, a real
server, a second account, a real organiser or a real correspondent. They fall into seventeen
groups below.

Three findings change what the phase should do first, and none of them is in the ledger.

1. **Pratik's builds cannot reach Google's calendars, contacts or tasks at all, and say nothing
   about it.** The Calendar, Contacts and Tasks syncs try Google only when the program has
   Google sign-in keys (`credentials_for("gmail")`), and skip the whole Google half silently when
   it has none. No `oauth.toml` exists where the program reads it, no `WIXEN_` variable is set,
   and no build script sets the compile-time default. His one account is also set to sign in
   with a password (`use_oauth` is 0), so even with keys there is no Google token for it. That
   matches every reading taken for #22: no Google calendar row, `sync_state` empty, and not one
   "Calendar sync error" line in three days of logs. REAL-01 cannot be proved until the program
   has Google sign-in keys and his account signs in through the browser. The same keys are the
   prerequisite for every Microsoft group as well.
2. **His profile already holds real-server evidence that no ledger entry has been told about.**
   Five deletes replayed against Gmail on 2026-09-20 and logged "Moved to Trash", on builds that
   carry 11-07.1 and 11-07.2. Gmail refused 118 sign-ins on 2026-09-19 with "Too many
   simultaneous connections" (ledgers 67, 525). The download of everything met about 940
   "closed the connection while opening the folder" drops over three days (ledgers 11, 72, 523).
   Two deletes are still waiting in his queue and go to Gmail at the next start.
3. **His log is at Info, not Debug.** The stored level wins over the alpha default, so the
   debug lines 11-04 added are not being written on his machine. Several proofs below read lines
   that exist only at Debug or do not exist at all.

**Primary recommendation:** open the phase with a small plan that settles the sign-in keys and
the log (both Pratik's decisions plus test-first logging lines), then take #22 on his account,
then the four write paths in the order copy, move, delete, using the evidence already in his
profile. Everything that needs a second person or a second provider waits behind those.

## How this was read

- The ledger was read with Python, both halves parsed and compared, then three passes over the
  open entries: the words "phase 14" or "REAL-01/02"; real account, server, provider, organiser,
  correspondent, recipient, "his account"; then a wider pass for Gmail, Google, Microsoft, Graph,
  CalDAV, CardDAV, IMAP server, SMTP, POP, OAuth, token, push, loopback, stand-in, Thunderbird.
  Every hit was read in full and placed by hand. Entries that only need the tester's ear and say
  "nothing needs a real account" were left out.
- `gh issue view 22 --comments` and `gh issue view 63 --comments`, read 2026-10-04.
- Pratik's profile, read with Python only, read-only: the three logs of 2026-09-18 to 2026-09-20
  tallied by line shape with numbers and addresses masked; `config/app_config.json` for the log
  level and the Allow Changes answers; and a copy of `message_cache.db` made into the scratchpad
  and queried for row counts and the account's shape. No message text, subject, address beyond
  the domain, password or token was read or printed. The profile's newest log is 2026-09-20; the
  database was last written 2026-09-20 16:56 local, so the program has not run on his profile
  since then.
- The code was read at the lines cited. Nothing was run.

## The prerequisite no entry names: sign-in keys and a browser sign-in

Every Google and Microsoft proof needs two things the ledger never says, because each entry
describes what is unproven rather than what proving it needs.

| Fact | Where it was read |
|---|---|
| The Calendar sync tries Google inside `if let Some(gmail_creds) = crate::service::oauth_credentials::credentials_for("gmail") {`, and the Contacts sync the same | [VERIFIED: src/presentation/wx_app.rs:31203 and :30867] |
| The Tasks sync's own comment: "an account that is signed in to neither costs nothing here: no credentials means the branch is skipped" | [VERIFIED: src/presentation/wx_app.rs:30949-30951] |
| `credentials_for` reads, in order, `WIXEN_GMAIL_CLIENT_ID` and `WIXEN_GMAIL_CLIENT_SECRET`, then `oauth.toml`, then `option_env!("WIXEN_GMAIL_CLIENT_ID_DEFAULT")` | [VERIFIED: src/service/oauth_credentials.rs:107-123] |
| `oauth.toml` is read from `self.config_dir().join("oauth.toml")` | [VERIFIED: src/common/paths.rs:134-136] |
| His `%LOCALAPPDATA%\wixen-mail\config` holds `app_config.json` and nothing else; no `WIXEN_` variable is set in this session's environment; no file under `scripts/`, `.github/` or `build.rs` names `CLIENT_ID_DEFAULT` | [VERIFIED: Python `os.listdir` and `os.environ`, and grep, 2026-10-04] |
| The installing guide: "`oauth.toml` holds the sign-in keys this build was made with. It says nothing about you, and a build made without one cannot offer the browser sign-in at all." | [VERIFIED: docs/installing.md:90-91] |
| A new Gmail account defaults to a password: `oauth_is_default` answers true only for `"outlook.com" \| "hotmail.com" \| "live.com" \| "msn.com"` | [VERIFIED: src/data/account.rs:154-166] |
| His account row: provider `Gmail`, protocol `imap`, `use_oauth` 0, created `2026-09-18T11:52:23Z`, server `imap.gmail.com` | [VERIFIED: query of a scratchpad copy of his database, 2026-10-04] |

So on his machine the Google half of each personal-information sync is skipped before it asks
anything, and nothing is logged for the skip. #63's comment of 2026-09-18 says sending was
proven over "OAuth"; the account row says password sign-in. The row is the later fact; which one
held at 12:37 that day cannot be told from here. **Pratik's question, not a guess:** whether the
send went through a browser sign-in or an app password.

What the keys need is outside the repository. Google: a Cloud project with a Desktop OAuth client
and the Gmail, Calendar, Contacts and Tasks scopes. A project left in Testing mode issues refresh
tokens that expire after seven days for anything beyond basic profile scopes, and caps test users
at 100 [CITED: developers.google.com/identity/protocols/oauth2]. That is enough for one tester
re-signing weekly, and it would show up in the proofs as a sign-in failing a week after it
worked. Microsoft: an app registration for Graph. Whether Pratik already has either is his to
say.

## What his profile already says

Read 2026-10-04 from the logs of 2026-09-18, 19 and 20. Counts are of log lines.

| Line, as the program wrote it (numbers masked) | Count | Days | Bears on |
|---|---|---|---|
| `Logging initialized at level: Info` | 11 | all | every proof that reads a Debug line |
| `A waiting delete was replayed: Moved to Trash` | 5 | 2026-09-20, builds `+321.g4a09bfc2` and `+351.g5f363255`, both carrying `fa20d04a` and `2526b31f` | REAL-02's delete line; ledgers 376, 546, 625 |
| `A move of message N is still waiting: the server could not be reached` | 3 | 2026-09-20 | the queue; ledger 546 |
| `Email sent successfully` | 2, one send | 2026-09-18 12:37Z, build `+149.g744d05ef` | the send line, already proven; ledger 148 |
| `Signing in again after a dropped connection did not work: ... "Too many simultaneous connections. (Failure)"` | 118 | 2026-09-19, build `+149.g744d05ef` | ledgers 64, 67, 525 |
| `Could not watch the inbox of ...: ... Too many simultaneous connections` | 2 | 2026-09-19 | ledgers 65, 525 |
| `Could not fetch the text of message N: Network error: The mail server closed the connection while opening the folder` | 932 | all three | ledgers 11, 72, 523 |
| `The download will be tried again in N seconds, after N failure(s) in a row` | 321 | all three | ledgers 11, 72, 523 |
| `The text download for ... stopped after N messages: the connection was lost` | 139 | 19 and 20 | ledgers 11, 523 |
| `An arriving message joined N conversations into one, moving N messages` | 20,362 | all three | ledgers 60, 549 |
| `Calendar loaded 2 update(s)` then `Speaking: 0 calendar events loaded` | 4 | 18 and 20 | #22: the module opened from the cache; no sync line follows |
| `Calendar sync error`, `Contacts sync error`, `Task sync` | 0 | none | #22: no Google half ran, or none was asked for |

And from the database copy: `calendars` 2 rows, both `source_provider` `local`; `calendar_events`
0; `contacts` 0; `tasks` 0; `sync_state` 0; `messages` 99,014; `moves_waiting` 2, both
`delete_to_trash` from `INBOX` into `[Gmail]/Trash`, asked 2026-09-20 at 20:56Z. Allow Changes:
`{"mail": true, "personal_information": true, "reading": true}`, and no per-account answer.

Two consequences for the planner. The two waiting deletes are Pratik's own and will reach Gmail
at the next start, which is worth telling him before he starts a build. And
`DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL` still says the download "has never been run against a
real account" [VERIFIED: src/application/allowed.rs:374-379], which his logs show stopped being
true on 2026-09-18. Gmail did what that sentence warns of: it closed connections and refused new
ones. That sentence is one of the warning surfaces this phase rewords.

## Can the program log what a proof reads

Logging is one target, `wixen_mail` at the chosen level, written to
`logs/wixen-mail.<date>.log` [VERIFIED: src/common/logging.rs:75-77, :125-130]. The level a fresh
profile starts at is Debug under alpha or beta, and "A profile that already holds a level keeps
it" [VERIFIED: src/common/logging.rs:58-60]. His holds `info` [VERIFIED: app_config.json
`log_level`]. Spoken lines are written as `Speaking: ...` at Info, and message content by length
only [VERIFIED: src/presentation/accessibility.rs:436-452].

| What a proof reads | Written today? | Where |
|---|---|---|
| A move, copy or delete within one account reaching the server | Yes, at Info, on success only: `A waiting move was replayed`, `A waiting copy was replayed: copied to {into}`, `A waiting delete was replayed` | [VERIFIED: src/application/moves_waiting.rs:561, :567, :573]. Since 11-07.1 every such change goes through the replay, so these are the success lines for the online case too |
| A move or copy across two accounts reaching both servers | Failure only: `A move of message {} to another account waits: {why}`. A finished crossing writes no line | [VERIFIED: src/application/moves_waiting.rs:907-940, :967] |
| A send | Yes: `Sending email from` (address masked) and `Email sent successfully` | [VERIFIED: src/service/protocols/smtp.rs:588-610] |
| The Google or Microsoft calendar or contacts sync's request and Google's answer | No. Errors only, as `Calendar sync error: ...` and `Contacts sync error: ...`; a skip for missing keys writes nothing; a success writes its counts only when results are spoken | [VERIFIED: src/presentation/wx_app.rs:23687, :23800, :18701-18707] |
| What an IMAP server answered a STORE, a label or a junk move | Partly: failures are said; success is a status line, spoken and so logged as `Speaking:` only when spoken | read, not traced per command |
| Gmail's connection limit and drops | Yes, at Info and Warn, as above | his logs |
| A Trash emptying | Failures at Warn; the once-a-day sentence is spoken | [VERIFIED: src/application/emptying_the_trash.rs:233-241] |
| Free/busy, directory, people search | Failures only | [VERIFIED: src/service/free_busy.rs:1144, src/service/directory.rs:762] |

What follows for the plans: #22 needs lines added before Pratik's run, test-first: one when a
module's sync skips a provider for want of keys or a sign-in, one per provider request with its
HTTP status and the count it brought, and the counts at sync end whether or not they are spoken.
A cross-account crossing needs a success line. And Pratik is asked to set the log to Debug for the
phase, or the plan makes the lines it needs Info.

## The groups

**Legend for the last column.** *Sitting*: his account only has to be there while the build runs,
or his profile is read with his agreement. *Hands*: he signs in, sends, acts, or reads the result
in another client. *Second account*: a second mail account of his. *Another person*: a real
organiser, correspondent, colleague or recipient who is not him, or a second identity he
controls. *Keys*: needs the sign-in keys above first.

### 1. Sending (REAL-02's first line, proven 2026-09-18)

| Entry | What it waits for | Proof needs | Logged? | Needs |
|---|---|---|---|---|
| 148 | A message leaving after a hold meeting a real server | Already met: #63 says "Ledger 148 ... closes on it", and the ledger still holds it open | Yes | Nothing: close it, citing #63's comment |
| 151 | A scheduled message leaving hours later with the program left running | Send Later set an hour ahead, the program left open, arrival read at the recipient | Partly: the send line, not the timer | Hands |
| 140 | A decorative picture's empty alt surviving to Gmail or Outlook | A message with a decorative picture sent to an address of his read in Gmail's web reader | No | Hands, a second mailbox to read it in |
| 590 | A Send Feedback report reaching support@ and security@ | A report sent; arrival seen in both mailboxes | Yes, the send | Hands, and both mailboxes, due by public testing |
| 376 | Allow Changes' per-account answers holding back real writes | One account unticked, a delete tried, nothing at the server | Refusal is said | Hands |

### 2. Copy and move within one account and across two (REAL-02)

| Entry | What it waits for | Proof needs | Logged? | Needs |
|---|---|---|---|---|
| 546 | A move and a copy within the account at a real server, online, offline, after a restart, and with another client changing the message meanwhile | #63's re-take list for 11-07.1 | Yes, the replay lines; offline and restart are visible as `still waiting` then `replayed` | Hands; another client for the changed-meanwhile case |
| 547 | The same across two accounts, a message over 25 MB, a refusal at either server | #63's re-take list for 11-07.2 | Failure only; success writes nothing | Hands, second account |
| 183 | APPEND's internal date, keyword lists, byte-for-byte arrival, a slow append | A copy across two accounts compared in both providers' web readers | No | Hands, second account |
| 187 | Ten-megabyte uploads; Gmail treating an appended message as a label; an identifier the destination already holds | Three crossings, one per question | No | Hands, second account |
| 188 | A dropped connection mid-append arriving as one error | Network pulled during a large crossing | Failure line | Hands, second account |
| 191 | A crossing killed mid-way and restarted, and the question on the next start heard | 04.1-04-SUMMARY's seven steps | Partly | Hands, second account, his ear |
| 625 | Undo of a move, copy or delete reaching the server | Undo after the replay line, read back in Gmail | Yes, replay lines | Hands |
| 674 | The whole-message fetch of an earlier download, and from a second account under All Inboxes | Open an old message with attachments in each account | No | Sitting for the first, second account for the second |
| 19, 32, 563 | A saved search narrowed to a real folder; under two accounts; All Inboxes with two accounts | Run each and read the rows | No | Sitting (19); second account (32, 563) |

### 3. Delete and Trash emptying (REAL-02's delete line, with 771, 784, 787 named for it)

| Entry | What it waits for | Proof needs | Logged? | Needs |
|---|---|---|---|---|
| 376, 546 | A delete reaching the server | Already partly met: five `Moved to Trash` replays on 2026-09-20. Missing: the message read in Gmail's Trash from the web, and the two waiting deletes replayed | Yes | Hands, a readback in Gmail |
| 771 | Empty the Trash after 15 or 30 days at a real IMAP server with and without UIDPLUS; a refusal; a drop | Gmail has UIDPLUS; "without" needs another provider | Failures only; the day's sentence when spoken | Hands, and a second, non-Gmail IMAP account. The setting's line says a Gmail account gets "the line in its place" (ledger 770), so whether Gmail is offered emptying at all is read first |
| 784 | Emptying as the program closes within five seconds; a slow server; a real POP account with Leave on Server | As written | Failures only | Hands, a POP account |
| 787 | A POP message taken off this computer not downloaded again, and still leaving the server on its day | A real POP server and several days | No | Hands, a POP account, days of waiting |

### 4. The queue: replay, refusal, restart, marks

| Entry | What it waits for | Proof needs | Logged? | Needs |
|---|---|---|---|---|
| 88 | Which errors a real provider gives an expired token against a wrong password | A deliberately wrong app password, then a corrected one | Yes, the sign-in error words | Hands |
| 623 | Undo of a mark, star or label at a real server | Mark, undo, read in Gmail | Partly | Hands |
| 686, 687 | Several actions over a set reaching the server in order (marks before the move) | The server's own record of commands, which no Gmail client shows | No; 687 says no harness exists | Hands; the order can only be inferred from the end state on Gmail |
| 87 | A label change put back when the push fails | A known limitation, not a proof; a real failure shows it | No | Sitting |
| 216 | "has not reached the account yet" followed by the account receiving it | Part of the personal information proofs (group 6) | No | Keys |

### 5. Mail sync at a real provider (Gmail's behaviour)

| Entry | What it waits for | Proof needs | Logged? | Needs |
|---|---|---|---|---|
| 11, 72, 523 | Whether Gmail permits, throttles or drops the download of everything | Largely answered already by his logs: about 940 drops "while opening the folder", 321 waits, 139 text runs stopped. Owed: write the answer onto the entries and reword the sentence | Yes | Sitting; the reading is done |
| 64, 65, 67, 525 | Reconnect after a drop, idle session drops, the connection budget, a watch per account | Partly answered: 118 refused re-sign-ins and 2 refused watches on 2026-09-19 with Gmail's "Too many simultaneous connections". Owed: whether the build after `+149` still meets it, and how long Gmail holds IDLE | Yes | Sitting, a day of running |
| 56, 549, 548 | X-GM-MSGID and X-GM-THRID on a live account, All Mail, threads matching Gmail's own | Compare a few threads with Gmail's web view | Partly: 20,362 merge lines | Sitting, his eye or ear on Gmail's web reader |
| 60 | Real clients' reply headers and the cost of the first open | His logs show merges at scale; the timing is owed | Partly | Sitting |
| 70, 71 | Whether any provider grants CONDSTORE or QRESYNC | Gmail never does by `imap/abilities.rs`; needs a Fastmail or Dovecot account | No: capabilities are not logged | A second, non-Gmail account |
| 52 | A real server renumbering a folder | Happens or does not; cannot be arranged on Gmail | Spoken, so logged when it happens | Sitting, by luck |
| 533 | Whether his Gmail lists All Mail (his folder list holds none) | Gmail's "Show in IMAP" setting decides | No | Hands, a Gmail setting |
| 447, 480 | Idle memory with a live connection; a 100,000-message mailbox's first listing, search and sort | His 99,014 messages are close to the size PERF-03 asks for | No | Sitting, measured by somebody running the harness against his profile, which needs his agreement |
| 535 | What a day at Debug costs on his disk | Debug set, one day | Yes | Sitting, after the level is set |
| 472, 474, 475 | STARTTLS and shutdown on IMAP and POP | Gmail is implicit TLS, so his account never reaches this code | No | A server that speaks STARTTLS |

### 6. Gmail's calendars, contacts and tasks (#22, REAL-01)

There is no ledger entry for #22 itself; REAL-01 and the issue carry it.

| Entry | What it waits for | Proof needs | Logged? | Needs |
|---|---|---|---|---|
| #22 | Adding a Gmail account brings its calendars, contacts and tasks; Refresh brings what the account has | Keys present, his account signed in through the browser, Refresh in each module, rows counted, Google's answers in the log | No: the skip is silent and requests are not logged (above) | Keys, Hands |
| 154, 628, 216, 203 | A push, an answered meeting, an undo, a copied item reaching Google | One of each made here, read in Google Calendar, Contacts, Tasks | Errors only | Keys, Hands |
| 194 | An override row from Google carrying its own RRULE | A moved day of a repeating Google event synced | No | Keys, Sitting once keys work |
| 218, 219, 221, 222, 223, 226, 229 | Task moves at a provider: timing, half-finished moves, field survival, the seven-day memory | Moves between Google task lists, read in Google Tasks | No | Keys, Hands |
| 597 | Five name parts round-tripping through Google | A contact with a prefix, middle name and suffix | No | Keys, Hands |
| 434, 435, 671, 727, 732 | Read in the code and not run: Google's default alert; an explicit "off" alert; a series' offset in summer; Tentative becoming busy; a day Google moved | Each one shows itself once a real Google calendar syncs; each is a `todo` with a recommendation waiting for Pratik's word | No | Keys, Sitting |
| (new) | The Calendar sync adds `held_for_you_to_choose` twice per provider: `src/presentation/wx_app.rs:31225-31226` and `:31258-31259` both read `total_held_for_a_choice += result.held_for_you_to_choose;` | A Google or Microsoft sync with one choice waiting says two | Not ledgered | Found reading; a red and green pair, no account needed |

### 7. Microsoft: Outlook.com, Microsoft 365 and Graph

| Entry | What it waits for | Proof needs | Logged? | Needs |
|---|---|---|---|---|
| 645, 702 | A Microsoft token longer than 1,280 characters failing to save; People.Read and Tasks.ReadWrite consent | A browser sign-in to Microsoft | Failures only | Keys, a Microsoft account, Hands |
| 703 | An account signed in before Tasks.ReadWrite refused its tasks token | A sign-in on an older build, then the new one | Task sync line | Keys, Hands |
| 701 | Microsoft people search rows heard | Ear | No | Keys, Hands, his ear |
| 225, 669 | To Do statuses surviving a move; events read with their zone; an edited meeting's hour at Outlook | As written | No | Keys, Hands |
| 249, 272, 283, 289, 298, 299 | OneNote: the four constraints, picture addresses, created HTML, paging, rate limits | A OneNote notebook synced | No | Keys, Hands |
| 768 | A Microsoft 365 and a Google Workspace account on their own domains | Two work accounts | Failures only | Keys, two work accounts, Hands |
| 706, 709 | getSchedule on a personal account; a colleague's working hours and zone | A personal account and a work account with a colleague | No | Keys, Another person |

### 8. Calendar servers and address book servers (CalDAV, CardDAV)

| Entry | What it waits for | Needs |
|---|---|---|
| 238, 239, 240, 241, 242 | A real server taking the notes' journal entries: listing, version marker, deletion, clash | A CalDAV account (Fastmail, iCloud, Nextcloud or similar); Hands |
| 275, 276, 277 | Notes moved between server folders; held removals; two servers on one account | Same, two servers for 277 |
| 254, 255, 256, 257, 260, 261, 262, 263, 264 | Discovery, card parsing, writing, If-Match and 412, PUT to a new address, change marker, ETag, home set | A CardDAV account; Hands |
| 84 | A push sent over the address book's newer copy, with a sentence afterwards and nobody asked | A CardDAV or Google contact changed in two places, then synced |
| 734, 154 (server half) | A repeating series answered on a server keeping RRULE and EXDATE | A CalDAV calendar and an invitation |
| 707 | Free/busy from two places | A CalDAV calendar and a Google calendar on one account |

Nothing in these groups is logged beyond failure lines (`src/service/caldav.rs` has three warning
lines, all about time zones; `carddav.rs`, `caldav_sync.rs` and `carddav_sync.rs` have none
[VERIFIED: grep count, 2026-10-04]). Pratik has never said he has such an account; the planner
asks.

### 9. Invitations and time zones

| Entry | What it waits for | Needs |
|---|---|---|
| 631 | Real invitations, cancellations and answers from Outlook, Google Calendar and a CalDAV server found and read right | Another person: an organiser on each |
| 152, 153 | A REPLY folded into the organiser's meeting; a forwarded invitation presenting as a meeting | Another person, or a second account of his on another provider |
| 154, 635 | Both orders of answer and calendar check; whether each provider's UID equals its invitations' | Keys, Another person |
| 637, 729, 734 | Real updates and cancellations, one day of a series, a REPLY to one day | Another person, a repeating meeting |
| 663, 669 | An invitation from another zone, Outlook's Windows names and Google's zone names | Another person in another zone |
| 486 | A meeting answer filed while its reply is undone | Hands |
| 666, 727, 730, 731, 735, 737 | Read in the code and not run, each a `todo` waiting for Pratik's word; 730's recommendation is to leave it said until a real organiser's message carries one | Shows itself in the runs above |

A second account of his own on another provider stands in for "another person" for most of this
group: he can invite himself. Only 663 and 709 truly need somebody in another time zone, and a
second account set to another zone may serve for 663.

### 10. S/MIME and PGP with a real correspondent

| Entry | What it waits for | Needs |
|---|---|---|
| 141, 143 | A real S/MIME envelope from Outlook or Thunderbird; a real certificate in his Windows store | A real certificate, Another person |
| 639 | A strongly protected or smart-card key's prompt | A real certificate with protection |
| 655 | A message signed and encrypted here opened in Outlook, Thunderbird and Apple Mail | Another person or a second account read in another program |
| 104, 144, 495, 641, 651, 652 | Real PGP/MIME and inline mail, locked keys, real signatures, a list's footer | A real key pair and a correspondent using Thunderbird, Proton Mail or Mutt |
| 657 | PGP signed and encrypted here opened in Thunderbird and Proton Mail | Same |
| 659 | Bcc with encryption: separate encrypted copies, decided by Pratik to follow phase 14's real correspondent | Waits on the rows above |

Thunderbird Daily is installed on this machine (ledger 789), so Pratik can be his own correspondent
for the Thunderbird half with a second account and a key of his own. Proton Mail, Apple Mail and
Mutt need somebody else.

### 11. Junk, block, rules and Quick Steps at a server

| Entry | What it waits for | Needs |
|---|---|---|
| 675, 756 | Report as Junk at Gmail and at a keyword-keeping IMAP server; its undo | Hands; a non-Gmail IMAP server for the keyword half |
| 689 | A block moving mail already here | Hands |
| 680 | A rule's mark, flag, label and delete at a real server, with and without CONDSTORE | Hands; non-Gmail for CONDSTORE |
| 686, 745, 750 | A set of actions and a Quick Step in order; a rule over thousands of messages at a provider's pace | Hands; 750 is 5,000 workers against Gmail, which his connection limit evidence above suggests may not be taken |
| 117 | Provider spam headers arriving in the shapes parsed | Sitting: his cached mail can be read for them |
| 95 | List-Unsubscribe shapes from real lists | Sitting, same |
| 768 | Report as Junk on Workspace and Microsoft 365 | Keys, two work accounts |

### 12. Directory and people search

| Entry | What it waits for | Needs |
|---|---|---|
| 695, 698 | A real LDAP directory over ldaps:// with a sign-in, an Active Directory root, the credential entry cleared | A work directory, Hands |
| 701, 702 | Microsoft people search | Keys, a Microsoft work account |

### 13. Free/busy

| Entry | What it waits for | Needs |
|---|---|---|
| 704 | Google's freeBusy from a real account, a colleague in the same Workspace, a guest outside | Keys, a Workspace account, Another person |
| 706, 709 | Microsoft getSchedule on personal and work accounts | Keys, Another person |
| 707 | Two places asked at once | Keys, a CalDAV calendar |
| 708 | A zone on a contact: a `todo`, later work | Nothing now |

### 14. Identities, shared mailboxes and delegation

| Entry | What it waits for | Needs |
|---|---|---|
| 718 | Sending from another address at Gmail (set up and not set up) and Exchange | Hands; Gmail's Send mail as setting; Exchange for its half |
| 721 | Shared mailboxes, send on behalf, delegation: not built, need a real shared mailbox | Another person's grant |
| 714 | Reading Gmail's Send mail as list: later work | Keys |

### 15. What real mail looks like (his mailbox can answer these without his hands)

| Entry | Question | How |
|---|---|---|
| 91, 112 | How often senders give Content-Description, on any part and on images | Counted over his cached mail |
| 95 | List-Unsubscribe shapes | Same |
| 104 | Whether real PGP mail arrives inline or as multipart/encrypted | Same, if his mail holds any |
| 111 | How many text attachments are not UTF-8 | Same |
| 117 | Which spam headers arrive | Same |
| 139 | How often the furniture threshold offers or refuses wrongly | Same, by counting pictures by size |
| 60 | Reply headers without References | Same |

Each is a count over his database, which a script can take read-only with his agreement and
print as counts only. Whether the database keeps the raw headers each question needs is the
planner's to read before promising it.

### 16. Imports and exports: explicitly not phase 14

Ledgers 499, 509, 789, 792, 795, 796, 799, 801, 805 and 806 each say in words "Nothing here
touches a server, so nothing goes to phase 14". They need Pratik's files (a `.pst`, a Thunderbird
profile, Outlook `.msg` files, an encrypted `.msg`) and his ear, not an account. Leave them out of
this phase's plans.

### 17. The gate and the four warning surfaces

| Entry | What it waits for | Needs |
|---|---|---|
| 376 | Per-account answers holding back real writes | Hands |
| 377 | The refusal naming Settings rather than the account: wording pinned in four files | Nothing external; a plan of its own when the sentences are rewritten |
| (REAL-02) | `application::allowed`'s default moved per proven path; the settings screen, first-run screen, end of `--help` and the alpha page reworded | Pratik's word per path |

The default today is `FOR_TESTING`: `mail: false, personal_information: true, reading: true`
[VERIFIED: src/application/allowed.rs:154-158]. His profile already holds all three true. The
warning constants a rewording reaches are `EXPERIMENTAL_WARNING`,
`DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL`, `READING_PGP_MAIL_IS_EXPERIMENTAL`,
`SIGNING_AND_ENCRYPTING_IS_EXPERIMENTAL`, `UNDOING_AT_THE_SERVER_IS_EXPERIMENTAL` and
`EMPTYING_THE_TRASH_IS_EXPERIMENTAL` [VERIFIED: src/application/allowed.rs:336, :374, :402, :417,
:431, :443]. One of them is already false, as said above.

## What needs Pratik's hands, and what needs only his account

| Kind | Groups | How many entries, roughly |
|---|---|---|
| Already answered by his profile, owed only a write-up | Sending's 148; delete's replays (376, 546); Gmail's drops and connection limit (11, 64, 65, 67, 72, 523, 525) | 10 |
| His account sitting there, or his database read with his agreement | 5 (part), 15, and 6's code-read todos once keys work | 25 |
| His hands on his one Gmail account | 1, 2 (within one account), 3 (Gmail half), 4, 11 (Gmail half), 14 (Gmail half) | 30 |
| Keys first, then his hands | 6, 7 | 35 |
| A second account of his | 2 (across), 3 (non-Gmail, POP), 5 (CONDSTORE), 8, 11 (keywords) | 35 |
| Another person | 9, 10, 12, 13, 14 (shared) | 35 |

The counts overlap where an entry sits in two groups; they are for sizing, not for a gate.

## Common pitfalls

- **Reading a silent log as a clean run.** On his machine a Refresh in Calendar writes nothing
  whether Google answered with nothing, was never asked, or was skipped for want of keys. Add the
  lines before the run, or the run settles nothing.
- **Proving on a build without keys.** Any plan whose steps say "sign in through the browser"
  fails on a build made without `oauth.toml`; the installing guide says so.
- **Testing-mode tokens expiring.** If the Google project stays in Testing, a sign-in made on day
  one fails on day eight. A proof run across more than a week should expect it, and the failure
  is Google's rule, not a defect.
- **Gmail's connection limit.** His own logs show 15 connections exhausted on 2026-09-19. A proof
  that opens a second program reading the same account (Thunderbird, a phone) while Wixen Mail
  runs may meet "Too many simultaneous connections" and read as a failure of the path under test.
- **The two waiting deletes.** They go to Gmail at the next start of any build on his profile.
  They are his own deletes, so this is expected, but a plan that counts replay lines must expect
  two before anything it asked for.
- **Entries already answered still open.** 148 is the example: proven on the issue, open in the
  ledger. The phase's closing read should look for others the proofs answer on the way.

## Assumptions log

| # | Claim | Section | Risk if wrong |
|---|---|---|---|
| A1 | Pratik's installed builds were made without `oauth.toml` and without the compile-time default, so no build he ran offered the browser sign-in | Prerequisite | If a build carried keys, #22's cause is the password sign-in alone, and the first plan narrows to signing in through the browser [ASSUMED from today's environment and files; the build machine's environment on 2026-09-15 to 20 cannot be read now] |
| A2 | #63's "OAuth" for the proven send is mistaken, or the account was re-added afterwards | Prerequisite | Only the record; the send itself is proven either way [ASSUMED] |
| A3 | Google CalDAV and CardDAV would not serve his account with an app password either | Group 6 | If they did, a no-keys route exists, but it is a different code path from the Google APIs this program uses [ASSUMED] |
| A4 | A second account of his can stand in for "another person" for most invitation proofs | Group 9 | Provider behaviour for a self-invite may differ from a real organiser's [ASSUMED] |
| A5 | His database keeps the headers group 15's counts need | Group 15 | Some counts may need a fetch, which is a read at Gmail [ASSUMED, not read] |

## Open questions for Pratik

1. Do you have, or want to make, a Google Cloud OAuth client and a Microsoft app registration for
   Wixen Mail? Without the Google one, the program cannot bring your calendars, contacts or tasks
   at all, which is #22, and without the Microsoft one nothing Microsoft can be proved.
2. When you proved sending on 2026-09-18, did you sign in through the browser or with an app
   password? Your account is set to an app password today.
3. May the log on your profile be set to Debug for this phase?
4. Which other accounts can you use: a second Gmail, an Outlook.com or Microsoft 365 account, a
   POP account, a calendar or address book server (Fastmail, iCloud, Nextcloud), a work directory?
5. Is there somebody who can send you invitations and signed or encrypted mail, or should the
   proofs that need another person use a second account of yours and stop there?
6. The two deletes waiting in your queue go to Gmail the next time Wixen Mail starts. Is that what
   you expect?

## Sources

- `.planning/WINDOWS.md`, both halves, parsed 2026-10-04.
- `.planning/ROADMAP.md` phase 14 entry; `.planning/REQUIREMENTS.md` REAL-01 and REAL-02; the
  phase 13 and 13.1 READMEs.
- GitHub issues #22 and #63 with comments, read 2026-10-04.
- Code at the lines cited above.
- Pratik's profile under `%LOCALAPPDATA%\wixen-mail`, read-only through Python.
- [CITED: developers.google.com/identity/protocols/oauth2] for Testing-mode refresh tokens; the
  seven-day figure was found through a web search and agrees with Google's page.
