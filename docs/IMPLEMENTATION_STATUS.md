# Implementation status

_Last updated: 2026-08-09_

This file is the canonical answer to "does this work yet". It is written to be
believed, so anything not finished is listed as not finished.

Wixen Mail is **pre-beta**. It can send and receive mail. Reading mail is the
part that has been used; everything that writes to a server is experimental,
and [the testing guide](ALPHA_TESTING.md) says what that means before anything
else. The changelog says what is in each release, so this page does not name a
version number that would go stale between updates.

## Can you use it today

| Task | State |
|------|-------|
| Send a message | Yes, over SMTP with TLS, signed in with a password or with OAuth |
| Read your inbox | Yes, over IMAP with TLS |
| Download mail with POP3 | Yes, and mail is left on the server unless you turn that off |
| Keep contacts, calendar, tasks, notes, reminders | Yes, stored locally and shown in their panels |
| Sync contacts and calendars with a provider | Built, and never yet run against a live account |
| Use it entirely from the keyboard | Yes |
| Use it with a screen reader | Yes for what exists, and not yet verified against NVDA |

## What works

**Sending.** SMTP with TLS and STARTTLS, signed in with a password or with
OAuth. Composing queues the message in a local outbox and flushes it, so a
send that fails is retried rather than lost. Failures say whether the problem
is the transport or the account's configuration. A draft can be saved, by
hand or automatically.

**Receiving.** IMAP with TLS and STARTTLS: folders, messages, search,
threading worked out from message headers, and fetching older mail a page at
a time. POP3 with the same protections and its own folders on this computer;
mail stays on the POP server unless the account is told to remove it. OAuth
sign-in works for receiving as well as sending.

**Local storage.** A cache on this computer holds messages, contacts, contact
groups, calendars, calendar events, reminders, task lists, tasks, note folders,
and notes. It is not encrypted, so anybody who can read the file can read the
mail in it. It carries no credentials, though: passwords and tokens go to the
Windows credential store instead.

**The six modules.** Mail, contacts, calendar, reminders, tasks, and notes share
one window, switched with `Ctrl+Shift+1` through `Ctrl+Shift+6`. Opening a module
loads its records from the cache. Notes can be edited and saved.

**Accessibility.** Every list, tree, and text field carries an accessible name
through a `wxAccessible`. Announcements are prioritised, deduplicated, coalesced
by topic, and bounded to four per second, with anything dropped reported rather
than silently discarded. Message reading can be muted with `Ctrl+M`, and
that preference persists. Check menu items report their real state.

**Provider sync.** Google and Microsoft Graph clients for contacts and
calendars, a calendar-server client for calendars added by their address, and
subscription feeds. All are reachable from the menus, a change made here is
pushed back when Allow Changes permits it, and a repeating event is shown on
the days it lands on. None of it has been exercised against a live account.

## What does not work

**Threaded view.** Present in the View menu and disabled. The data model carries
thread identifiers; nothing groups by them.

**Folder management.** A folder cannot be created, renamed or deleted, and a
whole folder cannot be marked read or emptied.

**Moving a task between lists.** A task goes into your provider's default list
when you make it, and moving and copying work for mail only.

**Anything that writes, against a real account.** Sending, deleting, moving,
copying, filing a copy in Sent, read receipts, subscriptions, and the syncs
that push changes are all built, and none of them has ever run against a real
account. [What is worth testing, and what is known to be
broken](ALPHA_TESTING.md) keeps the fuller list.

## Quality gates

Every commit must pass four checks, run together by `scripts/check.sh`:

```bash
bash scripts/check.sh
```

`rustfmt`, `clippy` with `-D warnings`, the test suite, and a release build. Turn
them on for every commit with `git config core.hooksPath .githooks`, so the
answer cannot be lost between running them and committing.

A separate non-blocking workflow starts the application once per window and
scans it on both of Windows' accessibility channels: Axe.Windows over the UI
Automation tree, which is what Narrator reads, and `scripts/msaa-names.ps1` over
the MSAA tree, which is what NVDA reads for native controls.

