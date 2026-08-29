<!-- refreshed: 2026-08-29 -->
# Architecture

**Analysis Date:** 2026-08-29

## System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                     Presentation Layer                       │
│  wxdragon UI (Windows) + accessibility bridge                │
│  `src/presentation/`  (wx_app.rs is the main window)          │
└──────────────────────────┬────────────────────────────────────┘
                            │ calls into
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│  Business logic: sync, filters, composition, PIM managers    │
│  `src/application/` (69 modules, mail_controller.rs is hub)   │
└──────────────────────────┬────────────────────────────────────┘
                            │ calls into
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                       Service Layer                          │
│  Protocol clients, security, external APIs                   │
│  `src/service/` (IMAP/SMTP/POP3, CalDAV, OAuth, security)     │
└──────────────────────────┬────────────────────────────────────┘
                            │ calls into
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                        Data Layer                             │
│  SQLite cache, account/config storage                        │
│  `src/data/` (message_cache is the one database)              │
└─────────────────────────────────────────────────────────────┘

           `src/common/` (error, paths, types, logging, version)
           sits beside all four layers and is used by every one of them.
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| Main window | Owns the wxdragon frame, wires UI events to application calls | `src/presentation/wx_app.rs` |
| Accessibility bridge | UI Automation / MSAA names, announcements, feedback events | `src/presentation/accessibility/` |
| Mail controller | Bridges UI to IMAP/SMTP/POP3 sessions, holds connection state | `src/application/mail_controller.rs` |
| Mail sync | Drives folder/message synchronization against a server | `src/application/mail_sync.rs` |
| CalDAV/contacts/tasks sync | Same sync pattern per PIM domain | `src/application/caldav_sync.rs`, `src/application/contacts_sync.rs`, `src/application/tasks_sync.rs`, `src/application/collection_sync.rs` |
| Filters | Rule-based message filtering | `src/application/filters.rs` |
| IMAP/SMTP/POP3 clients | Protocol implementations over the wire | `src/service/protocols/imap.rs`, `src/service/protocols/smtp.rs`, `src/service/protocols/pop3.rs` |
| CalDAV client | Calendar/task protocol client | `src/service/caldav.rs` |
| OAuth | Token exchange and storage for Google/Microsoft | `src/service/oauth.rs`, `src/service/oauth_credentials.rs` |
| Credential store | Windows credential store (DPAPI-backed) access | `src/service/credentials.rs` |
| Message cache | The one SQLite database (`message_cache.db`) | `src/data/message_cache/` |
| Account/config storage | Account records and app settings, `%LOCALAPPDATA%\wixen-mail\config\` | `src/data/account.rs`, `src/data/config.rs` |
| Error/Result types | Shared error enum used across all layers | `src/common/error.rs` |
| Paths | Single source of truth for the data folder root | `src/common/paths.rs` |
| Entry point | Process startup, single-instance claim, panic hook, logging init | `src/main.rs` |
| Library root | Declares the four layers as public modules | `src/lib.rs` |

## Pattern Overview

**Overall:** Layered architecture (presentation → application → service → data), each layer a Rust module tree under `src/`, not separate crates. There is no dependency-inversion boundary enforced by the compiler between layers; layering is a convention, not a hard module-visibility wall (application code freely does `crate::service::...` and `crate::data::...`).

**Key Characteristics:**
- Single binary crate (`wixen_mail`), `src/lib.rs` re-exports the four layers plus `common` and `vendor`.
- `src/application/` is flat and wide: 69 files, one manager/concern per file, named after the behavior rather than a noun (`answering.rs`, `closing.rs`, `filing.rs`, `handover.rs`) — see naming convention note in STRUCTURE.md.
- `src/presentation/` mixes generic UI infrastructure (`theme.rs`, `panes.rs`, `managers.rs`) with per-screen `wx_*.rs` modules (`wx_compose.rs`, `wx_settings.rs`, `wx_calendar_module.rs`) and a dedicated `accessibility/` subtree.
- `src/service/` groups protocol clients (`protocols/imap`, `protocols/pop3`, `protocols/smtp.rs`), provider APIs (`google_api.rs`, `microsoft_graph.rs`), and cross-cutting services (`security.rs`, `safety.rs`, `secret_store.rs`).
- One SQLite database for cached mail/PIM data; secrets never enter it — they go to the OS credential store through `service::credentials`, `service::oauth`, `service::caldav`.

## Layers

**Presentation (`src/presentation/`):**
- Purpose: render the wxdragon UI, translate user input into application-layer calls, own accessibility exposure (names, roles, announcements).
- Contains: the main window (`wx_app.rs`), per-feature dialogs/panels (`wx_compose.rs`, `wx_settings.rs`, `wx_calendar_module.rs`, `wx_contacts_module.rs`, `wx_tasks_module.rs`, `wx_notes_module.rs`, `wx_reminders_module.rs`), shared UI infrastructure (`theme.rs`, `panes.rs`, `managers.rs`, `status_line.rs`), and `accessibility/` (screen reader bridge, feedback events, names).
- Depends on: `application`, `common`, `data` (reads accounts/messages directly for display), `wxdragon` crate.
- Used by: `src/main.rs` only (nothing above it).

**Application (`src/application/`):**
- Purpose: business logic — account management, mail/PIM synchronization, composition, filtering, search, PIM item lifecycle.
- Contains: one file per concern; `mail_controller.rs` is the hub that owns the live IMAP/SMTP/POP3 sessions and exposes typed operations (`SendEmailRequest`, address parsing) to the presentation layer.
- Depends on: `service` (protocol clients), `data` (account/message storage), `common`.
- Used by: `presentation`.

**Service (`src/service/`):**
- Purpose: protocol implementations (IMAP/SMTP/POP3/CalDAV), external provider APIs (Google, Microsoft Graph), and cross-cutting services (security, credentials, spellcheck, PDF, MIME parsing).
- Contains: `protocols/` (imap/pop3/smtp, each with its own submodule for imap and pop3), `safebrowsing/`, `spellcheck/`, plus flat files for each service (`caldav.rs`, `oauth.rs`, `credentials.rs`, `security.rs`, `mime.rs`, `pdf.rs`).
- Depends on: `data` (rarely, mostly for typed conversions), `common`.
- Used by: `application`.

**Data (`src/data/`):**
- Purpose: persistence — the one SQLite cache database, account records, app configuration, static email-provider metadata.
- Contains: `message_cache/` (the database module, schema is additive-only per `docs/architecture.md`), `account.rs`, `config.rs`, `email_providers.rs`.
- Depends on: `common`.
- Used by: `application`, `presentation` (reads for display).

**Common (`src/common/`):**
- Purpose: shared types and utilities with no business logic — error type, path resolution, logging setup, version stamping.
- Contains: `error.rs` (the `Error`/`Result` used everywhere), `paths.rs` (single owner of the `%LOCALAPPDATA%\wixen-mail` root), `types.rs`, `logging.rs`, `version.rs`, `moment.rs`, plus test-only helpers gated behind `#[cfg(test)]` (`answering.rs`, `temp_home.rs`, `what_ships.rs`).
- Depends on: nothing internal.
- Used by: every other layer.

