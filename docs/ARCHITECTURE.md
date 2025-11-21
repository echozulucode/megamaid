# Architecture

## Overview

Megamaid is a high-performance storage analysis and cleanup tool built with Rust. This document describes the system architecture, component interactions, and key design decisions.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                       CLI Layer                          │
│  ┌───────────┐  ┌───────────┐  ┌──────────────────┐   │
│  │  Commands │  │Orchestrator│  │ Progress Reporter│   │
│  └─────┬─────┘  └─────┬─────┘  └────────┬─────────┘   │
└────────┼──────────────┼──────────────────┼─────────────┘
         │              │                  │
         ▼              ▼                  ▼
┌─────────────────────────────────────────────────────────┐
│                    Core Components                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │ Scanner  │─>│ Detector │─>│ Planner  │             │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘             │
│       │             │              │                    │
│       ▼             ▼              ▼                    │
│  FileEntry[]  DetectionResult[]  CleanupPlan           │
│       │             │              │                    │
└───────┼─────────────┼──────────────┼────────────────────┘
        │             │              │
        ▼             ▼              ▼
┌─────────────────────────────────────────────────────────┐
│                    Data Models                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │FileEntry │  │Detection │  │ Cleanup  │             │
│  │          │  │ Result   │  │  Plan    │             │
│  └──────────┘  └──────────┘  └──────────┘             │
└─────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────┐
│               File System / Storage                      │
│  ┌──────────┐                    ┌──────────┐          │
│  │Directory │                    │TOML Plan │          │
│  │  Tree    │                    │  Files   │          │
│  └──────────┘                    └──────────┘          │
└─────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Scanner (`src/scanner/`)

**Purpose**: Traverses the file system and collects metadata about files and directories.

**Key Files**:
- `traversal.rs`: Main `FileScanner` implementation using `walkdir`
- `progress.rs`: Atomic progress tracking with `ScanProgress`
- `mod.rs`: Public API and re-exports

**Data Flow**:
```
Path → FileScanner → Vec<FileEntry>
```

**Configuration** (`ScanConfig`):
- `skip_hidden: bool` - Skip hidden files/directories
- `max_depth: Option<usize>` - Limit traversal depth
- `follow_links: bool` - Follow symlinks (currently always false)

**Features**:
- Single-threaded traversal using `walkdir` crate
- Atomic progress tracking for concurrent access
- Configurable depth limiting and hidden file handling
- Error handling with `ScanError` type

**Performance Characteristics**:
- Memory: ~100MB for 100K files
- Speed: ~100K files in <60s on SSD
- Traversal order: Depth-first

### 2. Detector (`src/detector/`)

**Purpose**: Applies configurable rules to identify cleanup candidates from scanned entries.

**Key Files**:
- `rules.rs`: Detection rule trait and built-in rules
- `engine.rs`: `DetectionEngine` that orchestrates rules
- `mod.rs`: Public API and re-exports

**Data Flow**:
```
Vec<FileEntry> → DetectionEngine → Vec<DetectionResult>
```

**Rule System**:

The detector uses a trait-based plugin architecture:

```rust
pub trait DetectionRule: Send + Sync {
    fn name(&self) -> &str;
    fn should_flag(&self, entry: &FileEntry, context: &ScanContext) -> bool;
    fn reason(&self) -> String;
}
```

**Built-in Rules**:

1. **SizeThresholdRule**
   - Flags files exceeding a size threshold
   - Default: 100MB
   - Configurable threshold in bytes

2. **BuildArtifactRule**
   - Detects common build artifact directories
   - Patterns: `target`, `node_modules`, `build`, `.next`, `dist`, `__pycache__`, `.pytest_cache`, `bin`, `obj`
   - Directory-only, case-sensitive matching

**Rule Orchestration**:
- Rules applied in order
- First matching rule wins (one detection per entry)
- Custom rules can be added via `add_rule()`

**ScanContext**:
- Currently minimal
- Future: parent directory info, aggregate statistics, etc.

### 3. Planner (`src/planner/`)

**Purpose**: Converts detection results into human-editable TOML cleanup plans.

