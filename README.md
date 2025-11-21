# Megamaid - Storage Cleanup Tool

High-performance storage analysis and cleanup for developers.

## Features

- ⚡ **Fast directory scanning** - 100K+ files in under 60 seconds on SSD
- 🎯 **Smart detection** - Automatically identifies build artifacts and large files
- 📝 **Human-editable plans** - Review and modify YAML cleanup plans before execution
- 🔒 **Safe operation** - Drift detection prevents accidental deletions
- ♻️ **Multiple execution modes** - Dry-run, interactive, batch, backup, recycle bin
- 📋 **Complete audit trails** - Transaction logs for every execution
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

See [QUICKSTART.md](docs/QUICKSTART.md) for a comprehensive walkthrough.

### 1. Scan a Directory

```bash
megamaid scan /path/to/cleanup
```

This creates `cleanup-plan.yaml` with detected cleanup candidates.

### 2. Verify the Plan

```bash
megamaid verify cleanup-plan.yaml
```

Checks if files have changed since the plan was created (drift detection).

### 3. Review and Edit

```bash
# Open in your editor
vim cleanup-plan.yaml

# Change actions as needed:
# - action: delete  (will be deleted)
# - action: review  (for manual review)
# - action: keep    (will be preserved)
```

### 4. Execute the Plan

```bash
# Dry-run first (recommended)
megamaid execute cleanup-plan.yaml --dry-run

# Execute with backup
megamaid execute cleanup-plan.yaml --backup-dir ./backups

# Or execute normally
megamaid execute cleanup-plan.yaml
```

## Commands

### scan - Scan a Directory

```bash
megamaid scan [PATH] [OPTIONS]
```

**Options:**
- `--output, -o <FILE>` - Output plan file (default: `cleanup-plan.yaml`)
- `--large-file-threshold <MB>` - Minimum file size to flag in MB (default: 100)
- `--max-depth, -d <N>` - Maximum directory depth to scan
- `--skip-hidden` - Skip hidden files and directories (default: true)

**Examples:**
```bash
# Scan with default settings
megamaid scan ~/projects

# Limit scan depth
megamaid scan ~/projects --max-depth 3

# Flag files larger than 200MB
megamaid scan ~/projects --large-file-threshold 200

# Custom output path
megamaid scan ~/projects --output ~/cleanup/my-plan.yaml
```

### verify - Verify a Plan

```bash
megamaid verify <PLAN_FILE> [OPTIONS]
```

**Options:**
- `--output, -o <FILE>` - Save drift report to file
- `--fail-fast` - Stop on first drift detection
- `--skip-mtime` - Skip modification time checks (size-only verification)

**Examples:**
```bash
# Verify plan is safe to execute
megamaid verify cleanup-plan.yaml

# Save drift report
megamaid verify cleanup-plan.yaml --output drift-report.txt

# Quick check with fail-fast
megamaid verify cleanup-plan.yaml --fail-fast
```

### execute - Execute a Plan

```bash
megamaid execute <PLAN_FILE> [OPTIONS]
```

**Options:**
- `--dry-run` - Simulate without actually deleting
- `--interactive, -i` - Prompt for confirmation on each deletion
- `--backup-dir <DIR>` - Move files to backup instead of deleting
- `--recycle-bin` - Use system recycle bin (allows recovery)
- `--fail-fast` - Stop on first error
- `--skip-verify` - Skip verification before execution (not recommended)
- `--log-file <FILE>` - Transaction log path (default: `execution-log.yaml`)

**Examples:**
```bash
# Dry-run to preview (safest, always do this first)
megamaid execute cleanup-plan.yaml --dry-run

# Interactive mode with confirmation prompts
megamaid execute cleanup-plan.yaml --interactive

# Backup mode (safest real execution)
megamaid execute cleanup-plan.yaml --backup-dir ./backups

# Recycle bin mode (allows recovery on Windows)
megamaid execute cleanup-plan.yaml --recycle-bin

# Batch execution (default)
megamaid execute cleanup-plan.yaml

# Custom transaction log
megamaid execute cleanup-plan.yaml --log-file my-execution.yaml
```

