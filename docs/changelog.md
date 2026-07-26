# Changelog

All notable changes to this project will be documented in this file.
Versioning follows [SemVer](https://semver.org/): `0.1.0-alpha.N` during active development, `0.1.0-beta.N` when feature-complete, `0.1.0-rc.N` for release candidates, `0.1.0` for first public release.

## [Unreleased]

### Added

- **Five new modules alongside mail**: calendar, contacts, reminders, tasks, and notes. All six share one window and one focus model. Switch between them with `Ctrl+Shift+1` through `Ctrl+Shift+6`.
- **Calendar and contact sync** through the Google and Microsoft Graph APIs, with incremental sync using Google sync tokens and Microsoft delta links.
- **CalDAV support** for providers that offer no REST API, and read-only iCal subscription feeds.
- **Storage for the new modules** in the existing encrypted cache: calendars, calendar events, reminders, task lists, tasks, note folders, and notes.
- **Calendar display settings**: default view, weekend visibility, first day of the week, and reminder lead time.
- **Message delete and read-toggle** now reach the cache. Both actions were already in the context menu with nothing behind them.
- **The calendar, contacts, reminders, tasks, and notes panels now show your data.** Opening a module reads its records from the local cache and fills the panel. Every one of these panels previously rendered empty in a running build no matter what was stored, because nothing connected the storage to the display.
- **Default containers are created on first use**, so a new account opens with a calendar, a task list, and a note folder rather than empty sidebars.
- **Notes can be edited and saved.** Selecting a note loads its full body rather than the truncated list preview, and a Save Note button writes it back. Fields the editor does not show, such as the folder and the pin, are preserved through a save.
- **Muting message reading is remembered** across restarts, so working in a shared room does not mean switching it off again every session.
- **Queued mail is actually sent.** The outbox flush had a hardcoded failure in place of a call to the SMTP transport, so every queued message was recorded as failed with "SMTP send not yet wired". The transport itself was already real; only the call was missing. Failures now say whether the problem is the transport or the account's configuration.
- **Crash log** at `crash.log` under the local app data directory. Panics and startup failures also show a message box.
- **Accessibility CI**: a non-blocking Axe.Windows UI Automation scan on every pull request. It covers roughly half of WCAG and does not replace NVDA testing.
- **Announcements are paced.** The queue drops repeats, lets a progress counter supersede its own earlier steps, caps how many announcements can be waiting, and caps how many are spoken per second. Urgent announcements are never held back. Anything dropped is counted and reported rather than vanishing silently.
- **Mute for message reading** (`Ctrl+Shift+M`, also under View). Stops message text being read aloud without silencing status and error announcements, so muting before a screen share does not cost you your error messages.

### Security

- **Account validation now checks the ports.** `Account::validate` checked the name, email, servers, username and password and never looked at the port fields, so a typo like `5877` or `abc` was accepted and only surfaced later as a connection failure with no mention of which field caused it. Port 0 is refused too: it means "any free port" to the operating system and is never what someone meant to type.
- **Fixed an OAuth token expiry check that failed open.** `is_expired` returned "not expired" when the stored timestamp could not be parsed, so a corrupted expiry made a dead token look valid forever: the client never refreshed, every call came back 401, and there was nothing to tell the user. It now fails closed. The same rule existed twice, once here failing open and once inline in `get_valid_token` failing closed; there is now one implementation.
- **Fixed a remotely triggerable crash in calendar parsing.** `normalize_ical_datetime` sliced datetime values by byte offset without checking they were ASCII, so a subscribed .ics feed or CalDAV server sending a multibyte character across one of those offsets panicked the parser. An 8-byte value like `abc€de` was enough. Found by fuzzing.
- **Fixed iCalendar property lookup matching on a prefix.** Asking for `SUMMARY` was also satisfied by a crafted `SUMMARYX` line, letting a hostile feed feed values into fields that were never requested. A property name must now be followed by `:` or `;`.
- **Fixed unvalidated URLs reaching the operating system shell.** Clicking a link in a message, or using Save Link As on it, passed the URL straight to `open::that`. On Windows that is ShellExecute, which launches executables, reaches UNC paths across the network, and invokes any protocol handler registered on the machine. A `file:///C:/Windows/System32/calc.exe` or `\\evil.example\share\payload.exe` link would have been handed over without a check. All four sites now go through `HtmlRenderer::safe_external_url`, which allows http, https, and mailto and refuses everything else. Refusals are logged rather than silently ignored.
- **Replaced a hand-rolled JSON parser on an attacker-controlled path.** The context menu extracted a link href by scanning for `"href":"` and reading to the next quote, which breaks on the escaping `JSON.stringify` produces: a href containing a quote was silently truncated. It now uses `serde_json`, which was already a dependency.
- **Fixed an HTML injection in plain-text rendering mode.** `html_to_plain_text` strips tags and then decodes entities, so a message body containing `&lt;script&gt;` came back out as live markup. That is correct as plain text and an injection the moment it reaches the WebView. `sanitize_html` now escapes its plain-text output, so what it returns is always safe to embed. The path was not reachable in a shipped build, because nothing constructs the plain-text renderer yet, but the trap was set for whoever wired it up. Found by fuzzing, not by the hostile-input corpus.

### Changed

- Reminders group in the sidebar by urgency: overdue, today, upcoming, no due date, and completed.
- The contacts detail pane lists only fields that have a value, so a screen reader no longer reads out labels with nothing after them.
- Log files are written with a `.log` suffix. Daily rotation had been producing extensionless names that Windows would not open on a double-click.
- Version is `0.1.0-alpha.10`, continuing the alpha line. Two beta tags were cut by accident and have been withdrawn; the codebase is still pre-beta.

### Fixed

- The note editor filled the title and body with placeholder text on every selection. It now shows the selected note.
- **Check menu items never reflected their state.** Folder pane, preview pane, module buttons, mute, and offline mode all announced "checked" or "unchecked" from a state nothing updated, so a screen reader was told the opposite of the truth half the time.
- **Em-dashes removed from spoken text.** Sixteen user-facing strings used them, and screen readers announce them inconsistently depending on the user's punctuation level.
- A poisoned lock no longer takes the window down or silently discards an update. Every access to the shared UI state now recovers and carries on.
- Restored a green build. Formatting and clippy checks had been failing since the architecture overhaul, independent of any feature work.

### Known limitations

- **Receiving mail is not implemented.** The IMAP and POP3 modules perform no network I/O; every call returns fabricated data. Nothing in the window is wired to them, deliberately, because showing invented folders and messages as your own mail would be worse than showing none. Sending works; receiving does not.
- Sending does not support OAuth accounts. The SMTP layer authenticates with a password and has no XOAUTH2 support, so a Gmail or Outlook account configured for OAuth is refused with a message saying so rather than failing at the server.
- Threaded view appears in the View menu and is disabled, because threading is not implemented. It is left visible so its absence is discoverable rather than silently missing.
- Five accessibility scan findings remain, all inside WebView2's own accessibility tree (`Chrome_WidgetWin_1`, `BrowserRootView`, and three container views). They are not this application's controls and cannot be named or positioned from here.

## [0.1.0-alpha.9] - 2026-03-05

### Added
- **Edge WebView2 email preview**: replaced plain-text RichTextCtrl with a full HTML renderer powered by Edge WebView2 (`wxdragon` WebView widget). Emails now display formatting, colors, images, links, and quoted replies correctly.
- **Compose send preview uses WebView**: the "Review before send" dialog now renders the message body with full HTML formatting instead of plain text.
- **Spacebar read-aloud**: pressing Space on the message list reads the current email aloud through the screen reader (strips HTML to plain text via `HtmlRenderer::html_to_plain_text`).
- **Custom WebView context menu**: right-click on the email preview shows a native popup menu with Select All, Copy Link (on links), and Save Link As (on links). Implemented via JS-to-Rust bridge (`add_script_message_handler`).
- **Dark mode CSS**: email preview automatically adapts to the system color scheme via `prefers-color-scheme: dark`.

### Changed
- Email preview pane switched from `RichTextCtrl` to `WebView` with Edge backend
- `HtmlRenderer` gains `wrap_for_webview()`, which wraps sanitized HTML in a styled document template with responsive typography (Segoe UI, 14px, 1.6 line-height)

### Security
- All navigation inside the WebView is blocked; clicked links open in the default browser via `open::that()`
- New-window popup requests are vetoed
- Browser developer tools disabled (`enable_access_to_dev_tools(false)`)
- Default context menu disabled; replaced with a minimal custom menu
- HTML sanitization via `ammonia` remains the first line of defense against XSS
- Base URL set to `about:blank` to prevent relative resource resolution

## [0.1.0-alpha.8] - 2026-03-01

### Added
- Main window toolbar with stock icons (Get Mail, New, Reply, Reply All, Forward, Delete, Mark Read, Search)
- Compose dialog toolbar with Send (prominent), Undo, Redo, Bold, Italic, Underline, Attach
- Visual styling: folder tree sidebar tint, message list and preview fonts, 3-field status bar
- Compose dialog enlarged to 850x700 for comfortable editing

### Changed
- Architecture refactoring: AES-256-GCM encryption, MessageCache split into 11 modules, MailController cleanup with `SendEmailRequest` struct, type deduplication with `From` conversions
- Consolidated 50+ root-level planning/implementation docs into `docs/development/`
- Moved `ARCHITECTURE.md`, `ROADMAP.md`, `INTEGRATION_GUIDE.md`, `UI_FEATURES.md` into `docs/`
- Updated README with current project state and new documentation structure

### Fixed
- Removed dead code (unused imports, unreachable arms, stale feature flags)
- Fixed entry point to launch actual UI instead of diagnostic output

## [0.1.0-alpha.1] - 2026-02-15

### Added
- First internal alpha assembled from initial development work.
- Beta readiness diagnostics in the Help menu.
- POP3 command-surface support and IMAP IDLE push event plumbing.
- OAuth manager, offline outbox queue, filters, contacts, and HTML attachment pipeline support.
- Accessibility automation/UIA bridge coverage and expanded keyboard-first integrated UI flows.

### Packaging
- Windows setup packaging is available through the release workflow and `installer/Wixen-Mail-Setup.iss`.
