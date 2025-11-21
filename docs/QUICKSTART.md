# Megamaid - Quick Start Guide

## Table of Contents

- [User Quick Start](#user-quick-start)
- [Developer Quick Start](#developer-quick-start)
- [Running for Debug](#running-for-debug)
- [Common Workflows](#common-workflows)
- [Troubleshooting](#troubleshooting)

---

## User Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/megamaid.git
cd megamaid

# Build the release version
cargo build --release

# The binary will be at target/release/megamaid
```

### First Scan

```bash
# Scan your current directory
./target/release/megamaid scan .

# On Windows
.\target\release\megamaid.exe scan .
```

This creates a `cleanup-plan.toml` file with all detected cleanup candidates.

### Review the Plan

```bash
# View statistics
./target/release/megamaid stats cleanup-plan.toml

# Or open in your favorite editor
vim cleanup-plan.toml
code cleanup-plan.toml
notepad cleanup-plan.toml
```

### Customize the Scan

```bash
# Scan specific directory
megamaid scan /path/to/directory

# Custom output file
megamaid scan . --output my-cleanup.toml

# Flag files larger than 200MB
megamaid scan . --large-file-threshold 200

# Limit scan depth to 3 levels
megamaid scan . --max-depth 3

# Include hidden files
megamaid scan . --no-skip-hidden
```

---

## Developer Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git

### Setup

```bash
# Clone and enter project
git clone https://github.com/yourusername/megamaid.git
cd megamaid

# Verify setup
cargo test
cargo clippy
cargo fmt -- --check
```

### Build

```bash
# Debug build (fast compilation, includes debug symbols)
cargo build

# Release build (optimized, slower compilation)
cargo build --release
```

### Run Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# Property-based tests
cargo test --test scanner_properties

# Performance tests (long-running)
cargo test --test performance_tests -- --ignored --nocapture
```

---

## Running for Debug

### Basic Debug Run

```bash
# Run with cargo (automatically uses debug build)
cargo run -- scan /path/to/directory

# Scan current directory
cargo run -- scan .

# With custom output
cargo run -- scan . --output debug-plan.toml
```

### Debug with Options

```bash
# Show help
cargo run -- --help
cargo run -- scan --help

# Scan with all options
cargo run -- scan . \
  --output test.toml \
  --large-file-threshold 50 \
  --max-depth 3 \
  --skip-hidden

# View plan statistics
cargo run -- stats cleanup-plan.toml
```

### Debug with Enhanced Output

```bash
# With Rust backtrace (better error messages)
RUST_BACKTRACE=1 cargo run -- scan /path/to/directory

# Full backtrace (very detailed)
RUST_BACKTRACE=full cargo run -- scan /path/to/directory

# On Windows (PowerShell)
$env:RUST_BACKTRACE=1; cargo run -- scan C:\path\to\directory

# On Windows (CMD)
set RUST_BACKTRACE=1
cargo run -- scan C:\path\to\directory
```

### Using Built Binary Directly

```bash
# Build once
cargo build

# Run the debug binary
./target/debug/megamaid scan /path/to/directory

# On Windows
.\target\debug\megamaid.exe scan C:\path\to\directory
```

### Debug with VS Code

Create `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Megamaid Scan",
            "cargo": {
                "args": ["build", "--bin=megamaid"]
            },
            "args": ["scan", ".", "--output", "test-plan.toml"],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Megamaid Stats",
            "cargo": {
                "args": ["build", "--bin=megamaid"]
            },
            "args": ["stats", "cleanup-plan.toml"],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

Then press F5 in VS Code to start debugging.

### Debug with GDB/LLDB

```bash
# Build with debug symbols
cargo build

# Debug with rust-gdb (Linux)
rust-gdb --args target/debug/megamaid scan /path/to/directory

# Debug with rust-lldb (macOS/Linux)
rust-lldb -- target/debug/megamaid scan /path/to/directory

# Common GDB/LLDB commands:
# - break main              : Set breakpoint at main
# - run                     : Start execution
# - next (n)                : Step over
# - step (s)                : Step into
# - continue (c)            : Continue execution
# - print variable          : Print variable value
# - backtrace (bt)          : Show call stack
```

---

## Common Workflows

### Quick Development Cycle

```bash
# 1. Make code changes
# 2. Run tests
cargo test

# 3. Check with clippy
cargo clippy

# 4. Test manually
cargo run -- scan . --output test.toml

# 5. View results
cargo run -- stats test.toml
```

### Watch Mode (Auto-rebuild)

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild and run on file changes
cargo watch -x "run -- scan ."

# Auto-run tests on changes
cargo watch -x test

# Run clippy on changes
cargo watch -x clippy
```

### Testing with Sample Data

```bash
# Create test directory structure
mkdir -p /tmp/test-megamaid/src
mkdir -p /tmp/test-megamaid/target
echo "fn main() {}" > /tmp/test-megamaid/src/main.rs
echo "build artifact" > /tmp/test-megamaid/target/debug

# Scan it
cargo run -- scan /tmp/test-megamaid --output test.toml

# View results
cargo run -- stats test.toml
cat test.toml

# Clean up
rm -rf /tmp/test-megamaid test.toml
```

### Testing with Your Own Project

```bash
# Scan the megamaid project itself
cargo run -- scan . --output self-scan.toml

# See what build artifacts it finds
cargo run -- stats self-scan.toml

# Should detect target/, node_modules/ (if present), etc.
```

### Performance Testing

```bash
# Quick performance test (100 files)
cargo test test_scan_small_dataset -- --nocapture

# Medium test (10K files)
cargo test --test performance_tests test_scan_10k_files_performance -- --ignored --nocapture

# Large test (100K files, takes several minutes)
cargo test --test performance_tests test_scan_100k_files_performance -- --ignored --nocapture
```

### Code Quality Checks

```bash
# Run all quality checks
cargo test && cargo clippy -- -D warnings && cargo fmt -- --check

# Format code
cargo fmt

# Fix clippy warnings automatically (where possible)
cargo clippy --fix

# Check for security vulnerabilities
cargo install cargo-audit
cargo audit
```

### Building Documentation

```bash
# Build and open docs
cargo doc --open

# Build docs for all dependencies
cargo doc --document-private-items --open
```

---

## Troubleshooting

### Build Issues

**Problem**: Compilation errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
cargo build
```

**Problem**: Slow compilation

```bash
# Use debug build for development
cargo build  # Not cargo build --release

# Enable parallel compilation (add to ~/.cargo/config.toml)
[build]
jobs = 4  # Or number of CPU cores
```

### Runtime Issues

**Problem**: "Permission denied" errors

```bash
# Run with elevated permissions (if needed)
sudo cargo run -- scan /protected/directory

# Or change directory permissions
chmod -R +r /path/to/scan
```

**Problem**: Crashes or panics

```bash
# Run with backtrace
RUST_BACKTRACE=full cargo run -- scan /path

# Run with debug build for better error messages
cargo build
./target/debug/megamaid scan /path
```

**Problem**: Slow scanning

```bash
# Limit scan depth
cargo run -- scan /path --max-depth 5

# Skip hidden files (default: true)
cargo run -- scan /path --skip-hidden

# Test with smaller directory first
cargo run -- scan /path/subdirectory
```

### Test Issues

**Problem**: Tests failing

```bash
# Run specific test
cargo test test_name -- --nocapture

# Run tests in single thread (helpful for debugging)
cargo test -- --test-threads=1

# Show test output
cargo test -- --nocapture

# Run only failing tests
cargo test --lib  # Just unit tests
```

**Problem**: Property tests taking too long

```bash
# These tests run many iterations, use --nocapture to see progress
cargo test --test scanner_properties -- --nocapture

# Or skip them during development
cargo test --lib  # Only unit tests
```

### IDE Issues

**Problem**: VS Code rust-analyzer not working

```bash
# Restart rust-analyzer
# In VS Code: Ctrl+Shift+P -> "Rust Analyzer: Restart Server"

# Or rebuild from clean slate
cargo clean
cargo build
```

**Problem**: IntelliJ IDEA not recognizing code

```bash
# Refresh Cargo project
# In IDEA: Right-click Cargo.toml -> Refresh Cargo Project

# Or reimport
cargo clean
# Then reimport in IDE
```

---

## Environment Variables

```bash
# Enable debug logging (if using env_logger in future)
export RUST_LOG=megamaid=debug

# Backtrace levels
export RUST_BACKTRACE=1     # Simple backtrace
export RUST_BACKTRACE=full  # Full backtrace

# Windows (PowerShell)
$env:RUST_BACKTRACE=1
$env:RUST_LOG="megamaid=debug"
```

---

## Quick Reference

### Most Common Commands

```bash
# Development
cargo run -- scan .                    # Quick test scan
cargo test                             # Run all tests
cargo clippy                           # Lint code
cargo fmt                              # Format code

# Building
cargo build                            # Debug build
cargo build --release                  # Release build

# Testing
cargo test --lib                       # Unit tests only
cargo test --test '*'                  # Integration tests
cargo test -- --nocapture              # Show test output

# Running
./target/debug/megamaid scan .         # Run debug binary
./target/release/megamaid scan .       # Run release binary
```

### File Locations

```
megamaid/
├── src/                    # Source code
├── tests/                  # Integration tests
├── target/
│   ├── debug/             # Debug builds
│   │   └── megamaid       # Debug binary
│   └── release/           # Release builds
│       └── megamaid       # Release binary
├── cleanup-plan.toml      # Default output (gitignored)
└── Cargo.toml             # Project configuration
```

---

## Next Steps

- Read [README.md](../README.md) for full feature documentation
- See [ARCHITECTURE.md](ARCHITECTURE.md) for design details
- Check [CLAUDE.md](../CLAUDE.md) for development guidelines
- Review [MILESTONE_1_COMPLETE.md](MILESTONE_1_COMPLETE.md) for current status

---

## Getting Help

- **Documentation**: See `docs/` directory
- **Issues**: Open an issue on GitHub
- **Tests**: Run `cargo test -- --nocapture` to see what's expected
- **Examples**: Check `tests/` directory for usage examples

Happy coding! 🦀
