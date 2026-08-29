# External Integrations

**Analysis Date:** 2026-08-29

## APIs & External Services

**Mail Protocols:**
- IMAP - `src/service/protocols/imap.rs`, `src/service/protocols/imap/` (client: `async-imap`, TLS via `tokio-native-tls`/`native-tls`)
- POP3 - `src/service/protocols/pop3.rs`, `src/service/protocols/pop3/`
- SMTP - `src/service/protocols/smtp.rs` (client: `lettre`)
- XOAUTH2 - `src/service/protocols/xoauth2.rs` (SASL bridge letting IMAP/SMTP authenticate with an OAuth access token instead of a password)

**Google:**
- People API (contacts) - base `https://people.googleapis.com/v1`, `src/service/google_api.rs:397`
- Calendar API v3 - base `https://www.googleapis.com/calendar/v3`, `src/service/google_api.rs:398`
- Tasks API - `src/service/tasks_api.rs`
- OAuth authorize/token endpoints - `https://accounts.google.com/o/oauth2/v2/auth`, `https://oauth2.googleapis.com/token`, `src/service/oauth.rs:70-71`
- Scopes requested: `.../auth/contacts`, `.../auth/calendar`, `.../auth/tasks` (`src/service/oauth.rs`)
- Client: `reqwest` (project's own client), SDK/library: none (hand-rolled REST calls)
- Auth: `WIXEN_GMAIL_CLIENT_ID` / `WIXEN_GMAIL_CLIENT_SECRET`, or `[gmail]` in `oauth.toml`

**Microsoft / Outlook:**
- Microsoft Graph API v1.0 - base `https://graph.microsoft.com/v1.0`, `src/service/microsoft_graph.rs:392`
- Delta sync support for contacts and calendar (`@odata.deltaLink`), `src/service/microsoft_graph.rs`
- Scopes requested: `Contacts.ReadWrite`, `Calendars.ReadWrite`, `Tasks.ReadWrite` (`src/service/oauth.rs:96-99`)
- Client: `reqwest`, hand-rolled REST calls
- Auth: `WIXEN_OUTLOOK_CLIENT_ID` / `WIXEN_OUTLOOK_CLIENT_SECRET`, or `[outlook]` in `oauth.toml`
- Registration requires an "Authentication" platform of type "Mobile and desktop" with redirect URI `http://localhost:8087/oauth/callback`

**Google Safe Browsing (optional):**
- `https://safebrowsing.googleapis.com/v4`, `src/service/safebrowsing/client.rs:36`
- Used to download Google's public phishing/malware URL lists locally; links in messages are checked against the local copy, never sent to Google (per `oauth.toml.example` and `docs/privacy.md`)
- Opt-in, off by default; enabled via Settings → Advanced → "Check links against Google Safe Browsing"
- Auth: `WIXEN_SAFE_BROWSING_KEY`, or `[safe_browsing]` in `oauth.toml`
- Supporting modules: `src/service/safebrowsing/database.rs` (local list storage), `src/service/safebrowsing/rice.rs` (Rice/Golomb decoding of Google's compressed hash lists), `src/service/safebrowsing/urls.rs`

**CalDAV / iCalendar:**
- Generic CalDAV server support - `src/service/caldav.rs` (client: `reqwest` + `quick-xml` for WebDAV/XML, `icalendar` for parsing)
- iCalendar subscription feeds (read-only `.ics` URLs) - `src/service/ical_subscription.rs`
- Free/busy lookups - `src/service/free_busy.rs`

**LDAP:**
- Organizational directory lookup for name/address autocomplete - `src/service/directory.rs` (client: `ldap3`, TLS via rustls/ring)

## Data Storage

**Databases:**
- SQLite (embedded, `rusqlite` with `bundled` build) - single local cache file `message_cache.db`
  - Schema and access split by domain across `src/data/message_cache/`: `accounts.rs`, `attachment_content.rs`, `bodies.rs`, `calendar.rs`, `calendars.rs`, `contacts.rs`, `drafts.rs`, `filters.rs`, `folders.rs`, `messages.rs`, `notes.rs`, `outbox.rs`, `reminders.rs`, `saved_searches.rs`, `searching.rs`, `signatures.rs`, `signed_original.rs`, `tags.rs`, `tasks.rs`
  - Higher-level cache API/orchestration: `src/service/cache.rs`
  - Message bodies stored deflate-compressed in the cache (`flate2`) for space savings (~4.6:1 on real mail)
  - Custom `LOWER` SQL function registered via `rusqlite`'s `functions` feature for accent-aware case folding in search

**File Storage:**
- Local filesystem only. Data folder under the platform's standard app-data directory (resolved via `dirs`), e.g. `%LOCALAPPDATA%\wixen-mail\` on Windows
- Attachment content cached in SQLite (`src/data/message_cache/attachment_content.rs`), not stored as loose files
- Sound-scheme packs imported from `.zip` archives (`sound-schemes/`, `src/service/` earcon code)
- Outlook `.pst` files read directly from disk for one-time import (`src/service/outlook_data_file.rs`, `outlook-pst` crate)

**Caching:**
- The SQLite message cache described above is the primary cache layer for mail, calendar, contacts, and tasks synced from remote accounts. No external cache service (Redis, etc.) is used.

## Authentication & Identity

**Auth Provider:**
- No third-party identity/auth-as-a-service provider. Each mail/calendar account authenticates directly against its own server or provider (IMAP/SMTP password, or OAuth2 with the account's own provider — Google, Microsoft).
- OAuth 2.0 with PKCE implemented directly using the `oauth2` crate, `src/service/oauth.rs`. Components:
  - `OAuthProvider` - provider metadata (endpoints, scopes)
  - `OAuthTokenSet` - access/refresh tokens with expiry
  - `AuthManager` - per-account token lifecycle (authorize, refresh, retrieve)
  - Local redirect capture via `tiny_http` on `localhost` during the browser-based consent flow, launched with `open`
- Google issues a single token covering all `googleapis.com` resources; Microsoft issues per-resource tokens scoped to `graph.microsoft.com` (documented at `src/service/oauth.rs:823-846`)
- Tokens and passwords are stored in the OS credential store via `keyring`, behind the seam in `src/service/secret_store.rs` (test builds swap in an in-memory map instead of touching the real Windows Credential Manager)

## Monitoring & Observability

**Error Tracking:**
- None. No external error-tracking/crash-reporting service integrated.

**Logs:**
- Local file logging via `tracing` + `tracing-subscriber` (`env-filter`, `fmt`) + `tracing-appender`, written under the app's data folder with a `wixen-mail` log prefix. No remote log shipping.

## CI/CD & Deployment

**Hosting:**
- Distributed as a Windows installer (Inno Setup, `installer/Wixen-Mail-Setup.iss`); no server-side hosting component. GitHub is used for source and release artifacts (`repository = "https://github.com/PratikP1/Wixen-Mail"`).

**CI Pipeline:**
- GitHub Actions, `.github/workflows/`:
  - `ci.yml` - main test suite, runs on `windows-latest`, `cargo test --no-fail-fast`
  - `accessibility.yml` - accessibility-focused checks
  - `nvda.yml` - NVDA screen-reader-focused checks (`nvda-tests/`)
  - `mutants.yml` - mutation testing (`cargo-mutants`)
  - `release.yml` - release build/packaging

## Environment Configuration

**Required env vars (all optional; only needed to enable the corresponding integration):**
- `WIXEN_GMAIL_CLIENT_ID`, `WIXEN_GMAIL_CLIENT_SECRET` - Gmail/Google OAuth
- `WIXEN_OUTLOOK_CLIENT_ID`, `WIXEN_OUTLOOK_CLIENT_SECRET` - Microsoft/Outlook OAuth
- `WIXEN_SAFE_BROWSING_KEY` - Google Safe Browsing API key
- `WIXEN_BUILD` - build/commit identifier embedded at compile time (set by `scripts/build-installer.sh`, not a runtime secret)

**Secrets location:**
- Build-time OAuth client credentials: `oauth.toml` at `%LOCALAPPDATA%\wixen-mail\config\oauth.toml` (never committed; `oauth.toml.example` is the template), or the environment variables above
- Per-account runtime secrets (passwords, OAuth access/refresh tokens): OS credential store (Windows Credential Manager) via `keyring`, accessed only through `src/service/secret_store.rs`

## Webhooks & Callbacks

**Incoming:**
- OAuth redirect callback captured by a short-lived local HTTP server (`tiny_http`) on `http://localhost:8087/oauth/callback` during sign-in, `src/service/oauth.rs`. This is a local-loopback callback for the interactive OAuth flow, not an internet-facing webhook.

**Outgoing:**
- None (no outbound webhook notifications sent by the application).

---

*Integration audit: 2026-08-29*
</content>
