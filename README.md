<img src="assets/brand/wixen-fox.png" alt="The Wixen fox" width="96">

# Wixen Mail

Wixen Mail is an accessibility-first email client built with Rust on native Windows controls.
It focuses on complete keyboard navigation, screen-reader support, and practical multi-account workflows with a native Windows look and feel.

## Highlights

Read [Status](#status) first. This list describes what the project is for, not
what is finished.

- Native Windows controls with toolbar, three-pane layout, and modern styling
- Keyboard navigation throughout, and an accessibility layer built for NVDA, JAWS and Narrator. An automated suite drives real NVDA in CI against specific interactions; most of the application has not had a full manual pass
- Multiple account management with provider auto-detection and OAuth 2.0
- SMTP sending over TLS, with an outbox that retries rather than losing a failed send
- Composition with formatting toolbar, attachments, signatures, and preview-before-send
- Contact management with vCard import/export, groups, and autocomplete
- Search across subject, sender, and preview text, a message rules engine, and tag-based filtering
- Offline mode with outbox queue and sync-on-reconnect
- Passwords and sign-in tokens kept in the Windows credential store, never in a file

## Installing

Download `Wixen-Mail-Setup-<version>.exe` from the
[releases page](https://github.com/PratikP1/Wixen-Mail/releases) and run it. Installing for
yourself needs no administrator rights. See [installing and uninstalling](docs/installing.md) for the
silent install switches, where your data is kept, and what uninstalling removes.

## Building from source

```bash
git clone https://github.com/PratikP1/Wixen-Mail.git
cd Wixen-Mail
cargo build
cargo run
```

To build the setup file, which needs [Inno Setup 6](https://jrsoftware.org/isdl.php):

```bash
bash scripts/build-installer.sh
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
# Just the tests
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
- [Privacy](docs/privacy.md)

### Technical

- [Architecture](docs/architecture.md)
- [Brand](docs/brand.md)
- [Roadmap](docs/roadmap.md)
- [Implementation Status](docs/IMPLEMENTATION_STATUS.md)
- [How the interface is built](docs/wxdragon-integration.md)

### Development history

- [Implementation History](docs/development/implementation-history.md)
- [Requirements Backlog](docs/development/requirements-backlog.md)
- [Moving to the current interface toolkit](docs/development/wxdragon-migration.md)

## Status

The project is pre-beta. It can send and receive mail, over IMAP, POP3 and
SMTP, and it can sync contacts, calendars and tasks with Google and Microsoft.
Reading mail is the part that has been used. Everything that writes to a
server is experimental, has never run against a real account, and is gated by
a setting called Allow Changes, which
[the testing guide](docs/ALPHA_TESTING.md) explains before anything else.

Most of this application has not been verified with a real screen reader yet,
and for this project that is the bar that matters. An automated suite drives
real NVDA in CI against specific interactions, but a full manual pass has not
been done. See
[what is built and what is not](docs/IMPLEMENTATION_STATUS.md), which is
written to be believed rather than to sell the project.

## Contributing

See [how to contribute](docs/contributing.md).

## Acknowledgements

The message reader window follows the design of
[Paperback](https://github.com/trypsynth/paperback) by Quin Gillespie, an
accessible document reader built on the same interface toolkit. Paperback
renders every format it supports into a read-only rich text control inside a
tabbed notebook, and keeps its WebView for a separate optional dialog. It got
there for the same reasons we did, after we had learned them the hard way, and
copying a shape that already works was better than inventing a worse one.
Paperback is MIT licensed.

## License

Licensed under [MIT](LICENSE).
