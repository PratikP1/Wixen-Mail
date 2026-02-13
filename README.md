# Wixen Mail

A fully accessible, light-weight, open source mail client using Rust and WXDragon. The client aims to be fast and looks similar to Thunderbird with first-class support for screen readers and keyboard navigation.

## Features (Planned)

- 🦀 **Built with Rust**: Fast, safe, and reliable
- ♿ **Fully Accessible**: First-class support for screen readers (NVDA, JAWS, Narrator)
- ⌨️ **Keyboard-First**: Complete keyboard navigation support
- 🪟 **Windows Native**: Native Windows UI using WXDragon
- 📧 **Multiple Protocols**: IMAP, SMTP, and POP3 support
- 🔒 **Security Focused**: Built-in encryption and secure credential storage
- ⚡ **Lightweight**: Low memory footprint and fast startup
- 🎨 **Customizable**: Themes and customizable keyboard shortcuts

## Project Status

🚧 **This project is in early development stage.** We are currently setting up the foundation and planning the architecture.

See [ROADMAP.md](ROADMAP.md) for detailed project timeline and milestones.

## Documentation

- [ROADMAP.md](ROADMAP.md) - Project roadmap and task list
- [ACCESSIBILITY.md](ACCESSIBILITY.md) - Accessibility features and keyboard shortcuts
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture and design

## Building from Source

### Prerequisites

- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs/))
- Windows 10 or later (for now)
- WXDragon UI libraries (setup instructions coming soon)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/PratikP1/Wixen-Mail.git
cd Wixen-Mail

# Build the project
cargo build

# Run the application
cargo run
```

### Development Build

```bash
# Build in debug mode
cargo build

# Run with logging
RUST_LOG=debug cargo run
```

### Release Build

```bash
# Build optimized release version
cargo build --release
```

## Contributing

We welcome contributions! Please see [ROADMAP.md](ROADMAP.md) for areas where you can help.

### Guidelines

1. Fork the repository
2. Create a feature branch
3. Make your changes with appropriate tests
4. Ensure accessibility features are maintained
5. Submit a pull request

Please ensure:
- Code follows Rust style guidelines (use `cargo fmt`)
- All tests pass (`cargo test`)
- Clippy checks pass (`cargo clippy`)
- Accessibility features are not compromised

## Accessibility

Wixen Mail is designed with accessibility as a core principle, not an afterthought. We are committed to:

- WCAG 2.1 Level AA compliance
- Full screen reader support (NVDA, JAWS, Narrator)
- Complete keyboard navigation
- High contrast mode support
- Customizable display settings

For detailed accessibility information, see [ACCESSIBILITY.md](ACCESSIBILITY.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Mozilla Thunderbird
- Built with the amazing Rust community
- Accessibility guidance from screen reader users and advocates

## Contact

- GitHub Issues: [Report bugs or request features](https://github.com/PratikP1/Wixen-Mail/issues)
- GitHub Discussions: [Ask questions or discuss ideas](https://github.com/PratikP1/Wixen-Mail/discussions)

## Project Goals

Our mission is to create a mail client that:
1. **Is accessible to everyone**, including users who rely on assistive technologies
2. **Performs efficiently** with minimal resource usage
3. **Respects user privacy** with no telemetry or tracking
4. **Provides excellent user experience** for both power users and beginners
5. **Remains open source** and community-driven

---

**Note**: This is an early-stage project. Stars, feedback, and contributions are highly appreciated!
 
