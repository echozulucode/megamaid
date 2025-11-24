# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Megamaid** is a high-performance storage analysis and cleanup tool built with Rust and Tauri, targeting Windows 11 (with future Linux support). The tool scans directories to identify cleanup candidates (build artifacts, large files), generates human-editable YAML cleanup plans, and executes safe deletions with drift detection.

## Development Philosophy

This project prioritizes:

- **Safety first**: All deletions require user review via YAML plans with drift detection before execution
- **Performance**: Target is scanning 1M+ files in <5 minutes on SSD, with <100MB memory usage for 100K files
- **Test-driven development**: >85% code coverage requirement with comprehensive unit, integration, E2E, and property-based tests
- **Cross-platform abstractions**: OS-specific code isolated behind traits/modules for Windows/Linux compatibility

## Project Structure

**Milestones 1, 2, & Phase 3.1, 3.2, 3.3 COMPLETED ✅**

Source code follows this architecture:

```
src/
├── models/          # Core data structures (FileEntry, CleanupPlan, CleanupAction)
│   ├── file_entry.rs    # FileEntry with metadata and drift detection fields
│   ├── cleanup_plan.rs  # CleanupPlan, CleanupEntry, CleanupAction
│   └── mod.rs
├── scanner/         # Directory traversal and metadata collection
│   ├── traversal.rs # FileScanner with configurable traversal
│   ├── parallel.rs  # ParallelScanner with rayon-based work-stealing
│   ├── progress.rs  # AdvancedProgress with ETA and throughput tracking
│   └── mod.rs
├── detector/        # Cleanup candidate identification
│   ├── rules.rs     # DetectionRule trait, SizeThresholdRule, BuildArtifactRule
│   ├── engine.rs    # DetectionEngine with rule orchestration
│   └── mod.rs
├── planner/         # YAML plan generation
│   ├── generator.rs # PlanGenerator - converts detections to plans
│   ├── writer.rs    # PlanWriter - atomic file I/O with validation
│   └── mod.rs
├── verifier/        # Plan verification and drift detection
│   ├── engine.rs    # VerificationEngine with drift detection
│   ├── report.rs    # DriftReporter for human-readable reports
│   └── mod.rs
├── executor/        # Safe deletion execution
│   ├── engine.rs    # ExecutionEngine with parallel and sequential modes
│   ├── transaction.rs # TransactionLogger for audit trails
│   └── mod.rs
├── config/          # Configuration management
│   ├── schema.rs    # Configuration data structures with serde support
│   ├── loader.rs    # Config file loading and parsing
│   ├── validation.rs # Config validation with comprehensive checks
│   └── mod.rs
├── cli/             # Command-line interface
│   ├── commands.rs  # Clap argument definitions (scan, stats, verify, execute)
│   ├── orchestrator.rs # Command execution and orchestration
│   └── mod.rs
├── lib.rs           # Library root with public API
└── main.rs          # CLI entry point
```

## Architecture Patterns

### Complete Data Flow (Milestones 1 & 2)

```
User Input → Scanner → FileEntry[]
                ↓
          Detector → DetectionResult[]
                ↓
          Planner → CleanupPlan
                ↓
          YAML File → User Review/Edit
                ↓
          Verifier → Drift Check
                ↓
          Executor → Safe Deletion
                ↓
          Transaction Log → Audit Trail
```

### Detection Rule System

The detector uses a trait-based plugin architecture:

- `DetectionRule` trait defines `should_flag()`, `name()`, and `reason()`
- Built-in rules: `SizeThresholdRule`, `BuildArtifactRule` (detects target/, node_modules/, etc.)
- Engine applies rules in order; first match wins (one detection per file)
- Custom rules can be added via `DetectionEngine::add_rule()`

### OS Abstraction Layer (Future)

Windows-specific code (NTFS optimizations, Recycle Bin, file IDs) will be isolated behind:

- Trait-based filesystem operations
- `#[cfg(windows)]` conditional compilation
- OS-agnostic public APIs in core modules

## Key Dependencies