## Data Flow

### Primary Request Path (send/receive mail)

1. User action in a wxdragon widget triggers an event handler in `src/presentation/wx_app.rs`.
2. The handler calls into `application::mail_controller::MailController` (`src/application/mail_controller.rs`), which builds a typed request (e.g. `SendEmailRequest`) and parses addresses via `application::reply::split_addresses`.
3. `MailController` drives the protocol session in `service::protocols::imap` / `smtp` / `pop3` (`src/service/protocols/`).
4. Results (messages, folder state) are written to the SQLite cache in `data::message_cache` (`src/data/message_cache/`).
5. Presentation re-reads from `MessageCache`/`Account` (`src/data/message_cache/`, `src/data/account.rs`) to update the UI and fires accessibility announcements through `presentation::accessibility` (`src/presentation/accessibility.rs`).

### Startup Path

1. `src/main.rs` installs a panic hook first, so crashes always land in a log file even for the windowed subsystem.
2. Parses the command line (`presentation::command_line`), handling `--erase-all-data`, `--help`, `--version` before anything else opens a file.
3. Claims single-instance ownership via `application::running::claim()`.
4. Prepares the data folder (`common::paths::AppPaths`, migration from legacy locations) before logging opens any file.
5. Loads stored config to pick the log level, then calls `common::logging::init_logging`.
6. Hands off to `presentation::WxMailApp` (`src/presentation/wx_app.rs`) to build and show the main window.

