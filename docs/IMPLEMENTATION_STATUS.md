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
and notes. Encryption is AES-256-GCM with the key in the OS keychain.

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

`rustfmt`, `clippy` with `-D warnings`, the test suite, and a release build. A
separate non-blocking workflow scans the running application's UI Automation tree
with Axe.Windows on every pull request.

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
