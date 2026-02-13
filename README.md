# Wixen Mail

A fully accessible, lightweight, open-source email client built with Rust and egui. Designed with first-class support for screen readers and complete keyboard navigation, making email accessible to everyone.

## ✨ Features

### 🚀 Core Features
- **📧 Email Provider Selector**: One-click setup for Gmail, Outlook, Yahoo, iCloud, and ProtonMail
- **🧵 Thread View**: Conversation grouping with visual hierarchy and indentation
- **📎 Attachment Viewer**: View and save attachments with file type recognition
- **🔍 Advanced Search**: Search messages with results display
- **📱 Context Menus**: Right-click actions for quick message management
- **⚡ Performance Optimized**: Smooth scrolling and efficient rendering
- **❌ Smart Error Handling**: User-friendly errors with context-aware troubleshooting

### ♿ Accessibility First
- **Screen Reader Support**: Full compatibility with NVDA, JAWS, and Windows Narrator
- **Keyboard Navigation**: Every function accessible via keyboard (25+ shortcuts)
- **AccessKit Integration**: Native Windows UIA for best screen reader experience
- **WCAG 2.1 Level AA**: Compliant with accessibility standards
- **Focus Indicators**: Clear visual feedback for keyboard navigation
- **Announcements**: Status updates announced to screen readers

### 🦀 Built with Rust
- **Fast & Reliable**: Native performance with memory safety
- **Secure**: Type-safe code with minimal dependencies
- **Cross-platform Ready**: Foundation for future Linux and macOS support

## 📖 Documentation

Comprehensive documentation available in the `docs/` directory:

- **[USER_GUIDE.md](docs/USER_GUIDE.md)** - Complete user manual with setup guides
- **[KEYBOARD_SHORTCUTS.md](docs/KEYBOARD_SHORTCUTS.md)** - Full keyboard shortcut reference
- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Solutions for common issues
- **[PROVIDER_SETUP.md](docs/PROVIDER_SETUP.md)** - Step-by-step provider configuration
- **[FEATURES_SUMMARY.md](docs/FEATURES_SUMMARY.md)** - Technical feature overview
- **[NEXT_STEPS.md](NEXT_STEPS.md)** - v1.0 roadmap and implementation plan
- **[ROADMAP.md](ROADMAP.md)** - Complete project roadmap
- **[ACCESSIBILITY.md](ACCESSIBILITY.md)** - Accessibility framework details
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Technical architecture

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/PratikP1/Wixen-Mail.git
cd Wixen-Mail

# Build and run
cargo build
cargo run --bin ui_integrated
```

### First-Time Setup

1. Launch Wixen Mail
2. Click **File → Connect to Server** or press `Ctrl+O`
3. Enter your email address (e.g., `user@gmail.com`)
4. Provider settings auto-fill based on your email
5. Enter your username and password (use app password for Gmail/Yahoo/iCloud)
6. Click **Connect**

See [PROVIDER_SETUP.md](docs/PROVIDER_SETUP.md) for detailed provider-specific instructions.

## ⌨️ Essential Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New Message | `Ctrl+N` |
| Reply | `Ctrl+R` |
| Forward | `Ctrl+L` |
| Search | `Ctrl+F` |
| Delete | `Delete` |
| Send | `Ctrl+Enter` |
| Switch Panes | `F6` |
| Refresh | `F5` |

See [KEYBOARD_SHORTCUTS.md](docs/KEYBOARD_SHORTCUTS.md) for complete reference.

## 🎯 Project Status

**Current Version:** Beta  
**Status:** 8 of 9 core features complete  
**Tests:** 80/80 passing  
**Documentation:** 5 comprehensive guides (62KB+)

### Completed Features ✅
1. ✅ UI Provider Selector with auto-detection
2. ✅ Thread View UI with visual hierarchy
3. ✅ Attachment Viewer with file type icons
4. ✅ Advanced Search UI
5. ✅ Context Menus with quick actions
6. ✅ Performance Optimization
7. ✅ Enhanced Error Handling
8. ✅ Comprehensive Documentation

### In Progress 🚧
9. 🚧 Final Polish (UI consistency, testing, animations)

See [ROADMAP.md](ROADMAP.md) for detailed project timeline.

## 💻 Building from Source

### Prerequisites

- **Rust 1.70 or later** - Install from [rustup.rs](https://rustup.rs/)
- **Windows 10 or later** (Linux/macOS support planned)
- **Internet connection** for initial dependency download

### Build Steps

```bash
# Clone the repository
git clone https://github.com/PratikP1/Wixen-Mail.git
cd Wixen-Mail

# Build the project
cargo build

# Run the integrated UI
cargo run --bin ui_integrated

# Or run the basic UI
cargo run --bin ui
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Development Build

```bash
# Build in debug mode (faster compilation, slower runtime)
cargo build

# Run with logging
RUST_LOG=debug cargo run --bin ui_integrated

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

### Release Build

```bash
# Build optimized release version (slower compilation, faster runtime)
cargo build --release

