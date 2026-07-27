# Wixen Mail

Wixen Mail is an accessibility-first email client built with Rust and wxdragon (wxWidgets).
It focuses on complete keyboard navigation, screen-reader support, and practical multi-account workflows with a native Windows look and feel.

## Highlights

Read [Status](#status) first. This list describes what the project is for, not
what is finished.

- Native wxWidgets UI with toolbar, three-pane layout, and modern styling
- Keyboard navigation throughout, and an accessibility layer built for NVDA, JAWS and Narrator (not yet verified against a live screen reader)
- Multiple account management with provider auto-detection and OAuth 2.0
- SMTP sending over TLS, with an outbox that retries rather than losing a failed send
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
# The four checks CI runs, in the same order
bash scripts/check.sh
```

Use the script rather than the four commands separately. Cargo shares build
fingerprints between `check`, `build`, `test`, and `clippy`, so a clippy run
after a build can be treated as fresh and report success without linting
anything. The script touches `src/lib.rs` first to force the work.

```bash
# Just the tests (394 unit, 30 integration)
cargo test --all-targets

# Just a build
cargo build
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

## Status

All quality gates pass clean (`cargo fmt`, `cargo clippy`, `cargo test`).
150 unit tests and 25 integration tests passing with 0 warnings.

The project is pre-beta, at `0.1.0-alpha.12`. **It can send mail and it cannot
receive mail:** the IMAP and POP3 modules perform no network I/O yet, and nothing
in the window is wired to them, because showing invented folders as your own mail
would be worse than showing none.

What does work: sending, local storage for contacts, calendars, tasks, notes and
reminders with their panels, and the accessibility layer. See
[docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md), which is written
to be believed rather than to sell the project.

## Contributing

See [CONTRIBUTING.md](docs/contributing.md).

## Acknowledgements

The message reader window follows the design of
[Paperback](https://github.com/trypsynth/paperback) by Quin Gillespie, an
accessible document reader built on the same wxWidgets bindings. Paperback
renders every format it supports into a read-only rich text control inside a
tabbed notebook, and keeps its WebView for a separate optional dialog. It got
there for the same reasons we did, after we had learned them the hard way, and
copying a shape that already works was better than inventing a worse one.
Paperback is MIT licensed.

## License

Licensed under [MIT](LICENSE).