**Key Files**:
- `generator.rs`: `PlanGenerator` for creating plans
- `writer.rs`: `PlanWriter` for atomic file I/O
- `mod.rs`: Public API and re-exports

**Data Flow**:
```
Vec<DetectionResult> → PlanGenerator → CleanupPlan → TOML File
```

**Plan Generation Process**:

1. **Create Plan Structure**
   - Set version, timestamp, base path
   - Initialize empty entries vec

2. **Convert Detections**
   - Transform absolute paths to relative paths
   - Convert `SystemTime` to RFC3339 strings
   - Determine default action based on rule:
     - `build_artifact` → `Delete`
     - `large_file` → `Review`
     - Other → `Review` (conservative default)

3. **Serialize to TOML**
   - Use `toml` crate for serialization
   - Pretty-print for human readability
   - Atomic write via temp file + rename

**Atomic Write Process**:

```rust
// 1. Create parent directories if needed
fs::create_dir_all(parent)?;

// 2. Write to temporary file
let temp_path = target.with_extension("tmp");
fs::write(&temp_path, content)?;

// 3. Atomic rename (replaces existing file)
fs::rename(&temp_path, target)?;
```

This ensures the plan file is never left in a corrupted state.

### 4. Models (`src/models/`)

**Purpose**: Core data structures used throughout the application.

**Key Files**:
- `file_entry.rs`: File system entry representation
- `cleanup_plan.rs`: Cleanup plan and entry structures
- `mod.rs`: Public API and re-exports

**Data Structures**:

```rust
// File system entry
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub entry_type: EntryType,  // File | Directory
    pub file_id: Option<u64>,   // NTFS MFT record number (future)
}

// Cleanup plan
pub struct CleanupPlan {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub base_path: PathBuf,
    pub entries: Vec<CleanupEntry>,
}

// Individual cleanup entry
pub struct CleanupEntry {
    pub path: String,           // Relative to base_path
    pub size: u64,
    pub modified: String,       // RFC3339 format
    pub action: CleanupAction,  // Delete | Keep | Review
    pub rule_name: String,
    pub reason: String,
}
```

### 5. CLI (`src/cli/`)

**Purpose**: Command-line interface with argument parsing and user interaction.

**Key Files**:
- `commands.rs`: Clap command definitions
- `orchestrator.rs`: Command execution logic
- `mod.rs`: Public API and `run_command()` entry point

**Commands**:

1. **scan**
   ```
   megamaid scan <PATH> [OPTIONS]
   ```
   - Scans directory and generates cleanup plan
   - Options: output path, size threshold, max depth, skip hidden

2. **stats**
   ```
   megamaid stats <PLAN_FILE>
   ```
   - Displays statistics about a plan
   - No modifications made

**Progress Reporting**:
- Uses `indicatif` crate for progress bars
- Real-time updates during scan
- Shows: files scanned, bytes processed, elapsed time

**Error Handling**:
- Uses `anyhow::Result` for error propagation
- User-friendly error messages
- Exit codes: 0 (success), 1 (error)

## Data Flow

### Complete Workflow

```
1. User runs: megamaid scan /path/to/directory

2. CLI Layer:
   ├─ Parse arguments (clap)
   ├─ Create ScanConfig from options
   └─ Call orchestrator

3. Scanner:
   ├─ Traverse directory tree (walkdir)
   ├─ Collect FileEntry for each file/dir
   ├─ Update progress (atomic counters)
   └─ Return Vec<FileEntry>

4. Detector:
   ├─ For each FileEntry:
   │  ├─ Try each detection rule
   │  ├─ First match wins
   │  └─ Create DetectionResult if flagged
   └─ Return Vec<DetectionResult>

5. Planner:
   ├─ Create CleanupPlan with metadata
   ├─ For each DetectionResult:
   │  ├─ Convert to CleanupEntry
   │  ├─ Relativize path
   │  ├─ Determine default action
   │  └─ Add to plan.entries
   └─ Return CleanupPlan

6. Writer:
   ├─ Serialize plan to TOML
   ├─ Write to temp file
   ├─ Atomic rename to final path
   └─ Return success

7. CLI:
   ├─ Display summary statistics
   └─ Exit with code 0
```

