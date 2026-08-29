# Technology Stack

**Analysis Date:** 2026-08-29

## Languages

**Primary:**
- Rust (edition 2024, `rust-version = 1.87`) - entire application: `src/`, `build.rs`, `search-handler/`

**Secondary:**
- JavaScript (ECMAScript, embedded) - mail filter rule scripting evaluated via `boa_engine` (dev-dependency, used at runtime for filter conditions; see `src/service/` filter evaluation and `boa_engine` in `Cargo.toml`)
- Bash/PowerShell scripts - build and packaging automation under `scripts/`
- Inno Setup script - `installer/Wixen-Mail-Setup.iss` (Windows installer definition)

## Runtime

**Environment:**
- Native Windows desktop application (GUI subsystem), compiled to `wixen-mail.exe`
- Windows-specific dependencies gated behind `[target.'cfg(windows)'.dependencies]` in `Cargo.toml` (COM-based spell checker, console attach, named mutex for single-instance)

**Package Manager:**
- Cargo (Rust's package manager)
- Lockfile: present (`Cargo.lock`)

## Frameworks

**Core:**
- `wxdragon` `=0.9.17` (pinned exact version) - wxWidgets Rust bindings, the GUI framework; features: `aui`, `richtext`, `webview`
- `tokio` `1` (`full` features) - async runtime for network I/O (IMAP/POP3/SMTP/HTTP)
- `rusqlite` `0.40` (`bundled`, `functions`) - embedded SQLite for the local message/data cache; `functions` feature used to register a custom Unicode-aware `LOWER`

**Testing:**
- Built-in `cargo test` plus `tokio-test = "0.4"` for async test helpers
- `tempfile = "3.14"` for isolated filesystem/db fixtures in tests
- `boa_engine = "0.20"` doubles as dev-dependency (used both for filter-script execution and test tooling)

**Build/Dev:**
- `winresource = "0.1"` (Windows build-dependency) - embeds the executable's manifest, icon, and version metadata via `build.rs`
- `clippy` lints enforced at the crate level (`[lints.clippy]` in `Cargo.toml`): `significant_drop_in_scrutinee = "deny"`, `await_holding_lock = "deny"` — guards against a specific UI-thread deadlock class documented inline in `Cargo.toml`

## Key Dependencies

**Critical:**
- `lettre` `0.11` (`tokio1-native-tls`, `smtp-transport`, `builder`) - SMTP sending
- `async-imap` `0.11.3` (`runtime-tokio`, default features off) - IMAP protocol client, see `src/service/protocols/imap.rs`, `src/service/protocols/imap/`
- `mail-parser` `0.11` - MIME/message parsing
- `reqwest` `0.13` (`json`, `rustls`, `form`) - HTTP client for all REST API calls (Google, Microsoft Graph, Safe Browsing)
- `oauth2` `4` (default features off) - OAuth 2.0 + PKCE flows; project's own `reqwest` client is used instead of the crate's bundled HTTP stack (see `src/service/oauth.rs`)
- `tiny_http` `0.12` - local HTTP server that captures the OAuth redirect on `localhost`
- `keyring` `4` - OS credential store (Windows Credential Manager) for tokens and passwords, see `src/service/secret_store.rs`
- `icalendar` `0.17`, `quick-xml` `0.41` - CalDAV/iCalendar parsing, see `src/service/caldav.rs`, `src/service/ical_subscription.rs`
- `ldap3` `0.12.1` (`tls-rustls-ring`, default features off) - organizational directory lookups, see `src/service/directory.rs`
- `x509-parser` `0.18`, `ring` `0.17` - S/MIME certificate parsing and signature verification, see `src/service/signed_mail.rs`
- `outlook-pst` `1.2.0` - reading Outlook `.pst` data files (Microsoft's own crate, read-only), see `src/service/outlook_data_file.rs`
- `pdfpurr` `0.4.0` (default features off) - PDF handling (sibling project by the same author)
- `ammonia` `4.0`, `html-escape` `0.2` - HTML sanitization for message bodies
- `aes-gcm` `0.11`, `sha2` `0.11.0` - encryption/hashing primitives
- `spellbook` `0.4` - pure-Rust Hunspell-compatible spell checking (non-Windows path)
- `rodio` `0.22.2` - cross-platform audio playback for earcons (WAV/OGG/MP3/FLAC via Symphonia)
- `zip` `7.2.0` (`deflate` only), `flate2` `1.1.9` - sound-scheme pack import and message-body compression in the cache

**Infrastructure:**
- `dirs` `6.0` - platform-standard data/config directory resolution
- `tracing`, `tracing-subscriber` (`env-filter`, `fmt`), `tracing-appender` - structured logging, see `src/` logging setup
- `serde`, `serde_json`, `toml` - serialization and config parsing
- `chrono`, `chrono-tz`, `iana-time-zone` - date/time and IANA timezone handling for calendar features
- `windows` `0.62.2` (Windows-only) - `Win32_Globalization`, `Win32_System_Com`, `Win32_Foundation`, `Win32_System_Console`, `Win32_System_Threading`, `Win32_Security` - COM-based platform spell checker, console attach for `--help`, single-instance named mutex

## Configuration

**Environment:**
- OAuth client credentials via `oauth.toml` (copied from `oauth.toml.example`), stored at `%LOCALAPPDATA%\wixen-mail\config\oauth.toml` on Windows, or environment variables: `WIXEN_GMAIL_CLIENT_ID`, `WIXEN_GMAIL_CLIENT_SECRET`, `WIXEN_OUTLOOK_CLIENT_ID`, `WIXEN_OUTLOOK_CLIENT_SECRET`, `WIXEN_SAFE_BROWSING_KEY`
- `WIXEN_BUILD` env var - embeds a build/commit identifier at compile time via `build.rs`, set by `scripts/build-installer.sh`; empty for ordinary `cargo build`

**Build:**
- `Cargo.toml` - dependency and lint configuration
- `build.rs` - Windows manifest, icon, and version-info embedding (`wixen-mail.exe.manifest`, `assets/icon.ico`)
- `.github/workflows/ci.yml` - main test suite (Windows runner, `cargo test`, `--no-fail-fast`)
- `.github/workflows/accessibility.yml`, `nvda.yml` - accessibility- and screen-reader-focused CI checks
- `.github/workflows/mutants.yml` - mutation testing pipeline
- `.github/workflows/release.yml` - release build/packaging pipeline
- `installer/Wixen-Mail-Setup.iss` - Inno Setup script defining the Windows installer

## Platform Requirements

**Development:**
- Rust toolchain 1.87+ (edition 2024)
- Windows (primary target; CI runs on `windows-latest`; the accessibility/NVDA-focused workflows assume Windows)

**Production:**
- Windows desktop, distributed as an installed executable (`wixen-mail.exe`) via the Inno Setup installer
- Ships unsigned through alpha/beta distribution

---

*Stack analysis: 2026-08-29*
</content>
