# Implementation status

_Last updated: 2026-07-26_

This file is the canonical answer to "does this work yet". It is written to be
believed, so anything not finished is listed as not finished.

Wixen Mail is **pre-beta**, at version `0.1.0-alpha.10`. It can send mail. It
cannot receive mail.

## Can you use it today

| Task | State |
|------|-------|
| Send a message | Yes, over SMTP with TLS |
| Read your inbox | No, IMAP is not implemented |
| Download mail with POP3 | No, POP3 is not implemented |
| Keep contacts, calendar, tasks, notes, reminders | Yes, stored locally and shown in their panels |
| Sync contacts and calendars with a provider | Partly, see below |
| Use it entirely from the keyboard | Yes |
| Use it with a screen reader | Yes for what exists, and not yet verified against NVDA |

## What works

**Sending.** SMTP through `lettre`, with TLS and STARTTLS. Composing queues the
message in a local outbox and flushes it, so a send that fails is retried rather
than lost. Failures say whether the problem is the transport or the account's
configuration.

**Local storage.** An encrypted SQLite cache holds messages, contacts, contact
groups, calendars, calendar events, reminders, task lists, tasks, note folders,
and notes. Nothing sensitive is stored in it: passwords and tokens go to the
Windows credential store, and the cached mail is not encrypted.

**The six modules.** Mail, contacts, calendar, reminders, tasks, and notes share
one window, switched with `Ctrl+Shift+1` through `Ctrl+Shift+6`. Opening a module
loads its records from the cache. Notes can be edited and saved.

**Accessibility.** Every list, tree, and text field carries an accessible name
through a `wxAccessible`. Announcements are prioritised, deduplicated, coalesced
by topic, and bounded to four per second, with anything dropped reported rather
than silently discarded. Message reading can be muted with `Ctrl+Shift+M`, and
that preference persists. Check menu items report their real state.

**Provider sync.** Google and Microsoft Graph clients for contacts and calendars,
a CalDAV client, and read-only iCal subscriptions. These are reachable from the
menus. They have not been exercised against live accounts.

## What does not work

**Receiving mail.** `service::protocols::imap` and `service::protocols::pop3`
perform no network I/O. Every call returns fabricated data. Nothing in the window
is wired to them, deliberately: showing invented folders and messages as your own
mail would be worse than showing none. This is the largest single piece of
outstanding work.

**OAuth sending.** The SMTP layer authenticates with a password and has no
XOAUTH2 support, so an account configured for OAuth is refused with a message
saying so.

**Drafts.** Save Draft says it is not implemented, because it is not.

**Threaded view.** Present in the View menu and disabled. The data model carries
thread identifiers; nothing groups by them.

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

The second one is new, and until it existed no accessible name in this
application had ever been measured. For an edit box or a button, Windows
supplies its own UI Automation provider that shadows the MSAA object underneath
it, and `set_accessible_name` writes only to MSAA. So the UI Automation scan
reported the system's name for those controls and never the one the code set,
and every `set_accessible_name` call in the tree could have been deleted without
it noticing.

424 tests pass: 394 unit and 30 integration. Several are fuzz tests over
generated hostile input, covering the HTML renderer, the CalDAV and iCalendar
parsers, OAuth token expiry, and account validation.

The accessibility scan reports five findings, all inside WebView2's own
accessibility tree rather than this application's controls.

## Known gaps in verification

Automated scanning covers roughly half of WCAG. **No part of this application has
been tested with a real screen reader yet.** Structure being present is not the
same as the experience being good, and only an NVDA run can tell the difference.

The provider sync clients have not been run against live Google, Microsoft, or
CalDAV accounts.

## Which tests would fail if the code were wrong

Red/green started at commit 182 of 344, so most of the tests here were written
after the code they cover. A test written that way describes what the code
does rather than specifying what it should do, and cannot fail for the bug it
was written alongside. Mutation testing measures the difference: it alters the
code and reruns the suite, and reports anything nothing caught.

Run it with `scripts/mutants.sh <dir>`. A whole-tree run is about two days, so
it is used scoped.

| Module | Caught | Missed | What that means |
|---|---|---|---|
| `service/mime.rs` | 33 | 0 | Every message that arrives is parsed here, and the tests hold all of it |
| `common/error.rs` | 15 | 0 | The secret redaction is fully pinned |
| `data/message_cache/tasks.rs` | 16 | 1 | The survivor was dead code, not a weak test |

The one survivor was `delete_task_list`, which no code calls. That is the other
thing a survivor can mean, and the more useful one: Rust never reports a public
item in a lib crate as unused, so three dead container deletions had survived
two dead-code passes.

Line coverage is 60.4% (`cargo llvm-cov --lib --summary-only`). The least
covered modules are `service/protocols/imap.rs` at 28%, `service/google_api.rs`
at 35% and `service/microsoft_graph.rs` at 37%. That is the same fact as never
having been run against a live account rather than a separate problem, and no
amount of test writing substitutes for it.