- **walkdir**: Directory traversal (will upgrade to parallel jwalk or ignore crate)
- **rayon**: Parallel processing (Milestone 3+)
- **serde_yaml**: Serialization for cleanup plans and transaction logs
- **clap**: CLI argument parsing
- **indicatif**: Progress bars
- **uuid**: Unique execution IDs for transaction logs
- **trash**: Cross-platform recycle bin support
- **chrono**: Timestamp handling for drift detection
- **tempfile**: Test fixtures
- **proptest**: Property-based testing
- **criterion**: Benchmarking
- **assert_cmd**: End-to-end CLI testing

## Testing Strategy

### Test Organization

```
src/                 # Unit tests (in-module #[cfg(test)])
├── models/*.rs      # Model unit tests
├── scanner/*.rs     # Scanner unit tests
├── detector/*.rs    # Detector unit tests
├── planner/*.rs     # Planner unit tests
├── verifier/*.rs    # Verifier unit tests (8 tests)
├── executor/*.rs    # Executor unit tests (22 tests)
└── cli/*.rs         # CLI unit tests

tests/               # Integration tests
├── scanner_integration.rs     # Scanner integration tests (5 tests)
├── scanner_properties.rs      # Property-based tests (5 tests)
├── feature_matrix_tests.rs    # Feature combination tests (8 tests)
├── performance_tests.rs       # Performance tests (1 quick + 6 ignored)
└── benches/
    └── scanner_benchmarks.rs  # Performance benchmarks (placeholder)
```

**Current Test Count: 198 tests passing**

- 155 unit tests
  - 66 from Milestone 1 (models, scanner, detector, planner)
  - 17 from verifier module (Milestone 2)
  - 22 from executor module (Milestone 2)
  - 18 from parallel scanner module (Milestone 3)
  - 9 from scanner progress module
  - 7 from CLI module
  - 30 from config module (Milestone 3)
  - (14 additional from miscellaneous modules)
- 8 feature matrix tests
- 5 scanner integration tests
- 5 executor integration tests (Milestone 3)
- 5 end-to-end integration tests (Milestone 3)
- 8 config integration tests (Milestone 3)
- 5 property-based tests
- 3 documentation tests
- 4 performance tests (+ 6 ignored long-running tests)

### Test Patterns

- **Unit tests**: Use `tempfile::TempDir` for filesystem fixtures
- **Mocking**: Create helper functions like `create_test_entry()`, `create_realistic_rust_project()`
- **Assertions**: Test both positive cases and edge cases (empty dirs, unicode paths, max depth)
- **Parallelism**: Verify atomic operations with concurrent test threads

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# E2E tests
cargo test --test '*_e2e_*'

# Property-based tests
cargo test --test '*_property_*'

# Performance tests (long-running)
cargo test --ignored

# Single test
cargo test test_scan_empty_directory

# With output
cargo test -- --nocapture

# Benchmarks
cargo bench
```

### Coverage Requirements

| Component | Target |
| --------- | ------ |
| Models    | 100%   |
| Scanner   | 90%    |
| Detector  | 90%    |
| Planner   | 85%    |
| CLI       | 80%    |

## Quality Assurance & CI/CD

**CRITICAL: Run all quality checks upon completion of every phase and major milestone. Fix ALL issues before marking work as complete.**

GitHub Actions runs these checks automatically on every push. To avoid CI failures, run these tools locally before committing:

### Pre-Commit Quality Checklist

Run the following commands in order. All must pass without errors or warnings:

```bash
# 1. Format check (must pass before other checks)
cargo fmt --all --check

# 2. Auto-fix formatting issues
cargo fmt --all

# 3. Clippy lints (strict: no warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# 4. Run all unit and integration tests
cargo test --all

# 5. Run documentation tests
cargo test --doc

# 6. Build in release mode (catches optimization issues)
cargo build --release

# 7. Check for unused dependencies (optional but recommended)
cargo +nightly udeps
```

### When to Run Quality Checks

**MANDATORY quality check points:**

1. **After completing each phase** (e.g., Phase 3.1, Phase 3.2)
   - Run full quality checklist
   - Fix all clippy warnings
   - Ensure all tests pass
   - Verify formatting is correct

2. **Before marking a milestone complete** (e.g., Milestone 3)
   - Run full quality checklist
   - Run long-running performance tests: `cargo test --ignored`
   - Review and update test counts in this document
   - Update documentation to reflect new features

3. **Before creating commits**
   - At minimum: `cargo fmt --all && cargo clippy -- -D warnings && cargo test`
   - Fix all issues before committing

4. **Before creating pull requests**
   - Run complete quality checklist
   - Verify CI will pass
   - Update CLAUDE.md with progress notes

### Common Issues & Fixes

**Formatting issues:**
```bash
# Auto-fix all formatting
cargo fmt --all
```

**Clippy warnings:**
```bash
# See all warnings with explanations
cargo clippy --all-targets --all-features -- -D warnings