## Key Design Decisions

### 1. Why TOML for Plans?

**Decision**: Use TOML instead of JSON or YAML.

**Rationale**:
- Human-editable format
- Clear, unambiguous syntax
- Native Rust support via `toml` crate
- Non-technical users can understand it
- Comments possible for annotations

**Trade-offs**:
- Larger file size than JSON
- Slower parsing than binary formats
- Limited nesting (fine for our use case)

### 2. Why Trait-Based Detection Rules?

**Decision**: Use `DetectionRule` trait instead of hardcoded logic.

**Rationale**:
- **Extensibility**: Users can add custom rules
- **Testability**: Rules tested in isolation
- **Composability**: Mix and match rules without coupling
- **Maintainability**: Each rule is self-contained

**Implementation**:
```rust
pub trait DetectionRule: Send + Sync {
    fn name(&self) -> &str;
    fn should_flag(&self, entry: &FileEntry, context: &ScanContext) -> bool;
    fn reason(&self) -> String;
}
```

### 3. Why Relative Paths in Plans?

**Decision**: Store paths relative to `base_path` instead of absolute.

**Rationale**:
- Plans are portable (can be moved/shared)
- Shorter, more readable paths
- Works across different mount points
- User can move plan file without invalidation

**Example**:
```toml
base_path = "/home/user/projects/myapp"

[[entries]]
path = "target/debug"  # Not "/home/user/projects/myapp/target/debug"
```

### 4. Why Atomic File I/O?

**Decision**: Use temp file + rename pattern for writing plans.

**Rationale**:
- **Atomicity**: File is never partially written
- **Crash safety**: Process interruption doesn't corrupt file
- **Concurrent safety**: Readers never see incomplete data
- **Standard pattern**: Used by many Unix tools

**Implementation**:
```rust
let temp_path = output_path.with_extension("tmp");
fs::write(&temp_path, toml)?;
fs::rename(temp_path, output_path)?;  // Atomic on POSIX/Windows
```

### 5. Why Single-Threaded Scanner?

**Decision**: Use sequential traversal in Milestone 1.

**Rationale**:
- Simpler implementation and testing
- Sufficient performance for Milestone 1 goals
- HDD performance often bottlenecked by seeks, not CPU
- Parallel scanning deferred to Milestone 3

**Future**: Will add parallel scanning with `rayon` or `jwalk`.

## Error Handling

### Strategy

1. **Recoverable Errors**: Continue operation, log error
   - Example: Permission denied on single file
   - Action: Skip file, add to error log, continue scan

2. **Fatal Errors**: Stop operation, report to user
   - Example: Output directory not writable
   - Action: Return error immediately

### Error Types

```rust
// Scanner errors
pub enum ScanError {
    Io(io::Error),
    InvalidPath(String),
}

// Writer errors
pub enum WriteError {
    Serialization(toml::ser::Error),
    Io(io::Error),
    Validation(String),
}
```

### User-Facing Errors

- Clear, actionable error messages
- Suggest fixes when possible
- Exit codes: 0 (success), 1 (error)

## Performance Considerations

### Memory Usage

**Strategy**: Keep all entries in memory for Milestone 1.

**Current**: ~100MB for 100K files

**Components**:
- `FileEntry`: ~120 bytes each (path, metadata)
- `DetectionResult`: ~200 bytes each (entry + strings)
- `CleanupEntry`: ~150 bytes each (serialized metadata)

**Future Optimizations** (if needed):
- Stream processing for very large datasets
- On-disk intermediate storage
- Incremental plan generation

### I/O Patterns

**Scanner**:
- Sequential directory traversal
- Metadata-only reads (no file content)
- Optimized for SSD (parallel may help HDD in future)

**Writer**:
- Single write per plan generation
- Buffered I/O via `std::fs`
- Atomic via rename (no additional syscalls)

### CPU Usage

**Scanner**: Low (mostly I/O bound)
**Detector**: Low (simple pattern matching)
**Planner**: Low (string formatting, TOML serialization)