**State Management:**
- Live protocol sessions (IMAP/SMTP/POP3) are held behind `tokio::sync::Mutex` inside `MailController` (`src/application/mail_controller.rs`), shared via `Arc`.
- UI-visible state (account list, message rows) is read fresh from `data::message_cache`/`data::account` rather than cached in presentation-layer structs long-term.
- Cross-instance state (the "another copy is already running" claim) is tracked through `application::running` using an OS-level marker, not in-process state.

## Key Abstractions

**`common::Error` / `common::Result`:**
- Purpose: the one error type crossing every layer boundary (`Config`, `Network`, `Authentication`, `Protocol`, `Security`, `Api { status, provider, message }`, `Other`).
- Examples: `src/common/error.rs`, used throughout `application/` and `service/`.
- Pattern: foreign errors (`rusqlite`, `reqwest`, protocol crates) are mapped to `Error` at the boundary where they enter; `unwrap`/`expect` are disallowed outside tests and `build.rs`.

**`MailController`:**
- Purpose: the single owner of live protocol sessions; presentation code never talks to `service::protocols` directly.
- Examples: `src/application/mail_controller.rs`.
- Pattern: typed request structs (`SendEmailRequest`) built from raw UI strings via helper functions (`addresses`, `recipient_address`) that reuse the same header parser (`service::mime::parse_addresses`) used for incoming mail, so outgoing and incoming addresses are parsed identically.

**`AppPaths` (`common::paths`):**
- Purpose: single source of truth for where all user data lives, overridable via `WIXEN_MAIL_DATA` for portable/removable installs.
- Examples: `src/common/paths.rs`.
- Pattern: one root, three subfolders (`config/`, `cache/`, `logs/`); nothing roams (kept local, not in the roaming profile).

**Sync modules (`*_sync.rs`):**
- Purpose: one file per PIM domain implementing the same synchronize-against-server pattern.
- Examples: `src/application/mail_sync.rs`, `caldav_sync.rs`, `contacts_sync.rs`, `tasks_sync.rs`, `collection_sync.rs`, `pop_sync.rs`.
- Pattern: each owns its own sync marker/cursor logic (see `sync_marker.rs`) so a failed sync can resume rather than re-fetching everything.

## Entry Points

**`src/main.rs`:**
- Location: `src/main.rs`
- Triggers: process launch (also handles CLI flags: `--erase-all-data`, `--help`, `--version`, and read-only/scan-target flags via `presentation::command_line`).
- Responsibilities: panic hook installation, single-instance claim, data folder preparation/migration, logging init, then hands control to `presentation::WxMailApp`.

**`src/lib.rs`:**
- Location: `src/lib.rs`
- Triggers: used by `main.rs` and by every test in `tests/` (integration tests link against the library, not the binary).
- Responsibilities: declares the four layers (`presentation`, `application`, `service`, `data`) plus `common` and `vendor` as public modules; no logic of its own.

## Architectural Constraints