# Common fixes:
# - Remove unused imports
# - Add #[allow(dead_code)] for test helpers
# - Use .map_err() instead of match for error conversion
# - Replace .clone() with borrowing where possible
```

**Test failures:**
```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Run tests in single thread (for debugging race conditions)
cargo test -- --test-threads=1

# Run ignored long-running tests
cargo test --ignored
```

### GitHub Actions Workflow

The CI pipeline runs on every push and pull request:

1. **Format Check**: `cargo fmt --check` (fails if not formatted)
2. **Clippy**: `cargo clippy -- -D warnings` (fails on any warning)
3. **Tests**: `cargo test --all` (fails if any test fails)
4. **Build**: `cargo build --release` (fails if build fails)

**If CI fails:**
1. Pull the latest changes
2. Run the exact failing command locally
3. Fix all issues
4. Re-run full quality checklist
5. Commit fixes and push

### Quality Standards

- **Zero warnings policy**: Clippy must report 0 warnings
- **100% test pass rate**: All tests must pass on all platforms
- **Consistent formatting**: Use rustfmt defaults (no custom config)
- **No ignored clippy lints**: Fix issues instead of adding #[allow()]
- **Documentation coverage**: All public APIs must have doc comments

## Performance Targets

**Milestone 1 & 2 Goals (ACHIEVED ✅):**

- ✅ Scan 100K files in <60 seconds (SSD)
- ✅ Verify 100K entries in <10 seconds
- ✅ Execute 10K deletions in <60 seconds
- ✅ Memory usage <150MB for 100K entries
- ✅ Plan generation <30s for 1M entries
- ✅ YAML serialization <5s for 100K entries

**Milestone 3 Goals (ACHIEVED ✅):**

- ✅ Parallel scanning with configurable thread count
- ✅ 2.3x speedup for parallel deletion (measured)
- ✅ 217K+ files/sec throughput with parallel scanner
- ✅ Complete pipeline (scan+detect+plan+verify) in <20ms for 1.5K files
- ✅ Memory efficiency: <10MB for 5K entries
- ✅ Configuration file support with YAML
- ✅ CLI argument override of config values

**Measured Performance (Milestone 3):**

- Parallel scan: 5,000 files in 23ms (217,194 files/sec)
- Complete pipeline: 1,506 files processed in 17ms total
  - Scan: 9.9ms
  - Detect: 379µs
  - Plan generation: 854µs
  - Verify: 6.2ms
- Memory: 351KB for 5,000 entries
- Parallel execution: 2.3x faster than sequential for deletion

**Optimization Notes:**

- ✅ Parallel processing implemented in Milestone 3
- Configurable thread count (0 = auto-detect)
- Batch-based parallel execution for better error handling
- NTFS MFT scanning (Windows-specific) deferred to Milestone 5
- Transaction logging adds minimal overhead (<5%)
- Memory-efficient design scales linearly

## Implementation Phases

**Current Status: Milestone 4 IN PROGRESS 🚧 (Phase 4.1: Tauri Setup & Foundation)**

### Milestone 1: Directory Scan & Plan Generation ✅

All phases of Milestone 1 have been successfully implemented:

1. ✅ **Phase 1.1 - Core Data Models**
   - FileEntry, CleanupPlan, CleanupEntry, CleanupAction
   - YAML serialization/deserialization
   - 8 unit tests

2. ✅ **Phase 1.2 - File Traversal**
   - FileScanner with configurable options (max_depth, skip_hidden)
   - Atomic ScanProgress tracking
   - 8 unit tests + 5 integration tests

3. ✅ **Phase 1.3 - Artifact Detection**
   - DetectionRule trait system
   - SizeThresholdRule, BuildArtifactRule
   - DetectionEngine with rule orchestration
   - 20 unit tests

4. ✅ **Phase 1.4 - YAML Plan Generation**
   - PlanGenerator with path relativization
   - PlanWriter with atomic file operations
   - 15 unit tests

5. ✅ **Phase 1.5 - CLI Interface**
   - Full CLI with `scan` and `stats` commands
   - Progress reporting with indicatif
   - Error handling with anyhow
   - 7 unit tests

**Milestone 1 Total: 86 tests passing**

### Milestone 2: Plan Verification & Execution ✅

All phases of Milestone 2 have been successfully implemented:

1. ✅ **Phase 2.1 - Plan Verification & Drift Detection**
   - VerificationEngine with configurable checks (size, mtime, fail-fast)
   - Multi-level drift detection (missing, size mismatch, mtime mismatch)
   - DriftReporter for human-readable reports
   - Recursive directory size calculation
   - CLI `verify` command with options
   - 17 unit tests

2. ✅ **Phase 2.2 - Deletion Engine & Safety**
   - ExecutionEngine with three modes (DryRun, Interactive, Batch)
   - Multiple deletion methods (direct, backup, recycle bin)
   - User prompts for interactive mode
   - Fail-fast support with detailed error reporting
   - Only processes Delete actions (skips Keep/Review)
   - CLI `execute` command with comprehensive options
   - 13 unit tests

3. ✅ **Phase 2.3 - Transaction Logging**
   - TransactionLog with UUID-based execution IDs
   - Per-operation logging with timestamps
   - Atomic writes (temp + rename pattern)
   - Summary statistics (successful/failed/skipped/space freed)
   - Transaction status tracking (in_progress/completed/failed/aborted)
   - Logged even for dry-run executions
   - 9 unit tests

**Milestone 2 Total: 40 new tests (126 total)**

### Milestone 3 - Phase 3.2: Parallel Deletion ✅

**Status:** COMPLETED

All components of Phase 3.2 have been successfully implemented:

1. ✅ **Parallel Execution Engine**
   - Added `parallel` and `batch_size` fields to ExecutionConfig
   - Implemented batched parallel processing using rayon
   - Thread-safe result collection with Arc<Mutex<>>
   - Fail-fast support across batches
   - 5 new unit tests

2. ✅ **Progress Tracking Integration**
   - Integrated AdvancedProgress with ExecutionEngine
   - Real-time progress updates during parallel execution
   - Thread-safe atomic counters

3. ✅ **CLI Integration**
   - Added `--parallel` flag to enable parallel execution
   - Added `--batch-size` flag (default: 100)
   - Visual indicator for parallel mode
   - Full backward compatibility

4. ✅ **Integration Tests**
   - 5 comprehensive end-to-end tests
   - Performance benchmarking showing **2.3x speedup**
   - Nested directory handling
   - Error handling and fail-fast validation

**Phase 3.2 Performance:**
- **Measured speedup: 2.2-2.4x** for deletion operations
- Batch size configurable (default: 100 files per batch)
- Works with dry-run, backup, and recycle bin modes
- Not compatible with Interactive mode (validation enforced)

**Phase 3.2 Total: 10 new tests (5 unit + 5 integration = 136 total)**

### Milestone 3 - Phase 3.1: Parallel Scanning ✅

**Status:** COMPLETED (prior work)

- ParallelScanner with rayon-based work-stealing
- AdvancedProgress with ETA and throughput tracking
- ErrorCollector for parallel error handling
- Cross-platform hidden file detection (depth 0 filtering for Linux)
- 9 unit tests for parallel scanner

**Phase 3.1 Total: 14 new tests (9 unit + 5 integration = 120 total)**

### Milestone 3 - Phase 3.3: Configuration File Support ✅

**Status:** COMPLETED

All components of Phase 3.3 have been successfully implemented:

1. ✅ **Configuration Schema**
   - Comprehensive YAML-based configuration structure
   - Scanner, detector, executor, verifier, and output settings
   - Custom detection rules support (schema defined)
   - Example config file: `megamaid.example.yaml`

2. ✅ **Config Module Implementation**
   - Configuration data structures with serde support
   - Config file loading with default fallback locations
   - Comprehensive validation with detailed error messages
   - 30 unit tests for config module

3. ✅ **Integration with Core Modules**
   - Scanner: Config controls depth, hidden files, symlinks, threads
   - Detector: Config enables/disables rules, sets thresholds
   - Executor: Config sets parallel mode, batch size, defaults
   - Verifier: Config controls validation checks
   - Conversion traits for seamless integration

4. ✅ **CLI Integration**
   - Global `--config` flag for all commands
   - Config precedence: CLI args override config values
   - Automatic config discovery from standard locations
   - Validation on load with clear error messages

**Phase 3.3 Total: 30 new tests (config module)**

**Full System Test Coverage:**
- 181 tests passing
- 155 unit tests across all modules (125 + 30 config tests)
- 8 feature matrix tests
- 10 integration tests (5 scanner + 5 executor)
- 5 property-based tests
- 2 documentation tests
- 1 quick performance test (+ 6 ignored long-running tests)

Refer to `docs/plan-001-milestone1-implementation.md` for Milestone 1 details.
Refer to `docs/ai/plans/plan-003-milestone3-implementation.md` for Milestone 3 details.

## Critical Design Decisions

### Why YAML for Plans?

- Human-editable (users manually review/modify before deletion)
- Compact and readable format (60% smaller than TOML)
- Supports comments for user annotations
- Native Rust support via `serde_yaml` crate
- Clear syntax for non-technical users
- Fast parsing performance for large datasets

### Why Trait-based Detection Rules?

- Extensibility: users can add custom rules
- Testability: rules tested in isolation
- Composability: combine rules without tight coupling

### Why Atomic File I/O for Plans?

```rust
// Write to temp file, then rename (atomic on POSIX/Windows)
let temp_path = output_path.with_extension("tmp");
File::create(&temp_path)?.write_all(yaml.as_bytes())?;
file.sync_all()?; // Ensure disk persistence
std::fs::rename(temp_path, output_path)?;
```

Prevents corruption if process interrupted during write.

### Why Relative Paths in Plans?

Plans store paths relative to `base_path` to support:

- Moving the plan file
- Sharing plans across systems
- Clearer user review (shorter paths)

### Why Three Execution Modes?

ExecutionEngine supports DryRun, Interactive, and Batch modes:

- **DryRun**: Simulates execution without actual deletion - safe preview
- **Interactive**: Manual confirmation for each file - maximum user control
- **Batch**: Automated execution - optimal for trusted plans

This provides flexibility for different safety/convenience trade-offs.

### Why Transaction Logging?

Every execution creates a transaction log with:

- **UUID execution IDs**: Unique identification for each run
- **Atomic writes**: Temp + rename pattern prevents corruption
- **Per-operation logging**: Complete audit trail with timestamps
- **Summary statistics**: Quick overview of results
- **Even for dry-runs**: Helps with debugging and verification

Benefits:
- Accountability and compliance
- Debugging failed operations
- Historical records for analysis
- Verification of what was deleted

### Why Multi-Method Deletion?

Executor supports multiple deletion methods:

- **Direct deletion**: Fast, permanent removal
- **Backup mode**: Moves files preserving directory structure - easy recovery
- **Recycle bin**: Uses OS trash - familiar recovery mechanism
- **Fail-fast option**: Stops on first error for critical operations

Users can choose based on their recovery requirements and risk tolerance.

## Drift Detection (Milestone 2)

Plans capture metadata snapshots:

- `size`: File size in bytes
- `modified`: RFC3339 timestamp
- `file_id`: Optional NTFS file ID for rename detection

Before execution, verify:

1. File still exists at expected path
2. Size matches recorded value
3. Modification time matches (or hash if paranoid mode)

If drift detected → halt and warn user.

## Build Artifact Detection

Default patterns in `BuildArtifactRule`:

- `target/` (Rust)
- `node_modules/` (Node.js)
- `build/`, `dist/` (generic)
- `.next/` (Next.js)
- `__pycache__/`, `.pytest_cache/` (Python)
- `bin/`, `obj/` (C#/C++)

Case-sensitive matching. Directories only (not files named "target").

## Configuration Management

Megamaid supports configuration files to set default behavior across all commands. Configuration files use YAML format and can customize scanner, detector, executor, verifier, and output settings.

### Configuration File Locations

Megamaid looks for configuration files in the following order:
1. File specified with `--config` flag
2. `megamaid.yaml` in current directory
3. `megamaid.yml` in current directory
4. `.megamaid.yaml` in current directory
5. `.megamaid.yml` in current directory
6. Built-in defaults if no config file is found

### Example Configuration File

See `megamaid.example.yaml` for a complete example with all available options. Basic structure:

```yaml
# Scanner settings
scanner:
  max_depth: null          # null = unlimited
  skip_hidden: true
  follow_symlinks: false
  thread_count: 0          # 0 = auto-detect

