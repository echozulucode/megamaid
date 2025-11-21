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

**Milestone 1 (Directory Scan & Plan Generation) - COMPLETED ✅**

Source code follows this architecture:

```
src/
├── models/          # Core data structures (FileEntry, CleanupPlan, CleanupAction)
│   ├── file_entry.rs    # FileEntry with metadata and drift detection fields
│   ├── cleanup_plan.rs  # CleanupPlan, CleanupEntry, CleanupAction
│   └── mod.rs
├── scanner/         # Directory traversal and metadata collection
│   ├── traversal.rs # FileScanner with configurable traversal
│   ├── progress.rs  # Atomic progress tracking with AtomicUsize
│   └── mod.rs
├── detector/        # Cleanup candidate identification
│   ├── rules.rs     # DetectionRule trait, SizeThresholdRule, BuildArtifactRule
│   ├── engine.rs    # DetectionEngine with rule orchestration
│   └── mod.rs
├── planner/         # YAML plan generation
│   ├── generator.rs # PlanGenerator - converts detections to plans
│   ├── writer.rs    # PlanWriter - atomic file I/O with validation
│   └── mod.rs
├── cli/             # Command-line interface
│   ├── commands.rs  # Clap argument definitions (scan, stats)
│   ├── orchestrator.rs # Command execution and orchestration
│   └── mod.rs
├── lib.rs           # Library root with public API
└── main.rs          # CLI entry point
```

## Architecture Patterns

### Data Flow (Milestone 1)

```
User Input → Scanner → FileEntry[]
                ↓
          Detector → DetectionResult[]
                ↓
          Planner → CleanupPlan
                ↓
          YAML File → User Review → (Milestone 2: Execution)
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
- **serde_yaml**: Serialization for cleanup plans
- **clap**: CLI argument parsing
- **indicatif**: Progress bars
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
└── cli/*.rs         # CLI unit tests

tests/               # Integration tests
├── scanner_integration.rs     # Scanner integration tests (5 tests)
├── scanner_properties.rs      # Property-based tests (5 tests)
├── feature_matrix_tests.rs    # Feature combination tests (8 tests)
├── performance_tests.rs       # Performance tests (1 quick + 6 ignored)
└── benches/
    └── scanner_benchmarks.rs  # Performance benchmarks (placeholder)
```

**Current Test Count: 86 tests passing**

- 66 unit tests
- 5 integration tests
- 5 property-based tests
- 8 feature matrix tests
- 1 quick performance test (+ 6 ignored long-running tests)
- 1 doc test

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

## Performance Targets

**Milestone 1 Goals:**

- Scan 100K files in <60 seconds (SSD)
- Scan 1M files in <5 minutes (SSD)
- Memory usage <100MB for 100K file scan
- Plan generation <30s for 1M entries
- YAML serialization <5s for 100K entries

**Optimization Notes:**

- Single-threaded traversal in Phase 1.2
- Parallel processing added in Milestone 3
- NTFS MFT scanning (Windows-specific) deferred to Milestone 5
- Consider streaming for very large datasets to avoid memory bloat

## Implementation Phases

**Current Status: Milestone 1 COMPLETE ✅**

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

**Total: 86 tests passing, full end-to-end functionality with property-based and performance testing**

Refer to `docs/plan-001-milestone1-implementation.md` for detailed phase breakdown.

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

## Research Documentation

See `docs/ai/research/` for:

- Performance analysis (NTFS MFT scanning, multi-threading strategies)
- UX design patterns (DaisyDisk, WizTree inspiration)
- Archiving strategies (ZIP vs TAR+Zstd)
- Windows-specific optimizations

## Future Milestones

- **Milestone 2**: Plan verification & deletion execution
- **Milestone 3**: Parallel operations & concurrency
- **Milestone 4**: Tauri GUI integration
- **Milestone 5**: NTFS optimizations & cross-platform support

## License

MIT License - See LICENSE file
