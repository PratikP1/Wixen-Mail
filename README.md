# Wixen Mail

Wixen Mail is an accessibility-first email client built with Rust, egui, and AccessKit.
It focuses on complete keyboard navigation, screen-reader support, and practical multi-account workflows.

## Highlights

- Accessible integrated UI with keyboard-first navigation
- Multiple account management and account-scoped data isolation
- IMAP/SMTP support, plus POP3 command-surface implementation
- Threaded message view, HTML sanitization/rendering helpers, and attachment handling
- Message rules, contact management, OAuth manager, and offline outbox queue
- Beta readiness diagnostics for quick operational checks

## Quick Start

```bash
git clone https://github.com/PratikP1/Wixen-Mail.git
cd Wixen-Mail
cargo build
cargo run --bin ui_integrated
```

## Development Commands

```bash
# Run tests
cargo test --quiet

# Build
cargo build

# Quality gates (all passing as of 2026-02-27)
cargo fmt --check
cargo clippy -- -D warnings
```

## Documentation

Canonical, actively maintained docs:

- [docs/USER_GUIDE.md](docs/USER_GUIDE.md)
- [docs/KEYBOARD_SHORTCUTS.md](docs/KEYBOARD_SHORTCUTS.md)
- [docs/PROVIDER_SETUP.md](docs/PROVIDER_SETUP.md)
- [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)
- [docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md) (single source of truth for status/tasks)
- [ACCESSIBILITY.md](ACCESSIBILITY.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [ROADMAP.md](ROADMAP.md)

Historical phase/session requirement files in the repository root are retained for traceability but should be treated as implementation records, not the current product overview.

## Current Status (2026-02-27)

All quality gates pass clean (`cargo fmt`, `cargo clippy`, `cargo test`).

**Implemented:** Multi-account management, composition with attachments & signatures, contact management (full CRUD + vCard import/export), advanced search with date/sender/attachment filters, message rules engine, OAuth UI (token exchange pending real HTTP calls), IMAP IDLE push, POP3, PGP/S-MIME detection, phishing scoring, offline queue infrastructure.

**Remaining for v1.0:** Real OAuth token exchange (2 stub functions), offline mode UI wiring, spell check integration, compose account selector, contact groups, expanded test coverage. See [docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md) for full details.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under [MIT](LICENSE).
