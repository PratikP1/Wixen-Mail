# Changelog

All notable changes to this project will be documented in this file.
Versioning follows [SemVer](https://semver.org/): `0.1.0-alpha.N` during active development, `0.1.0-beta.N` when feature-complete, `0.1.0-rc.N` for release candidates, `0.1.0` for first public release.

## [Unreleased]

### Added

- **The mail folder tree now fills from the cache.** Opening mail read nothing at all: the handler for a loaded folder list existed and no code ever sent one, so the tree was empty in every build no matter what had been synced.
- **Selecting a folder loads its messages.** The status line said "Loading INBOX..." and then nothing happened. The list only ever filled from the sample mailbox on the Help menu.
- **A Columns dialog** on `F8`, also at View, Columns. Choose which columns the message list shows and in what order. Space shows or hides a column, `Alt+Up` and `Alt+Down` move one, and every change is announced. The last remaining column cannot be hidden. Your choice is remembered across restarts.
- **Sorting from column headers**, with the second click on the same header reversing the order. Dates start at newest first and text at A to Z. The header and the Sort Messages menu stay in step, so the menu always states the order that is actually in effect.
- **A snippet column that has something to read.** The first line of the body is stored beside the message when the body is fetched, so the column keeps working after the body cache evicts. Messages with no plain text part fall back to their HTML.
- **A size column**, spoken in units: "2 KB" rather than "2048". A size we do not know yet reads as blank rather than as "0 bytes", which would be a claim we cannot make.
- **To and Cc columns**, which prefer a display name over a raw address the same way the correspondent column does.
- **The contacts, calendar, reminders, tasks, and notes lists now run in virtual mode**, like the message list. Filling a native list row by row stops being usable somewhere around ten thousand items, and an address book or a task history reaches that. Memory is now proportional to what is on screen rather than to what exists, and UI Automation still reports the real count, so a screen reader says "row 12 of 40,000" and means it.
- **Conversations.** Messages are grouped from the `References` and `In-Reply-To` headers, so threading costs no extra fetch. Subject matching is deliberately not used: "Re: lunch" collides across years and strangers, and a thread that quietly merges two conversations is worse than two threads. A late message that references two separate trees merges them, which is the case the tests are built around.
- **`Enter` on a message in a conversation opens a tree**, on a native tree control so the screen reader announces the level itself. `Enter` on the first row opens the whole conversation as one document; `Enter` on a message opens that message. The first row is labelled "Whole conversation, 5 messages" rather than repeating the subject, because `Enter` does two things there and the row has to say which. `Escape` goes back to the list with focus on the row it came from. A message with no conversation opens straight into the preview, with no tree in the way.
- **A whole conversation renders as one document**, each message introduced by a heading so `H` moves between them. Levels cap at `h6` and never skip: skipping a heading level is a structure violation in its own right, and conversations go deeper than six, so the real depth moves into the heading text as "Reply, level 8". Bodies are sanitized exactly as they are anywhere else; being part of a thread does not make a stranger's HTML safer.
- **A conversation of one is not reported as a conversation**, so an ordinary message carries no thread indicator and raises no earcon.
- **Next and previous unread** on `Ctrl+Shift+N` and `Ctrl+Shift+P`. They wrap at the ends, and say "no unread messages" rather than doing nothing, because a key that silently does nothing is indistinguishable from a key that is broken.
- **Flag a message** with `Ctrl+Shift+S`, which writes through to the cache.
- **`F5` reads the current folder again** and `F6` moves between the folder, message, and preview panes, skipping the preview when it is hidden rather than focusing something invisible.
- **Space reads the item under the cursor, in all six modules.** A list row is read as its visible columns and nothing else, so a task's description, a contact's phone number, or a message's recipients were invisible until you opened the item. `Space` reads the short form, pressing it again reads everything the record holds, and a third press goes back. `Shift+Space` reads everything outright. Moving to another row starts again at the short form. There is no double-press timing window: the second press does the second thing however long you took, because a timing window locks out anyone who types slowly.
- **Landing on a conversation is signalled**, so it can be a short tone rather than another sentence on every row. Which channel it uses is a setting, not a decision made in the code.
- **Feedback on four channels: speech, braille, sound, and the status bar.** Events such as new mail, a sent message, a lost connection, or a failed send are now facts the application signals rather than sentences it speaks. A new Feedback tab in Settings decides which channels each one reaches. This matters most to two groups pulling in opposite directions: a deaf-blind user can switch speech off and keep braille, and someone working in an open office can swap a spoken sentence for a short tone.
- **Nothing is ever signalled by sound alone.** If sounds are the only channel left on, a written equivalent is added automatically, unless you switched every text channel off yourself. The rule lives in the routing rather than at each call site, so no future event can bypass it by forgetting.
- **Each event has its own tone**, and tones are spaced out so a syncing mailbox does not run them together. An earcon that cannot be told apart from its sibling carries no information.
- **Attachment records are stored and read back.** The attachments table existed and nothing ever wrote to it, so the attachment column could never have been true. Listing a folder now reports attachment presence without loading the attachments.

### Fixed

- **Two menu items shared an identifier, and the application asserted on startup.** `Ctrl+Shift+M` (mute) and `F8` (columns) were written with the same offset, as were next-unread and the Help menu's sample mailbox. wxWidgets resolves a duplicate id by acting on the first item that carries it, so the startup mute sync tried to tick the Columns item, which is not a checkable item. Beyond the assert, `Ctrl+Shift+M` would have opened the Columns dialog. Identifiers are now numbered by a macro rather than by hand, so a collision cannot be written, and a test refuses any that are added by hand beside it.
- **The keyboard shortcut reference documented eight shortcuts that did not exist**: `F5`, `F6`, `F3`, `N`, `P`, `S`, `Ctrl+1`, and `Ctrl+2`. The useful ones are now implemented, and the rest are gone from the document. A reference that lists keys which do nothing is worse than one that lists fewer keys.

### Removed

- **The Answered, Draft, and Tags columns.** They were offered in the column model with no data behind them, so switching one on would have given a column that read blank on every row. They return when IMAP flag sync lands and there is something real to put in them.

### Known limitations

- Sorting still happens in memory over the loaded folder rather than in SQL. That is fine for the folder sizes the application can currently reach, and it is the wrong shape for the hundreds of thousands of messages the storage design targets. The SQL ordering is written and tested; the listing query does not use it yet.
- Earcons are Windows-only for now. On macOS and Linux the sound channel is silent and the text channels carry the event on their own; a port needs its own audio path.
- Feedback preferences are per channel, not per event. The per-event overrides exist in the model and have no interface yet, because a grid of nine events by four channels is not the choice most people are making.
- Threading runs over the loaded folder rather than incrementally as mail arrives, because no IMAP sync feeds it yet. The `References` headers have nowhere to come from until that lands, so in practice every message currently threads alone.

### Added, earlier in this cycle

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

- **Message bodies moved out of the messages table.** They used to sit inline, so every folder listing dragged body text through SQLite to render a subject line, and a mailbox of a few hundred thousand messages would have been tens of gigabytes in one file. Bodies now live in their own table, are read only when a message is opened, and can be evicted least-recently-read against a size budget. Databases written by earlier versions have their inline bodies moved across on first open, and the space is reclaimed.

- **Announcements now actually reach the screen reader.** `announce` stored the text, then fired a name-change event telling the screen reader to re-read the title bar. The text was never handed to any accessibility API, so nothing the application announced was ever spoken. It now uses `UiaRaiseNotificationEvent`, the call meant for saying something not tied to a focus change, which NVDA routes to speech and to a connected braille display. The queue's priority and topic are passed through, so its coalescing and the screen reader's agree instead of fighting.

### Security

- **Dependency advisories are now checked in CI.** `cargo audit` runs on every push and pull request. Advisories reach this project through transitive dependencies, where a green build says nothing about them, and the first run found five.
- **Three of those five affect TLS certificate validation at runtime.** `rustls-webpki 0.101.7` carries two name-constraint bypasses and a reachable panic in certificate revocation list parsing. It is pinned by `oauth2 4`, which depends on `reqwest 0.11`, which depends on `rustls 0.21`. Upgrading to `oauth2 5` is the fix and is not yet done: version 5 moves to a typestate builder that changes the shape of the client construction and the token exchange, and the OAuth flow has never been run against a live account, so it needs its own pass with real credentials rather than a compile-and-hope.
- The other two advisories are `quick-xml 0.38.4` denial-of-service issues reached only through `wxdragon-macros`, a proc macro. They run at compile time and never see network data. The `quick-xml 0.41` that parses CalDAV responses is the fixed version.
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