The second one exists because until it did, no accessible name in this
application had ever been measured. For an edit box or a button, Windows
supplies its own UI Automation provider that shadows the MSAA object underneath
it, and `set_accessible_name` writes only to MSAA. So the UI Automation scan
reported the system's name for those controls and never the one the code set,
and every `set_accessible_name` call in the tree could have been deleted without
it noticing.

3362 tests pass: 3282 unit and 80 integration, measured 2026-08-09 with
`cargo test --all-targets`. Several are fuzz tests over generated hostile
input, covering the HTML renderer, the calendar-document parsers, OAuth token
expiry, and account validation.

When the accessibility scan was last read, on 2026-07-26, it reported five
findings, all inside WebView2's own accessibility tree rather than this
application's controls.

## Known gaps in verification

Automated scanning covers roughly half of WCAG. **No part of this application has
been tested with a real screen reader yet.** Structure being present is not the
same as the experience being good, and only an NVDA run can tell the difference.

The provider sync clients have not been run against live Google, Microsoft, or
CalDAV accounts.

## Which tests would fail if the code were wrong

Red/green started at commit 182 of 344, so many of the older tests here were
written after the code they cover. A test written that way describes what the
code does rather than specifying what it should do, and cannot fail for the bug
it was written alongside. Mutation testing measures the difference: it alters
the code and reruns the suite, and reports anything nothing caught.

Run it with `scripts/mutants.sh <dir>`. A whole-tree run is about two days, so
it is used scoped. The table below is from 2026-07-26.

| Module | Caught | Missed | What that means |
|---|---|---|---|
| `service/mime.rs` | 33 | 0 | Every message that arrives is parsed here, and the tests hold all of it |
| `common/error.rs` | 15 | 0 | The secret redaction is fully pinned |
| `data/message_cache/tasks.rs` | 16 | 1 | The survivor was dead code, not a weak test |

The one survivor was `delete_task_list`, which no code calls. That is the other
thing a survivor can mean, and the more useful one: Rust never reports a public
item in a lib crate as unused, so three dead container deletions had survived
two dead-code passes.

A later sweep, on 2026-08-01, took the message filters, due dates, tagging and
signatures modules through the same measurement: 157 mutants, 141 caught, 16
that would not compile, and none missed.

On 2026-08-12, at commit 0bc0614, the four modules that decide what becomes of
somebody's copy of a message were measured for the first time: where a deleted
message may go, the copy of a draft kept at the server, the copy of a sent
message, and what the list does once the server has answered a delete. 66
mutants, 53 caught, 1 missed, 12 that would not compile, none timed out. The
suite answered for 54 of the 66, so 18 percent of that run asked nothing. It
took two runs to get there, because the first was killed after 35 of them; each
of the four files was finished within one run, and no file's result is spread
across both.

The one survivor was the closing of the connection a queue of outgoing mail
opens. Emptied out, the whole suite still passed, so a build that held a
connection open at the server after every send would have gone out. It is
pinned now.

Two things that run could not see matter more than the one it found. No mutant
is produced for the early return that stops a reason the server already
punctuated being punctuated again, so a doubled full stop in the middle of the
sentence saying where a message ended up was untested in both of the two places
that sentence is built. And of the four mutants in the module deciding what the
list does after a delete, two would not compile and the other two are about the
sentence for a change that never reached the server, so a clean result there
means two questions were asked about the least consequential function in the
file. The lists of endings it walks are written out by hand and nothing made
them keep up with the two sets of endings they stand for. Both gaps are tests
now.

Line coverage was 60.4% when last measured, on 2026-07-26, with
`cargo llvm-cov --lib --summary-only`. A great deal has landed since, so that
number is the last measurement rather than today's answer. The least covered
code then was the network transport, which is the same fact as never having
been run against a live account rather than a separate problem, and no amount
of test writing substitutes for it.
