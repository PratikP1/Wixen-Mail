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
- **Crash log** at `crash.log` under the local app data directory. Panics and startup failures also show a message box.
- **Accessibility CI**: a non-blocking Axe.Windows UI Automation scan on every pull request. It covers roughly half of WCAG and does not replace NVDA testing.

- **Announcements are paced.** The queue drops repeats, lets a progress counter supersede its own earlier steps, caps how many announcements can be waiting, and caps how many are spoken per second. Urgent announcements are never held back. Anything dropped is counted and reported rather than vanishing silently.
- **Mute for message reading** (`Ctrl+Shift+M`, also under View). Stops message text being read aloud without silencing status and error announcements, so muting before a screen share does not cost you your error messages.


### Security

- **Fixed unvalidated URLs reaching the operating system shell.** Clicking a link in a message, or using Save Link As on it, passed the URL straight to `open::that`. On Windows that is ShellExecute, which launches executables, reaches UNC paths across the network, and invokes any protocol handler registered on the machine. A `file:///C:/Windows/System32/calc.exe` or `\\evil.example\share\payload.exe` link would have been handed over without a check. All four sites now go through `HtmlRenderer::safe_external_url`, which allows http, https, and mailto and refuses everything else. Refusals are logged rather than silently ignored.
- **Replaced a hand-rolled JSON parser on an attacker-controlled path.** The context menu extracted a link href by scanning for `"href":"` and reading to the next quote, which breaks on the escaping `JSON.stringify` produces: a href containing a quote was silently truncated. It now uses `serde_json`, which was already a dependency.
- **Fixed an HTML injection in plain-text rendering mode.** `html_to_plain_text` strips tags and then decodes entities, so a message body containing `&lt;script&gt;` came back out as live markup. That is correct as plain text and an injection the moment it reaches the WebView. `sanitize_html` now escapes its plain-text output, so what it returns is always safe to embed. The path was not reachable in a shipped build, because nothing constructs the plain-text renderer yet, but the trap was set for whoever wired it up. Found by fuzzing, not by the hostile-input corpus.

### Changed

- Reminders group in the sidebar by urgency: overdue, today, upcoming, no due date, and completed.
- The contacts detail pane lists only fields that have a value, so a screen reader no longer reads out labels with nothing after them.
- Log files are written with a `.log` suffix. Daily rotation had been producing extensionless names that Windows would not open on a double-click.
- Version renumbered to `0.1.0-alpha.9` to match this changelog. The `0.1.1-beta.9` tag had jumped ahead of a codebase that is still pre-beta.

### Fixed

- The note editor filled the title and body with placeholder text on every selection. It now shows the selected note.
- Restored a green build. Formatting and clippy checks had been failing since the architecture overhaul, independent of any feature work.

### Known limitations

- Nothing loads data into the calendar, contacts, reminders, tasks, or notes panels yet. The storage, sync clients, and managers are in place and the panels render, but no code path sends the loaded records to the UI, so the panels stay empty in a running build.
- Notes carry a body preview rather than the full body, so the note editor shows the preview. Saving edits is not wired up.

## [0.1.0-alpha.9] - 2026-03-05

### Added
- **Edge WebView2 email preview** — replaced plain-text RichTextCtrl with a full HTML renderer powered by Edge WebView2 (`wxdragon` WebView widget). Emails now display formatting, colors, images, links, and quoted replies correctly.
- **Compose send preview uses WebView** — the "Review before send" dialog now renders the message body with full HTML formatting instead of plain text.
- **Spacebar read-aloud** — pressing Space on the message list reads the current email aloud through the screen reader (strips HTML to plain text via `HtmlRenderer::html_to_plain_text`).
- **Custom WebView context menu** — right-click on the email preview shows a native popup menu with Select All, Copy Link (on links), and Save Link As (on links). Implemented via JS-to-Rust bridge (`add_script_message_handler`).
- **Dark mode CSS** — email preview automatically adapts to the system color scheme via `prefers-color-scheme: dark`.

### Changed
- Email preview pane switched from `RichTextCtrl` to `WebView` with Edge backend
- `HtmlRenderer` gains `wrap_for_webview()` — wraps sanitized HTML in a styled document template with responsive typography (Segoe UI, 14px, 1.6 line-height)

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