# Detector settings
detector:
  rules:
    size_threshold:
      enabled: true
      threshold_mb: 100
      action: review
    build_artifacts:
      enabled: true
      action: delete
      custom_patterns: []

# Executor settings
executor:
  parallel: false
  batch_size: 100
  default_mode: dry_run    # dry_run, interactive, or batch
  fail_fast: false
  use_recycle_bin: false
  backup_dir: null

# Output settings
output:
  plan_file: "cleanup-plan.yaml"
  log_file: "execution-log.yaml"
  drift_report: "drift-report.txt"

# Verifier settings
verifier:
  check_mtime: true
  check_size: true
  fail_fast: false
```

### Configuration Precedence

CLI arguments override configuration file values. For example:
```bash
# Config file sets parallel: false, but CLI enables it
megamaid execute plan.yaml --parallel --config my-config.yaml
# Result: Parallel execution is ENABLED (CLI wins)
```

### Using Configuration Files

```bash
# Use default config file (megamaid.yaml in current directory)
megamaid scan /path/to/directory

# Specify custom config file
megamaid scan /path/to/directory --config /path/to/config.yaml

# Config applies to all commands
megamaid execute cleanup-plan.yaml --config production.yaml
megamaid verify cleanup-plan.yaml --config production.yaml
```

### Configuration Validation

All configuration files are validated on load. Validation checks include:
- `scanner.max_depth`: Must be >= 1 or null (unlimited)
- `scanner.thread_count`: Cannot exceed 256
- `detector.rules.size_threshold.threshold_mb`: Must be 1-1,000,000 MB
- `executor.batch_size`: Must be 1-10,000
- Required fields cannot be empty (e.g., output filenames)

Invalid configurations will fail with descriptive error messages.

## CLI Usage

### Scan Command

```bash
# Basic scan
megamaid scan /path/to/directory