### stats - View Plan Statistics

```bash
megamaid stats <PLAN_FILE>
```

Displays statistics about a cleanup plan without making any changes.

**Example:**
```bash
megamaid stats cleanup-plan.yaml
```

Output:
```
📊 Cleanup Plan Statistics

Base Path: /path/to/directory
Version:   0.1.0
Created:   2025-11-21 10:30:00

Entries:   8
  • Delete: 3
  • Review: 4
  • Keep:   1

Total Size: 2048 MB
```

## Detected Patterns

### Build Artifacts

Megamaid automatically detects common build artifact directories:

- **Rust**: `target/`
- **Node.js**: `node_modules/`
- **Python**: `__pycache__/`, `.pytest_cache/`
- **Next.js**: `.next/`
- **Generic**: `build/`, `dist/`, `bin/`, `obj/`

These default to `action: delete` since they can be regenerated.

### Large Files

Files exceeding the size threshold (default 100MB) are flagged for review. These default to `action: review` for user discretion.

## Plan File Format

Cleanup plans use YAML format for easy editing:

```yaml
version: "0.1.0"
created_at: "2025-11-21T10:30:00Z"
base_path: /path/to/directory

entries:
  - path: target
    size: 524288000
    modified: "2025-11-21T09:15:00Z"
    action: delete
    rule_name: build_artifact
    reason: Common build artifact directory

  - path: video.mp4
    size: 2147483648
    modified: "2025-11-20T15:20:00Z"
    action: review
    rule_name: large_file
    reason: File exceeds size threshold of 100 MB
```

## Safety Features

### Drift Detection

Before execution, Megamaid verifies:
- ✅ Files still exist at expected paths
- ✅ Sizes match recorded values
- ✅ Modification times haven't changed
- ⚠️ Warns if any changes detected

### Execution Modes

- **Dry-Run**: Preview what would be deleted without actually deleting
- **Interactive**: Confirm each deletion manually
- **Backup**: Move files to backup directory (preserves structure)
- **Recycle Bin**: Use system trash (Windows: Recycle Bin, Linux: Trash)

### Transaction Logs

Every execution creates a transaction log (`execution-log.yaml`) with:
- Unique execution ID
- Timestamp and duration
- Every operation performed
- Success/failure status
- Errors encountered
- Space freed

Example:
```yaml
execution_id: e52af49c-8b50-450f-92d7-3550a7f62e28
started_at: 2025-11-21T15:28:06Z
completed_at: 2025-11-21T15:28:06Z
status: completed
operations:
  - path: target
    action: Delete
    status: Success
    size_freed: 524288000
summary:
  successful: 3
  failed: 0
  space_freed: 1073741824
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

- **126 tests passing**
  - 105 unit tests
  - 8 feature matrix tests
  - 5 integration tests
  - 5 property-based tests
  - 2 documentation tests
  - 1 quick performance test
- 6 additional long-running performance tests available with `--ignored`

## Performance

### Targets (Milestone 1 & 2)

- ✅ Scan 100K files in <60 seconds (SSD)
- ✅ Verify 100K entries in <10 seconds
- ✅ Execute 10K deletions in <60 seconds
- ✅ Memory usage <150MB for 100K entries

### Benchmarks

Run performance tests:

```bash
# Quick performance test
cargo test test_scan_small_dataset -- --nocapture

# Full performance suite (creates 100K files, takes several minutes)
cargo test --test performance_tests -- --ignored --nocapture
```

## Architecture

### Complete Workflow

```
User Input → Scanner → FileEntry[]
                ↓
          Detector → DetectionResult[]
                ↓
          Planner → CleanupPlan
                ↓
          YAML File → User Review
                ↓
          Verifier → Drift Check
                ↓
          Executor → Safe Deletion
                ↓
          Transaction Log → Audit Trail