# Run release build
cargo run --release --bin ui_integrated
```

## 🤝 Contributing

We welcome contributions! This project is open source and accessible to all skill levels.

### How to Contribute

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
4. **Make your changes** with appropriate tests
5. **Ensure accessibility** features are maintained
6. **Run tests** (`cargo test`)
7. **Commit your changes** (`git commit -m 'Add amazing feature'`)
8. **Push to your fork** (`git push origin feature/amazing-feature`)
9. **Open a Pull Request**

### Contribution Guidelines

- **Code Quality**
  - Follow Rust style guidelines (`cargo fmt`)
  - Pass all linter checks (`cargo clippy`)
  - Include tests for new features
  - Document public APIs

- **Accessibility**
  - Maintain keyboard accessibility
  - Ensure screen reader compatibility
  - Test with NVDA, JAWS, or Narrator
  - Follow WCAG 2.1 Level AA standards

- **Documentation**
  - Update relevant documentation
  - Add inline code comments for complex logic
  - Include examples where helpful

### Areas Needing Help

See [ROADMAP.md](ROADMAP.md) for specific tasks. Some areas where you can contribute:

- 🐛 **Bug Fixes** - Check GitHub issues
- ✨ **New Features** - OAuth support, calendar sync, etc.
- 📖 **Documentation** - Improve guides and examples
- ♿ **Accessibility Testing** - Test with different screen readers
- 🌍 **Internationalization** - Add language support
- 🎨 **UI/UX** - Improve visual design
- 🧪 **Testing** - Add more test coverage

## ♿ Accessibility

Wixen Mail is designed with accessibility as a core principle, not an afterthought. We are committed to:

- ✅ **WCAG 2.1 Level AA** compliance
- ✅ **Full screen reader support** (NVDA, JAWS, Narrator)
- ✅ **Complete keyboard navigation** (25+ shortcuts)
- ✅ **High contrast mode** support
- ✅ **Customizable settings** (font sizes, themes)
- ✅ **Focus indicators** for keyboard navigation
- ✅ **Status announcements** for screen readers

For detailed accessibility information, see [ACCESSIBILITY.md](ACCESSIBILITY.md).

## 🛡️ Security

- **TLS/SSL encryption** for IMAP and SMTP connections
- **App password support** for enhanced security
- **HTML email sanitization** to prevent XSS attacks
- **Secure credential storage** (Windows DPAPI planned)
- **No plain text password storage**
- **Connection status indicators**

## 🗺️ Roadmap

### Current Release (Beta)
- ✅ Email provider auto-configuration
- ✅ Thread view with conversation grouping
- ✅ Attachment viewer with file type recognition
- ✅ Advanced search functionality
- ✅ Context menus and quick actions
- ✅ Performance optimizations
- ✅ Enhanced error handling
- ✅ Comprehensive documentation

### Next Release (v1.0)
- 🔜 OAuth 2.0 authentication
- 🔜 Multiple account support
- 🔜 Offline mode with sync
- 🔜 Rich text composition
- 🔜 Message rules and filters
- 🔜 Email signatures

### Future Plans
- 📅 Calendar integration (CalDAV)
- 👥 Contacts integration (CardDAV)
- 🔐 Email encryption (PGP/GPG)
- 🌐 Internationalization (i18n)
- 🐧 Linux support
- 🍎 macOS support

See [ROADMAP.md](ROADMAP.md) for detailed timeline.

## 📊 Technical Stack

- **Language:** Rust (2021 edition)
- **UI Framework:** egui 0.29 (immediate mode GUI)
- **Accessibility:** AccessKit (Windows UIA support)
- **Email Protocols:** Custom IMAP, lettre SMTP
- **Async Runtime:** Tokio 1.x
- **Database:** rusqlite 0.32 (message caching)
- **HTML Sanitization:** ammonia 4.0
- **Email Parsing:** mail-parser 0.9

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Inspired by** Mozilla Thunderbird's accessible email experience
- **Built with** the amazing Rust ecosystem and community
- **Accessibility guidance** from screen reader users and advocates
- **Email protocol specs** from IETF RFCs
- **UI framework** from egui contributors
- **Accessibility layer** from AccessKit project

## 📞 Contact & Support

- **Issues:** [Report bugs or request features](https://github.com/PratikP1/Wixen-Mail/issues)
- **Discussions:** [Ask questions or share ideas](https://github.com/PratikP1/Wixen-Mail/discussions)
- **Documentation:** Comprehensive guides in [docs/](docs/) directory
- **Email:** See GitHub profile for contact information

## 🎯 Project Goals

**Mission:** Create a fully accessible, secure, and user-friendly email client that works seamlessly with screen readers and keyboard-only navigation.

**Vision:** Email should be accessible to everyone, regardless of ability. Wixen Mail aims to prove that accessibility and excellent user experience can go hand-in-hand.

**Values:**
- **Accessibility First:** Built with accessibility from the ground up
- **User Privacy:** Your email data stays private
- **Open Source:** Transparent, community-driven development
- **Performance:** Fast, efficient, and resource-friendly
- **Standards Compliance:** Following email and accessibility standards

---

**Made with ❤️ and ♿ by the Wixen Mail community**

Our mission is to create a mail client that:
1. **Is accessible to everyone**, including users who rely on assistive technologies
2. **Performs efficiently** with minimal resource usage
3. **Respects user privacy** with no telemetry or tracking
4. **Provides excellent user experience** for both power users and beginners
5. **Remains open source** and community-driven

---

**Note**: This is an early-stage project. Stars, feedback, and contributions are highly appreciated!
 
