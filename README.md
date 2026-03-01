# Wixen Mail

Wixen Mail is an accessibility-first email client built with Rust and wxdragon (wxWidgets).
It focuses on complete keyboard navigation, screen-reader support, and practical multi-account workflows with a native Windows look and feel.

## Highlights

- Native wxWidgets UI with toolbar, three-pane layout, and modern styling
- Full keyboard navigation with 25+ shortcuts and screen reader support (NVDA, JAWS, Narrator)
- Multiple account management with provider auto-detection and OAuth 2.0
- IMAP/SMTP support with IDLE push notifications, plus POP3
- Composition with formatting toolbar, attachments, signatures, and preview-before-send
- Contact management with vCard import/export, groups, and autocomplete
- Advanced search (FTS), message rules engine, and tag-based filtering
- Offline mode with outbox queue and sync-on-reconnect
- AES-256-GCM credential encryption and phishing detection

## Quick Start

```bash
git clone https://github.com/PratikP1/Wixen-Mail.git
cd Wixen-Mail
cargo build
cargo run --bin ui_integrated
```

## Development Commands

```bash
# Run tests (150 unit + 26 integration)
cargo test --quiet

# Build
cargo build

# Quality gates
cargo fmt --check
cargo clippy -- -D warnings
```

## Documentation

### User-facing

- [User Guide](docs/USER_GUIDE.md)
- [Keyboard Shortcuts](docs/KEYBOARD_SHORTCUTS.md)
- [Provider Setup](docs/PROVIDER_SETUP.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Accessibility Guide](docs/accessibility.md)

### Technical

- [Architecture](docs/architecture.md)
- [Roadmap](docs/roadmap.md)
- [Implementation Status](docs/IMPLEMENTATION_STATUS.md)
- [wxdragon Integration](docs/wxdragon-integration.md)

### Development history

- [Implementation History](docs/development/implementation-history.md)
- [Requirements Backlog](docs/development/requirements-backlog.md)
- [wxdragon Migration Notes](docs/development/wxdragon-migration.md)

## Current Status (2026-03-01)

All quality gates pass clean (`cargo fmt`, `cargo clippy`, `cargo test`).
150 unit tests and 25 integration tests passing with 0 warnings.

The project is at release-candidate status. All v1.0 feature gaps have been closed including OAuth token exchange, offline mode wiring, spell check, contact groups, preview-before-send, and comprehensive test coverage. See [docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md) for full details.

## Contributing

See [CONTRIBUTING.md](docs/contributing.md).

## License

Licensed under [MIT](LICENSE).