```

### Components

- **Scanner** (`src/scanner/`): Directory traversal and metadata collection
- **Detector** (`src/detector/`): Rule-based cleanup candidate identification
- **Planner** (`src/planner/`): YAML plan generation and serialization
- **Verifier** (`src/verifier/`): Drift detection and plan verification
- **Executor** (`src/executor/`): Safe deletion with multiple modes and transaction logging
- **CLI** (`src/cli/`): Command-line interface with progress reporting
- **Models** (`src/models/`): Core data structures

For detailed architecture documentation, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Roadmap

### Milestone 1: Directory Scan & Plan Generation ✅

- [x] File system scanning
- [x] Build artifact detection
- [x] Large file detection
- [x] YAML plan generation
- [x] CLI with scan and stats commands
- [x] Comprehensive test suite (105 unit tests)
- [x] YAML format migration (60% smaller than TOML)

### Milestone 2: Plan Verification & Deletion ✅

- [x] Plan verification with drift detection
- [x] Safe deletion execution (dry-run, interactive, batch)
- [x] Backup mode (move instead of delete)
- [x] Recycle bin support (Windows & Linux)
- [x] Transaction logging and audit trails
- [x] Complete test coverage (126 tests)

### Milestone 3: Parallel Operations (Future)

- [ ] Multi-threaded scanning with rayon
- [ ] Parallel deletion operations
- [ ] Advanced progress reporting with ETA
- [ ] Configuration file support

### Milestone 4: Tauri GUI (Future)

- [ ] Visual interface for plan review
- [ ] Interactive file browser
- [ ] Real-time progress visualization
- [ ] Disk usage charts

### Milestone 5: Advanced Features (Future)

- [ ] NTFS MFT scanning for Windows optimization
- [ ] Custom detection rules from config
- [ ] Archive mode (ZIP/TAR instead of delete)
- [ ] Scheduled cleanup tasks

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
- Transaction logging inspired by database systems
- Created as a learning project for Rust systems programming

## FAQ

### Q: Will this delete my files?

Only if you explicitly run `megamaid execute`. The tool has multiple safety layers:
1. Dry-run mode to preview changes
2. Verification to detect drift
3. Interactive mode for manual confirmation
4. Backup and recycle bin options
5. Transaction logs for audit trails

### Q: What if I accidentally delete something?

- Use `--backup-dir` to move files instead of deleting (can recover easily)
- Use `--recycle-bin` to send files to system trash (can restore from there)
- Check `execution-log.yaml` to see exactly what was deleted
- Always run `--dry-run` first to preview changes

### Q: Is it safe to scan my entire drive?

Yes. The scanner only reads file metadata (path, size, modification time). It doesn't modify or delete anything. However, scanning very large drives may take time.

### Q: Can I add custom detection rules?

Currently, rules are built-in (build artifacts and large files). Custom rules via config files are planned for Milestone 3.

### Q: What about symlinks?

By default, symlinks are not followed to avoid potential cycles. This may be configurable in the future.

### Q: How does it handle permission errors?

- **During scan**: Permission errors are skipped, scan continues
- **During verification**: Reported as warnings (non-blocking)
- **During execution**: Logged as failures, execution continues (unless `--fail-fast`)

### Q: Can I undo an execution?

Not directly, but:
- Use `--backup-dir` to preserve files (can move back manually)
- Use `--recycle-bin` to allow OS-level recovery
- Check `execution-log.yaml` to see what was deleted
- Transaction logs provide complete audit trail

### Q: What's the difference between verify and stats?

- `stats`: Shows summary statistics from the plan file (quick, no filesystem access)
- `verify`: Checks plan against current filesystem state (slower, detects drift)

## Support

- **Issues**: Report bugs at [GitHub Issues](https://github.com/yourusername/megamaid/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/yourusername/megamaid/discussions)
- **Documentation**: See [docs/](docs/) for detailed guides

---

Made with ❤️ and Rust