# Custom output file
megamaid scan /path/to/directory --output my-plan.yaml

# Limit scan depth
megamaid scan /path/to/directory --max-depth 5

# Custom large file threshold (200 MB)
megamaid scan /path/to/directory --large-file-threshold 200

# Skip hidden files (default: true)
megamaid scan /path/to/directory --skip-hidden
```

### Stats Command

```bash
# View plan statistics
megamaid stats cleanup-plan.yaml
```

Output includes:

- Base path and version info
- Entry counts by action (Delete/Review/Keep)
- Total size of flagged entries

### Verify Command

```bash
# Verify plan is safe to execute
megamaid verify cleanup-plan.yaml

# Save drift report to file
megamaid verify cleanup-plan.yaml --output drift-report.txt

# Stop on first drift detection
megamaid verify cleanup-plan.yaml --fail-fast

# Skip modification time checks
megamaid verify cleanup-plan.yaml --skip-mtime
```

Checks:
- File existence
- Size matches
- Modification time matches (within 2-second tolerance)
- Reports drift with human-readable output

### Execute Command

```bash
# Dry-run first (recommended)
megamaid execute cleanup-plan.yaml --dry-run

# Interactive mode with confirmation prompts
megamaid execute cleanup-plan.yaml --interactive

# Backup mode (safest real execution)
megamaid execute cleanup-plan.yaml --backup-dir ./backups

