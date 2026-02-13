# Getting Started with Wixen Mail

## Introduction

Welcome to Wixen Mail! This guide will help you get started with building and contributing to the project.

## Prerequisites

Before you begin, ensure you have the following installed:

### Required
- **Rust**: Version 1.70 or later
  - Install from [rustup.rs](https://rustup.rs/)
  - Verify installation: `rustc --version`
  
- **Git**: For version control
  - Download from [git-scm.com](https://git-scm.com/)
  - Verify installation: `git --version`

### Recommended
- **Visual Studio Code** with rust-analyzer extension
  - Or any editor with Rust support
  
- **Windows 10 or later** (required for WXDragon UI)

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/PratikP1/Wixen-Mail.git
cd Wixen-Mail
```

### 2. Build the Project

```bash
# Build in debug mode
cargo build

# Or build in release mode (optimized)
cargo build --release
```

### 3. Run the Application

```bash
# Run in debug mode
cargo run

# Or run the release build
./target/release/wixen-mail
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name
```

## Development Workflow

### Code Style

We use standard Rust formatting and linting tools:

```bash
# Format your code
cargo fmt

# Check formatting without making changes
cargo fmt --check

# Run the linter
cargo clippy

# Run clippy with strict warnings
cargo clippy -- -D warnings
```

### Making Changes

1. Create a new branch for your feature or fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and test them:
   ```bash
   cargo build
   cargo test
   cargo clippy
   ```

3. Commit your changes:
   ```bash
   git add .
   git commit -m "Add feature: description"
   ```

4. Push to your fork and create a pull request

## Project Structure

```
Wixen-Mail/
├── .github/          # GitHub workflows and configurations
│   └── workflows/    # CI/CD workflows
├── docs/             # Additional documentation
├── src/              # Source code
│   └── main.rs       # Application entry point
├── tests/            # Integration tests (to be added)
├── Cargo.toml        # Project manifest
├── Cargo.lock        # Dependency lock file
├── README.md         # Project overview
├── ROADMAP.md        # Project roadmap
├── ACCESSIBILITY.md  # Accessibility guide
├── ARCHITECTURE.md   # Technical architecture
└── CONTRIBUTING.md   # Contribution guidelines
```

## Next Steps

- Read the [ROADMAP.md](../ROADMAP.md) to understand the project direction
- Check [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines
- Review [ACCESSIBILITY.md](../ACCESSIBILITY.md) for accessibility requirements
- Explore [ARCHITECTURE.md](../ARCHITECTURE.md) for technical details

## Troubleshooting

### Build Fails

If the build fails, try:
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build
```

### Rust Version Issues

Ensure you're using Rust 1.70 or later:
```bash
# Check current version
rustc --version

# Update Rust
rustup update
```

### Missing Dependencies

If you get dependency errors:
```bash
# Update Cargo.lock
cargo update

# Or delete Cargo.lock and rebuild
rm Cargo.lock
cargo build
```

## Getting Help

- **Documentation**: Check the docs/ folder
- **Issues**: Search existing issues on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Contributing**: See CONTRIBUTING.md

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [rustup Documentation](https://rust-lang.github.io/rustup/)

## Building for Release

When preparing a release build:

```bash
# Build with optimizations
cargo build --release

# The binary will be in:
# target/release/wixen-mail.exe (on Windows)
```

## Running with Debug Logging

To see debug output:

```bash
# Windows (PowerShell)
$env:RUST_LOG="debug"; cargo run

# Windows (CMD)
set RUST_LOG=debug && cargo run

# Linux/macOS
RUST_LOG=debug cargo run
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

## Contributing

We welcome contributions! See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

Areas where you can help:
- Implementing mail protocol support (IMAP, SMTP)
- Building accessible UI components
- Writing tests
- Improving documentation
- Testing with screen readers

---

Happy coding! If you have questions, don't hesitate to ask in GitHub Discussions or open an issue.
