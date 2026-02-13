# Contributing to Wixen Mail

Thank you for your interest in contributing to Wixen Mail! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and professional in all interactions.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue with:
- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs. actual behavior
- Your environment (OS version, Rust version, etc.)
- Screenshots or logs if applicable

**For accessibility issues**, please also include:
- Assistive technology being used (name and version)
- Specific accessibility barrier encountered

### Suggesting Features

Feature requests are welcome! Please open an issue with:
- A clear description of the feature
- Use cases and benefits
- How it fits with the project goals
- Consider accessibility implications

### Pull Requests

1. **Fork the repository** and create a new branch for your feature or fix
2. **Make your changes** following our coding guidelines
3. **Add tests** for new functionality
4. **Ensure all tests pass** with `cargo test`
5. **Run the linter** with `cargo clippy`
6. **Format your code** with `cargo fmt`
7. **Update documentation** if needed
8. **Submit a pull request** with a clear description

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- A code editor (VS Code with rust-analyzer recommended)

### Setting Up Your Development Environment

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/Wixen-Mail.git
cd Wixen-Mail

# Add upstream remote
git remote add upstream https://github.com/PratikP1/Wixen-Mail.git

# Create a new branch
git checkout -b feature/your-feature-name
```

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Check for common issues
cargo clippy

# Format code
cargo fmt
```

## Coding Guidelines

### Rust Style

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` to automatically format code
- Run `cargo clippy` and address all warnings
- Write idiomatic Rust code

### Documentation

- Add doc comments (`///`) for public APIs
- Include examples in doc comments when helpful
- Keep comments up-to-date with code changes
- Document accessibility considerations

### Testing

- Write unit tests for new functionality
- Aim for high test coverage
- Include integration tests for complex features
- Test accessibility features with screen readers when possible

### Accessibility Requirements

When contributing code that affects the UI or user interaction:

1. **Keyboard Navigation**:
   - All functionality must be accessible via keyboard
   - Implement logical tab order
   - Provide keyboard shortcuts for common actions

2. **Screen Reader Support**:
   - Use appropriate ARIA labels and roles
   - Provide descriptive text for UI elements
   - Test with at least one screen reader (NVDA recommended)

3. **Focus Management**:
   - Visible focus indicators
   - Proper focus trapping in dialogs
   - Focus returns to appropriate location after operations

4. **Testing**:
   - Test all new features with keyboard only
   - Verify screen reader announcements
   - Check high contrast mode compatibility

### Commit Messages

Use clear, descriptive commit messages:

```
Add feature: Brief description

Detailed explanation of what was changed and why.
Include any relevant context or references.

Fixes #123
```

Format:
- First line: Short summary (50 chars or less)
- Blank line
- Detailed description (if needed)
- Reference related issues

## Project Structure

```
Wixen-Mail/
├── src/               # Source code
│   ├── main.rs       # Application entry point
│   └── ...           # Other modules (to be added)
├── tests/            # Integration tests
├── docs/             # Additional documentation
├── Cargo.toml        # Project manifest
├── README.md         # Project overview
├── ROADMAP.md        # Project roadmap
├── ACCESSIBILITY.md  # Accessibility guide
├── ARCHITECTURE.md   # Technical architecture
└── CONTRIBUTING.md   # This file
```

## Areas to Contribute

### High Priority

1. **Core Email Protocols**: IMAP, SMTP implementation
2. **UI Components**: Accessible UI widgets using WXDragon
3. **Accessibility Layer**: Screen reader integration
4. **Testing**: Test suite development

### Medium Priority

1. **Message Parsing**: Email parsing and rendering
2. **Contact Management**: Address book functionality
3. **Search**: Full-text search implementation
4. **Configuration**: Settings management

### Future

1. **Security**: Encryption and authentication features
2. **Extensions**: Plugin system
3. **Cross-platform**: Linux and macOS support

See [ROADMAP.md](ROADMAP.md) for detailed task list.

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Issues**: Check existing issues or create a new one
- **Real-time chat**: (Coming soon)

## Recognition

Contributors will be:
- Listed in the project contributors
- Credited in release notes for significant contributions
- Invited to participate in project decisions

## License

By contributing to Wixen Mail, you agree that your contributions will be licensed under the MIT License.

## Review Process

1. A maintainer will review your pull request
2. Feedback will be provided if changes are needed
3. Once approved, your PR will be merged
4. You'll be added as a contributor

## Accessibility Review

For PRs affecting accessibility:
1. Manual keyboard navigation testing required
2. Screen reader testing recommended
3. Accessibility checklist must be completed
4. May request testing from users with disabilities

## Thank You!

Your contributions help make Wixen Mail accessible and useful for everyone. We appreciate your time and effort!