- **Threading:** Single main UI thread for all wxdragon rendering and input; background work (sync, send, index) goes through `tokio` (full feature enabled in `Cargo.toml`) and channels (`async_channel::{Sender, Receiver}` is used in `wx_app.rs`) rather than blocking the UI thread. `MailController` uses `tokio::sync::Mutex`/`MappedMutexGuard` to serialize access to live protocol sessions across async tasks.
- **Platform coupling:** the accessibility bridge is more Windows-only than the rest of the codebase looks — `wxAccessible` (backing `set_accessible_name`) and `UiaRaiseNotificationEvent` (backing spoken/brailled announcements) exist only on Windows; both compile and silently do nothing elsewhere. Windows-only code sits behind `#[cfg(target_os = "windows")]` with a non-Windows fallback that keeps the crate building. See `src/presentation/accessibility/`.
- **No secrets in the database:** `data::message_cache` never stores credentials. Passwords, OAuth tokens, and CalDAV sign-ins each have exactly one owner module (`service::credentials`, `service::oauth`, `service::caldav`) so the code that erases them on uninstall names the same entries as the code that wrote them.
- **Additive-only schema:** `MessageCache` opens existing user databases in place; new tables use `CREATE TABLE IF NOT EXISTS`, new columns use `ensure_column_exists`. Nothing that has shipped is dropped or renamed (one documented exception: an unused OAuth-token table was dropped because leaving unrotated secrets in a copyable file was worse than the rule it broke).
- **No compiler-enforced layer boundary:** layer separation (presentation/application/service/data) is a naming and file-location convention, not a `pub(crate)` visibility wall — any module can `use crate::<any layer>` directly. New code should still route presentation → application → service → data and not skip layers, to keep this convention meaningful.

## Anti-Patterns

### Accessibility calls that look right but are not

**What happens:** Widgets get a screen-reader name via `wxWindow::set_name()` instead of the accessibility-tree APIs.
**Why it's wrong:** `set_name()` sets an internal wxWidgets identifier that never reaches UI Automation or MSAA. It compiles and passes the test suite while doing nothing for a screen reader.
**Do this instead:** Use `presentation::accessibility::names::set_accessible_name` / `set_accessible_name_and_description` (`src/presentation/accessibility/`), and verify with a real UIA scan (`Axe.Windows`) and `scripts/msaa-names.ps1`, not just a passing unit test.

### Implemented but never wired

**What happens:** A feature (storage, sync client, manager, and UI panel) is built end to end but no code path actually invokes it from the running application.
**Why it's wrong:** Every layer compiles, unit tests pass, and the feature is still absent from the running app — historically true for all eight PIM update variants at once.
**Do this instead:** Before calling a feature done, trace the call path from a real UI action (button press, menu item) through to the layer that was added, and confirm reachability, not just compilation. `tests/wired.rs` exists specifically to check this class of regression.

## Error Handling

**Strategy:** One shared error enum (`common::Error`) crosses every layer; `unwrap`/`expect` are forbidden outside tests and `build.rs`. Foreign errors are converted to `Error` at the boundary where they enter (e.g. `rusqlite::Error`, `reqwest::Error`, protocol-crate errors), preferably via `From` implementations rather than hand-written mapping functions.

**Patterns:**
- Protocol/network failures map to `Error::Network` or `Error::Protocol`.
- Provider REST failures map to `Error::Api { status, provider, message }`, preserving the HTTP status and which provider (Google/Microsoft) failed.
- Anything else falls back to `Error::Other(String)`.

## Cross-Cutting Concerns

**Logging:** `tracing` + `tracing-subscriber`, initialized once in `main.rs` via `common::logging::init_logging`, with the level read from stored settings before the config manager's usual path is available. Log files rotate under `%LOCALAPPDATA%\wixen-mail\logs\`. Never log a token, password, or message body.

**Validation:** untrusted input (message bodies, provider responses) is sanitized at the boundary where it enters — HTML message previews go through `ammonia` before rendering in the WebView (`presentation::html_renderer`), while preserving heading structure and link text for screen reader navigation.

**Authentication:** account passwords go to the Windows credential store via `service::credentials` (backed by `keyring`, DPAPI-encrypted, per-user); OAuth tokens via `service::oauth`; CalDAV sign-ins via `service::caldav`. No master key; nothing sensitive is ever written to `message_cache.db`.

---

*Architecture analysis: 2026-08-29*