# Recycle bin mode (allows OS recovery)
megamaid execute cleanup-plan.yaml --recycle-bin

# Batch execution (default)
megamaid execute cleanup-plan.yaml

# Custom transaction log
megamaid execute cleanup-plan.yaml --log-file my-execution.yaml

# Fail-fast mode (stop on first error)
megamaid execute cleanup-plan.yaml --fail-fast

# Skip verification (not recommended)
megamaid execute cleanup-plan.yaml --skip-verify

# Parallel execution with default batch size (100)
megamaid execute cleanup-plan.yaml --parallel

# Parallel execution with custom batch size
megamaid execute cleanup-plan.yaml --parallel --batch-size 50

# Parallel execution with backup mode
megamaid execute cleanup-plan.yaml --parallel --backup-dir ./backups
```

Features:
- Automatic verification before execution (unless --skip-verify)
- Multiple execution modes for different safety levels
- Parallel execution support with configurable batch size
- Complete transaction logging with UUID execution IDs
- Progress reporting with indicatif
- Only processes Delete actions (skips Keep/Review)

## Research Documentation

See `docs/ai/research/` for:

- Performance analysis (NTFS MFT scanning, multi-threading strategies)
- UX design patterns (DaisyDisk, WizTree inspiration)
- Archiving strategies (ZIP vs TAR+Zstd)
- Windows-specific optimizations

## Roadmap

### Completed Milestones

- ✅ **Milestone 1**: Directory Scan & Plan Generation
  - File system scanning
  - Build artifact detection
  - Large file detection
  - YAML plan generation
  - CLI with scan and stats commands

- ✅ **Milestone 2**: Plan Verification & Deletion Execution
  - Plan verification with drift detection
  - Safe deletion execution (dry-run, interactive, batch)
  - Backup mode (move instead of delete)
  - Recycle bin support (Windows & Linux)
  - Transaction logging and audit trails
  - CLI with verify and execute commands

- ✅ **Milestone 3 - Phase 3.1**: Parallel Scanning
  - Multi-threaded directory traversal with rayon
  - Advanced progress tracking with atomic counters
  - Error collection and reporting
  - Configurable thread count

- ✅ **Milestone 3 - Phase 3.2**: Parallel Deletion
  - Parallel execution engine with batched processing
  - Thread-safe result collection
  - Fail-fast support across batches
  - CLI integration with --parallel and --batch-size flags
  - 2.3x measured speedup for deletion operations

- ✅ **Milestone 3 - Phase 3.3**: Configuration File Support
  - YAML-based configuration system
  - Scanner, detector, executor, verifier settings
  - Global --config flag with precedence rules
  - Automatic config discovery from standard locations
  - Comprehensive validation with clear error messages
  - 30 new unit tests

- ✅ **Milestone 3 - Phase 3.4**: Integration & Testing
  - 5 comprehensive end-to-end integration tests
  - Config integration tests (8 tests)
  - Performance benchmarks with real measurements
  - Parallel scan performance: 217K files/sec
  - Complete pipeline benchmarks (<20ms for 1.5K files)
  - Memory profiling tests (<10MB for 5K entries)
  - Updated performance targets documentation

**Milestone 3 Complete! ✅**
- 198 total tests passing
- All performance goals achieved
- Full parallel processing support
- Configuration system fully integrated

### Current Milestone

- **Milestone 4 - Phase 4.1**: Tauri Setup & Foundation (IN PROGRESS 🚧)
  - Installing Tauri CLI (868 crates compiling)
  - ✅ Svelte + TypeScript frontend created (megamaid-ui/)
  - ✅ Frontend dependencies installed (92 packages)
  - Pending: Tauri project initialization
  - Pending: Foundational Tauri commands
  - Pending: Basic IPC and state management
  - Pending: App layout and routing

### Future Phases

- **Milestone 4 - Phase 4.2**: Scan Interface & Progress
  - Directory selection UI
  - Scan configuration panel
  - Real-time progress display
  - Backend integration

- **Milestone 4 - Phase 4.3**: Results Visualization
  - Interactive file tree view
  - Disk usage tree map (DaisyDisk-style)
  - Statistics dashboard
  - Detail panel

- **Milestone 4 - Phase 4.4**: Plan Review & Editing
  - Plan editor interface
  - Action assignment UI
  - Plan save/load functionality
  - Verification display

- **Milestone 4 - Phase 4.5**: Execution & Monitoring
  - Execution configuration UI
  - Real-time progress monitoring
  - Results display
  - Notification system

- **Milestone 4 - Phase 4.6**: Polish & Distribution
  - UI/UX polish
  - Performance optimization
  - Windows installer
  - Documentation

### Future Milestones

- **Milestone 5**: Advanced Features & NTFS Optimizations
  - NTFS MFT scanning for Windows optimization
  - Custom detection rules from config
  - Archive mode (ZIP/TAR instead of delete)
  - Scheduled cleanup tasks

## License

MIT License - See LICENSE file