**Bottleneck**: Typically I/O (disk seek time, metadata reads)

## Testing Strategy

### Test Organization

```
src/                # Unit tests (in-module #[cfg(test)])
├── models/*.rs     # Model unit tests
├── scanner/*.rs    # Scanner unit tests
├── detector/*.rs   # Detector unit tests
├── planner/*.rs    # Planner unit tests
└── cli/*.rs        # CLI unit tests

tests/              # Integration tests
├── scanner_integration.rs  # Scanner integration tests
├── scanner_properties.rs   # Property-based tests
├── feature_matrix_tests.rs # Feature combination tests
└── performance_tests.rs    # Performance benchmarks
```

### Test Types

1. **Unit Tests** (66 tests)
   - Test individual components in isolation
   - Mock dependencies when needed
   - Fast, run on every build

2. **Integration Tests** (5 tests)
   - Test component interactions
   - Use real filesystem (via `tempfile`)
   - Realistic project structures

3. **Property-Based Tests** (5 tests)
   - Use `proptest` for fuzzing
   - Test invariants across random inputs
   - Example: path depth respects max_depth

4. **Feature Matrix Tests** (8 tests)
   - Test feature combinations
   - Example: skip_hidden + max_depth
   - Ensure orthogonal features work together

5. **Performance Tests** (7 tests, 6 ignored)
   - Benchmark large-scale operations
   - Run with `--ignored` flag
   - Create 10K-100K test files

6. **Doc Tests** (2 tests)
   - Verify documentation examples compile
   - Keep docs in sync with code

### Coverage Targets

| Component | Target | Status |
|-----------|--------|--------|
| Models    | 100%   | ✅     |
| Scanner   | 90%    | ✅     |
| Detector  | 90%    | ✅     |
| Planner   | 85%    | ✅     |
| CLI       | 80%    | ✅     |

### Running Tests

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

# With coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

## Future Enhancements

### Milestone 2: Execution & Safety

- **Drift Detection**
  - Verify file metadata before deletion
  - Detect renames using file IDs (NTFS)
  - Hash verification for paranoid mode

- **Safe Deletion**
  - Recycle bin support (Windows)
  - Trash support (Linux via `trash` crate)
  - Confirmation prompts
  - Dry-run mode

### Milestone 3: Performance

- **Parallel Scanning**
  - Use `rayon` or `jwalk` for parallel traversal
  - CPU utilization for metadata processing
  - Scalable to millions of files

- **NTFS Optimizations** (Windows-specific)
  - Direct MFT scanning
  - Bypass filesystem cache
  - File ID tracking for rename detection

### Milestone 4: GUI

- **Tauri Integration**
  - Modern web-based UI
  - Visual tree map (DaisyDisk-style)
  - Interactive plan editing
  - Real-time scan progress

### Milestone 5: Advanced Features

- **Custom Rules**
  - User-defined detection patterns
  - Rule configuration files
  - Regex-based matching

- **Archiving**
  - Compress before delete
  - TAR+Zstd archives
  - Restore capability

## Dependencies

### Core Dependencies

- `walkdir` - Directory traversal
- `serde` + `toml` - Serialization
- `chrono` - Timestamp handling
- `clap` - CLI argument parsing
- `indicatif` - Progress bars
- `rayon` - Parallel processing (future)
- `anyhow` + `thiserror` - Error handling

### Development Dependencies

- `tempfile` - Test fixtures
- `proptest` - Property-based testing
- `criterion` - Benchmarking
- `assert_cmd` - CLI testing
- `predicates` - Test assertions

## Platform Support

### Current (Milestone 1)

- ✅ Windows 11 (primary target)
- ✅ Linux (tested)
- ⚠️  macOS (should work, untested)

### Future

- Windows-specific optimizations (NTFS MFT)
- Linux-specific features (extended attributes)
- macOS testing and support

## Build & Release

### Debug Build

```bash
cargo build
# Output: target/debug/megamaid
```

### Release Build

```bash
cargo build --release
# Output: target/release/megamaid
```

### Optimization Flags

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
```

## Changelog

See [CHANGELOG.md](../CHANGELOG.md) for version history.

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
