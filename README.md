# Megamaid - Storage Cleanup Tool

High-performance storage analysis and cleanup for developers.

## Features

- ⚡ **Fast directory scanning** - 100K+ files in under 60 seconds on SSD
- 🎯 **Smart detection** - Automatically identifies build artifacts and large files
- 📝 **Human-editable plans** - Review and modify TOML cleanup plans before execution
- 🔒 **Safe operation** - Drift detection prevents accidental deletions
- 💻 **Cross-platform** - Windows 11 and Linux support

## Installation

### From Source

```bash
git clone https://github.com/yourusername/megamaid.git
cd megamaid
cargo build --release
```

The binary will be available at `target/release/megamaid`.

### Using Cargo

```bash
cargo install megamaid
```

## Quick Start

### Scan a Directory

```bash
# Scan current directory
megamaid scan .

# Scan specific path with custom options
megamaid scan /path/to/directory --output my-plan.toml --large-file-threshold 200
```

### Review the Plan

The scan creates a `cleanup-plan.toml` file with detected cleanup candidates:

```toml
version = "0.1.0"
created_at = "2025-11-19T12:00:00Z"
base_path = "/path/to/directory"

[[entries]]
path = "target"
size = 524288000
modified = "2025-11-19T10:30:00Z"
action = "delete"
rule_name = "build_artifact"
reason = "Rust build artifact directory"

[[entries]]
path = "video.mp4"
size = 2147483648
modified = "2025-11-18T15:20:00Z"
action = "review"
rule_name = "large_file"
reason = "File exceeds size threshold of 100 MB"
```

### Edit the Plan

Review and modify actions as needed:

```bash
# Open in your editor
vim cleanup-plan.toml

# Or use any text editor
notepad cleanup-plan.toml
```

Change any `action = "delete"` to `action = "keep"` for files you want to preserve.

### View Plan Statistics

```bash
megamaid stats cleanup-plan.toml
```

Output:

```
Cleanup Plan Statistics
========================

Plan Details:
  Version: 0.1.0
  Created: 2025-11-19T12:00:00Z
  Base Path: /path/to/directory

Entry Counts by Action:
  Delete:  5 entries
  Review:  3 entries
  Keep:    1 entries

Total Size: 2.45 GB
```

## Usage

### Scan Command

```bash
megamaid scan [PATH] [OPTIONS]
```

#### Options

- `--output, -o <FILE>` - Output plan file (default: `cleanup-plan.toml`)
- `--large-file-threshold <MB>` - Minimum file size to flag in MB (default: 100)
- `--max-depth <N>` - Maximum directory depth to scan
- `--skip-hidden` - Skip hidden files and directories (default: true)

#### Examples

```bash
# Scan with default settings
megamaid scan ~/projects

# Limit scan depth
megamaid scan ~/projects --max-depth 3

# Flag files larger than 200MB
megamaid scan ~/projects --large-file-threshold 200

# Include hidden files
megamaid scan ~/projects --no-skip-hidden

# Custom output path
megamaid scan ~/projects --output ~/cleanup/my-plan.toml
```

### Stats Command

```bash
megamaid stats <PLAN_FILE>
```

Displays statistics about a cleanup plan without making any changes.

## Detected Patterns

### Build Artifacts

Megamaid automatically detects common build artifact directories:

- **Rust**: `target/`
- **Node.js**: `node_modules/`
- **Python**: `__pycache__/`, `.pytest_cache/`
- **Next.js**: `.next/`
- **Generic**: `build/`, `dist/`, `bin/`, `obj/`

These default to `action = "delete"` since they can be regenerated.

### Large Files

Files exceeding the size threshold (default 100MB) are flagged for review. These default to `action = "review"` for user discretion.

## Plan File Format

Cleanup plans use TOML format for easy editing:

```toml
version = "0.1.0"           # Plan format version
created_at = "..."          # ISO 8601 timestamp
base_path = "/path"         # Scanned directory

[[entries]]                 # Repeated for each entry
path = "relative/path"      # Path relative to base_path
size = 1048576              # Size in bytes
modified = "..."            # Last modified time (RFC3339)
action = "delete"           # delete | keep | review
rule_name = "..."           # Which rule flagged this
reason = "..."              # Human-readable reason
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run property-based tests
cargo test --test scanner_properties

# Run performance benchmarks (long-running, requires --ignored)
cargo test --test performance_tests -- --ignored --nocapture
```

### Current Test Coverage

- **86 tests passing** (66 unit + 5 integration + 5 property-based + 8 feature matrix + 1 performance + 1 doc)
- 6 additional long-running performance tests available with `--ignored`

## Performance

### Targets (Milestone 1)

- ✅ Scan 100K files in <60 seconds (SSD)
- ✅ Scan 1M files in <5 minutes (SSD)
- ✅ Memory usage <100MB for 100K file scan
- ✅ Plan generation <30s for 1M entries

### Benchmarks

Run performance tests:

```bash
# Quick performance test
cargo test test_scan_small_dataset -- --nocapture

# Full performance suite (creates 100K files, takes several minutes)
cargo test --test performance_tests -- --ignored --nocapture
```

## Architecture

### Workflow

```
User Input → Scanner → FileEntry[]
                ↓
          Detector → DetectionResult[]
                ↓
          Planner → CleanupPlan
                ↓
          TOML File → User Review → (Future: Execution)
```

### Components

- **Scanner** (`src/scanner/`): Directory traversal and metadata collection
- **Detector** (`src/detector/`): Rule-based cleanup candidate identification
- **Planner** (`src/planner/`): TOML plan generation and serialization
- **CLI** (`src/cli/`): Command-line interface with progress reporting
- **Models** (`src/models/`): Core data structures

For detailed architecture documentation, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Roadmap

### Milestone 1: Directory Scan & Plan Generation ✅

- [x] File system scanning
- [x] Build artifact detection
- [x] Large file detection
- [x] TOML plan generation
- [x] CLI with scan and stats commands
- [x] Comprehensive test suite (86 tests)

### Milestone 2: Plan Verification & Deletion (Next)

- [ ] Plan verification with drift detection
- [ ] Safe deletion execution
- [ ] Recycle bin support (Windows)
- [ ] Deletion confirmation prompts

### Future Milestones

- **Milestone 3**: Parallel operations & multi-threading
- **Milestone 4**: Tauri GUI integration
- **Milestone 5**: NTFS optimizations & advanced features

## Contributing

Contributions are welcome! Please see the development guidelines in [CLAUDE.md](CLAUDE.md).

### Running Tests

Before submitting a PR:

```bash
# Run all tests
cargo test

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Run all checks
cargo test && cargo fmt -- --check && cargo clippy -- -D warnings
```

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust 🦀
- Inspired by tools like DaisyDisk and WizTree
- Created as a learning project for Rust systems programming

## FAQ

### Q: Will this delete my files?

No. Milestone 1 only generates TOML plans for review. File deletion will be added in Milestone 2 with multiple safety checks.

### Q: Is it safe to scan my entire drive?

Yes. The scanner only reads file metadata (path, size, modification time). It doesn't modify or delete anything.

### Q: Can I add custom detection rules?

Currently, rules are built-in (build artifacts and large files). Custom rules via config files are planned for a future milestone.

### Q: What about symlinks?

By default, symlinks are not followed to avoid potential cycles. This may be configurable in the future.

### Q: How does it handle permission errors?

Permission errors are logged but don't stop the scan. Files you can't access are simply skipped.

## Support

- **Issues**: Report bugs at [GitHub Issues](https://github.com/yourusername/megamaid/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/yourusername/megamaid/discussions)

---

Made with ❤️ and Rust
