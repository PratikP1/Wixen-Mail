# Codebase Structure

**Analysis Date:** 2026-08-29

## Directory Layout

```
Wixen-Mail/
├── src/
│   ├── main.rs              # Process entry point (binary)
│   ├── lib.rs                # Library root, declares the four layers
│   ├── common/                # Shared types/utilities, no business logic
│   ├── data/                  # SQLite cache, account/config storage
│   │   └── message_cache/       # The one database module
│   ├── application/           # Business logic, one file per concern (69 files)
│   ├── service/                # Protocol clients and external services
│   │   ├── protocols/            # imap/, pop3/, smtp.rs
│   │   ├── safebrowsing/
│   │   └── spellcheck/
│   ├── presentation/           # wxdragon UI + accessibility
│   │   └── accessibility/        # Screen reader bridge, names, feedback
│   └── vendor/                  # Vendored/adapted third-party code
├── tests/                    # Cross-layer integration tests (link against the lib)
├── nvda-tests/                # Node/Jest-based NVDA/screen-reader test harness
├── guards/                   # guards.toml: mutation-guard records for scripts/guards.sh
├── scripts/                  # check.sh, mutants.sh, guards.sh, msaa-names.ps1, build-installer.sh
├── docs/                     # architecture.md, principles.md, changelog.md, KEYBOARD_SHORTCUTS.md
├── installer/                 # Windows installer build assets
├── sound-schemes/              # Earcon/sound scheme assets (e.g. soft-chimes/)
├── search-handler/             # Separate Rust crate/binary (Windows search integration)
├── assets/                    # Brand assets, icons
├── .planning/                 # GSD planning artifacts (this document lives here)
├── build.rs                  # Build script (version stamping, resource embedding)
├── Cargo.toml / Cargo.lock    # Crate manifest (single binary crate `wixen_mail`)
└── oauth.toml.example         # Tracked template for the gitignored oauth.toml
```

## Directory Purposes

**`src/common/`:**
- Purpose: cross-layer primitives with zero business logic.
- Contains: `error.rs` (the `Error`/`Result` type), `paths.rs` (data folder root), `types.rs`, `logging.rs`, `version.rs`, `moment.rs` (stored-moment shapes); test-only helpers gated behind `#[cfg(test)]`: `answering.rs` (loopback server for protocol tests), `temp_home.rs`, `what_ships.rs`.
- Key files: `src/common/error.rs`, `src/common/paths.rs`.

**`src/data/`:**
- Purpose: persistence layer.
- Contains: `account.rs` (Account model/storage), `config.rs` (`ConfigManager`), `email_providers.rs` (static provider metadata for autodiscovery), `message_cache/` (the SQLite database module — schema, queries, migrations).
- Key files: `src/data/message_cache/`, `src/data/account.rs`, `src/data/config.rs`.

**`src/application/`:**
- Purpose: business logic — one file per concern, named after the behavior it implements (verb/gerund style) rather than a noun-plus-suffix.
- Contains: mail send/receive orchestration (`mail_controller.rs`, `mail_session.rs`, `mail_sync.rs`, `mail_auth.rs`), sync per PIM domain (`caldav_sync.rs`, `contacts_sync.rs`, `tasks_sync.rs`, `collection_sync.rs`, `pop_sync.rs`), composition (`draft_message.rs`, `attaching.rs`, `sign_off.rs`, `sending_later.rs`), filtering/search (`filters.rs`, `search.rs`, `saved_searches.rs`, `categories.rs`), PIM item lifecycle (`new_item.rs`, `editing.rs`, `deletions.rs`, `occurrences.rs`, `repeating.rs`, `invitations.rs`).
- Key files: `src/application/mail_controller.rs` (hub for live protocol sessions), `src/application/accounts.rs`.

**`src/service/`:**
- Purpose: protocol implementations and external/cross-cutting services.
- Contains: `protocols/` (`imap/`, `pop3/` subfolders plus `smtp.rs`, `xoauth2.rs`), provider REST clients (`google_api.rs`, `microsoft_graph.rs`), calendar/CalDAV (`caldav.rs`, `ical_subscription.rs`, `free_busy.rs`, `vtimezone.rs`), security (`security.rs`, `signed_mail.rs`, `safety.rs`, `safebrowsing/`), credentials/OAuth (`credentials.rs`, `secret_store.rs`, `oauth.rs`, `oauth_credentials.rs`), parsing/output (`mime.rs`, `pdf.rs`, `outlook_data_file.rs`, `mailbox_archive.rs`), `spellcheck/`.
- Key files: `src/service/protocols/imap.rs`, `src/service/caldav.rs`, `src/service/credentials.rs`.

**`src/presentation/`:**
- Purpose: wxdragon UI and its accessibility exposure.
- Contains: main window (`wx_app.rs`), per-screen modules prefixed `wx_*` (`wx_compose.rs`, `wx_settings.rs`, `wx_account_manager.rs`, `wx_calendar_module.rs`, `wx_contacts_module.rs`, `wx_tasks_module.rs`, `wx_notes_module.rs`, `wx_reminders_module.rs`, `wx_reader.rs`, `wx_thread_view.rs`, `wx_tray.rs`), shared UI infrastructure without the prefix (`theme.rs`, `panes.rs`, `managers.rs`, `status_line.rs`, `message_rows.rs`, `message_columns.rs`, `pim_rows.rs`), and `accessibility/` (screen reader bridge, `names.rs`, `feedback` events).
- Key files: `src/presentation/wx_app.rs`, `src/presentation/accessibility.rs`, `src/presentation/accessibility/`.

**`src/vendor/`:**
- Purpose: vendored or adapted third-party code that doesn't fit the crate dependency model.
- Committed: yes.

**`tests/`:**
- Purpose: cross-layer integration tests that link against `wixen_mail` as a library (not the binary).
- Key files: `tests/integration_tests.rs`, `tests/wired.rs` (checks features are actually reachable, not just compiled), `tests/house_style.rs` (repo-wide convention checks).

**`nvda-tests/`:**
- Purpose: separate Node/Jest project driving NVDA/screen-reader assertions against the running app.
- Contains: its own `node_modules/`, `tests/`, `helpers/`, `results/`. Not part of the Rust crate.

**`guards/`:**
- Purpose: `guards.toml` records the exact edit and the tests expected to fail for each mutation guard, checked by `scripts/guards.sh`.

**`scripts/`:**
- Purpose: the commands CLAUDE.md requires be run instead of raw cargo/clippy invocations: `check.sh` (build+test+clippy with fingerprint-busting), `mutants.sh` (scoped mutation testing), `guards.sh` (guard verification), `msaa-names.ps1` (MSAA accessibility name scan), `build-installer.sh`.

**`search-handler/`:**
- Purpose: a separate Rust crate (own `src/`, own `target/`) for Windows search integration, not part of the `wixen_mail` binary crate.

**`sound-schemes/`:**
- Purpose: earcon/audio asset packs (e.g. `soft-chimes/sounds/`) used for accessible audio cues.

## Key File Locations

**Entry Points:**
- `src/main.rs`: process startup, CLI parsing, single-instance claim, logging init.
- `src/lib.rs`: library root; every layer is a `pub mod` here.

**Configuration:**
- `Cargo.toml`: crate manifest, dependency versions.
- `oauth.toml` (gitignored) / `oauth.toml.example` (tracked template): OAuth client credentials.
- `.cargo/`: cargo config.

**Core Logic:**
- `src/application/mail_controller.rs`: the hub bridging UI to live protocol sessions.
- `src/data/message_cache/`: the one SQLite database.
- `src/common/paths.rs`: single owner of the on-disk data folder layout.

**Testing:**
- `tests/`: integration tests linking against the library.
- `guards/guards.toml` + `scripts/guards.sh`: mutation-guard records.
- `nvda-tests/`: screen-reader-driven tests (separate Node project).
- `#[cfg(test)] mod tests` blocks beside the code they cover (unit tests), per CLAUDE.md.

## Naming Conventions

**Files:**
- `src/application/`: named after the behavior/verb, not a noun+suffix — `closing.rs`, `filing.rs`, `handover.rs`, `answering.rs`, not `CloseManager.rs` or `filing_service.rs`.
- `src/presentation/`: UI screens prefixed `wx_` (`wx_compose.rs`, `wx_settings.rs`); shared UI infrastructure has no prefix (`theme.rs`, `panes.rs`).
- `src/service/`: named after the protocol or service directly (`caldav.rs`, `oauth.rs`, `security.rs`).
- Sync modules end in `_sync.rs` (`mail_sync.rs`, `caldav_sync.rs`, `contacts_sync.rs`, `tasks_sync.rs`, `collection_sync.rs`, `pop_sync.rs`).

**Directories:**
- Layer directories are single lowercase nouns (`common`, `data`, `application`, `service`, `presentation`).
- Multi-protocol services get their own subdirectory (`service/protocols/imap/`, `service/protocols/pop3/`, `service/safebrowsing/`, `service/spellcheck/`).

## Where to Add New Code

**New PIM/mail feature (business logic):**
- Implementation: a new file in `src/application/`, named after the behavior it performs (follow the existing verb/gerund convention, not `XManager.rs`).
- If it syncs against a server, mirror the `*_sync.rs` pattern (e.g. `src/application/caldav_sync.rs`) including its own sync-marker handling (`src/application/sync_marker.rs`).
- Tests: `#[cfg(test)] mod tests` in the same file; cross-layer behavior goes in `tests/`.

**New protocol client or external service:**
- Implementation: `src/service/protocols/` for a mail protocol, or a new top-level file in `src/service/` for anything else (a new provider API, a new security service).
- Keep the transport thin and parsing pure, per CLAUDE.md, so parsing/error-mapping is unit-testable without a live server.

**New UI screen/dialog:**
- Implementation: `src/presentation/wx_<name>.rs` if it is a distinct window/panel; extend `src/presentation/managers.rs` or `panes.rs` if it's shared infrastructure.
- Accessibility names must go through `presentation::accessibility::names::set_accessible_name` (never `wxWindow::set_name()`), and any dynamic announcement through `presentation::accessibility::screen_reader`/feedback events.

**New data stored persistently:**
- If it's cacheable mail/PIM data: extend `src/data/message_cache/` with `CREATE TABLE IF NOT EXISTS` / `ensure_column_exists` — never drop or rename a shipped column.
- If it's a secret: route through `service::credentials`, `service::oauth`, or `service::caldav` (one owner per secret name) — never into `message_cache.db`.

**Utilities:**
- Shared, business-logic-free helpers: `src/common/`.
- Test-only helpers: `src/common/` gated behind `#[cfg(test)]` (see `answering.rs`, `temp_home.rs`, `what_ships.rs` for the existing pattern).

## Special Directories

**`target/`:**
- Purpose: Cargo build output.
- Generated: Yes.
- Committed: No.

**`mutants.out.old/`:**
- Purpose: stale output from a previous mutation-testing run.
- Generated: Yes.
- Committed: No (excluded from analysis; do not treat as current).

**`dist/`:**
- Purpose: build/installer output artifacts.
- Generated: Yes.
- Committed: No.

**`.planning/`:**
- Purpose: GSD workflow planning artifacts, including this codebase map.
- Generated: partially (documents like this one are generated; phase/plan content is authored).
- Committed: Yes.

---

*Structure analysis: 2026-08-29*
