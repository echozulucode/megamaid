# Megamaid - Quick Start Guide

Welcome to Megamaid! This guide will walk you through your first storage cleanup operation from start to finish.

## Table of Contents

- [What You'll Learn](#what-youll-learn)
- [Prerequisites](#prerequisites)
- [Workflow Overview](#workflow-overview)
- [Step 1: Scan Your First Directory](#step-1-scan-your-first-directory)
- [Step 2: Review the Generated Plan](#step-2-review-the-generated-plan)
- [Step 3: View Plan Statistics](#step-3-view-plan-statistics)
- [Step 4: Verify the Plan](#step-4-verify-the-plan)
- [Step 5: Execute the Plan (Dry Run First!)](#step-5-execute-the-plan-dry-run-first)
- [Step 6: Execute the Plan (For Real)](#step-6-execute-the-plan-for-real)
- [Step 7: Review Transaction Logs](#step-7-review-transaction-logs)
- [Common Workflows](#common-workflows)
- [Safety Best Practices](#safety-best-practices)
- [Troubleshooting](#troubleshooting)
- [Developer Quick Start](#developer-quick-start)

---

## What You'll Learn

By the end of this guide, you'll know how to:
1. Scan a directory to identify cleanup candidates
2. Review and edit a cleanup plan
3. Verify the plan is safe to execute
4. Execute the plan with various safety options
5. Review transaction logs for audit trails

## Prerequisites

- Megamaid installed (see main [README.md](../README.md) for installation instructions)
- A directory you want to analyze (we'll use `~/projects` in our examples)
- Basic familiarity with the command line

## Workflow Overview

```
Scan → Review Plan → Verify → Execute → Check Logs
```

---

## Step 1: Scan Your First Directory

Let's start by scanning a projects directory to find build artifacts and large files:

```bash
megamaid scan ~/projects
```

**What's happening?**
- Megamaid traverses all files and directories in `~/projects`
- It identifies build artifacts (like `target/`, `node_modules/`)
- It flags large files (default: >100 MB)
- Progress is shown in real-time

**Example Output:**
```
🔍 Scanning directory: /home/user/projects

Scanning: 45,231 files scanned...

✨ Scan complete!

📊 Results:
   Total files scanned: 45,231
   Build artifacts found: 8
   Large files found: 3
   Total flagged size: 4.2 GB

📝 Plan saved to: cleanup-plan.yaml

Next steps:
  1. Review the plan: cat cleanup-plan.yaml
  2. Verify the plan: megamaid verify cleanup-plan.yaml
  3. Execute the plan: megamaid execute cleanup-plan.yaml --dry-run
```

**Customizing the Scan:**

```bash
# Flag files larger than 200MB
megamaid scan ~/projects --large-file-threshold 200

# Limit scan depth to 3 levels
megamaid scan ~/projects --max-depth 3

# Save to a custom location
megamaid scan ~/projects --output ~/cleanup/my-scan.yaml
```

---

## Step 2: Review the Generated Plan

Open the cleanup plan in your favorite editor:

```bash
cat cleanup-plan.yaml
# or
vim cleanup-plan.yaml
# or
code cleanup-plan.yaml
```

**Example Plan File:**

```yaml
version: "0.1.0"
created_at: "2025-11-21T10:30:00Z"
base_path: /home/user/projects

entries:
  # Build artifact - safe to delete
  - path: rust-project/target
    size: 524288000
    modified: "2025-11-21T09:15:00Z"
    action: delete
    rule_name: build_artifact
    reason: Common build artifact directory

  # Another build artifact
  - path: web-app/node_modules
    size: 314572800
    modified: "2025-11-20T14:22:00Z"
    action: delete
    rule_name: build_artifact
    reason: Common build artifact directory

  # Large file - needs review
  - path: videos/presentation.mp4
    size: 2147483648
    modified: "2025-11-15T16:45:00Z"
    action: review
    rule_name: large_file
    reason: File exceeds size threshold of 100 MB

  # Another large file
  - path: datasets/training-data.csv
    size: 524288000
    modified: "2025-11-10T11:20:00Z"
    action: review
    rule_name: large_file
    reason: File exceeds size threshold of 100 MB
```

### Understanding Actions

Each entry has an `action` field that determines what happens:

- **`delete`** - Will be deleted during execution (typically build artifacts)
- **`review`** - Flagged for your attention, won't be deleted unless you change it
- **`keep`** - Will be preserved (you can change entries to this)

### Editing the Plan

You can manually edit actions based on your needs:

**Example 1: Keep a build artifact**
```yaml
# Change this:
- path: rust-project/target
  action: delete

# To this:
- path: rust-project/target
  action: keep
```

**Example 2: Delete a large file after review**
```yaml
# Change this:
- path: videos/presentation.mp4
  action: review

# To this:
- path: videos/presentation.mp4
  action: delete
```

**Example 3: Mark for manual review**
```yaml
# Change this:
- path: datasets/training-data.csv
  action: delete

# To this:
- path: datasets/training-data.csv
  action: review
```

---

## Step 3: View Plan Statistics

Before executing, check what the plan will do:

```bash
megamaid stats cleanup-plan.yaml
```

**Example Output:**
```
📊 Cleanup Plan Statistics

Base Path: /home/user/projects
Version:   0.1.0
Created:   2025-11-21 10:30:00

Entries:   4
  • Delete: 2
  • Review: 2
  • Keep:   0

Total Size: 3.3 GB (3,510,742,400 bytes)
```

This gives you a quick overview without accessing the filesystem.

---

## Step 4: Verify the Plan

Before executing, verify that files haven't changed since the plan was created:

```bash
megamaid verify cleanup-plan.yaml
```

**What's happening?**
- Megamaid checks if each file still exists
- Verifies file sizes match the plan
- Checks modification times haven't changed
- Reports any drift (unexpected changes)

**Success Output:**
```
🔍 Verifying cleanup plan: cleanup-plan.yaml

Verifying entries... ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100%

✅ Verification complete!

📊 Results:
   Total entries: 4
   Verified: 2
   Skipped (review/keep): 2
   Drift detected: 0
   Missing files: 0

✨ Plan is safe to execute!
```

**Drift Detected Output:**
```
⚠️  Drift detected!

❌ Files have changed since plan was created:

Missing Files (1):
  • /home/user/projects/rust-project/target

Size Mismatches (1):
  • /home/user/projects/web-app/node_modules
    Expected: 314,572,800 bytes
    Actual:   320,100,450 bytes
    Diff:     +5,527,650 bytes

⚠️  Not safe to execute! Reasons:
   • 1 file(s) missing
   • 1 file(s) have different sizes

Recommendations:
  1. Re-scan the directory to create a fresh plan
  2. Review changes manually before proceeding
  3. Use --skip-verify flag only if you're certain (not recommended)
```

**Handling Drift:**

If drift is detected, you have several options:

```bash
# Option 1: Re-scan to create a fresh plan (recommended)
megamaid scan ~/projects --output cleanup-plan-new.yaml

# Option 2: Save drift report for review
megamaid verify cleanup-plan.yaml --output drift-report.txt

# Option 3: Skip mtime checks (if only timestamps changed)
megamaid verify cleanup-plan.yaml --skip-mtime

# Option 4: Skip verification during execution (not recommended)
megamaid execute cleanup-plan.yaml --skip-verify
```

---

## Step 5: Execute the Plan (Dry Run First!)

**ALWAYS** start with a dry run to preview what will happen:

```bash
megamaid execute cleanup-plan.yaml --dry-run
```

**Example Output:**
```
🔍 Verifying plan before execution...
✅ Verification passed

🎯 Executing cleanup plan: cleanup-plan.yaml
Mode: Dry Run (simulation only)

Processing entries... ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100%

Operations (preview):
  ✓ Would delete: rust-project/target (500.0 MB)
  ✓ Would delete: web-app/node_modules (300.0 MB)
  ⊘ Skipped (review): videos/presentation.mp4
  ⊘ Skipped (review): datasets/training-data.csv

📊 Summary (Dry Run):
   Total operations: 2
   Would succeed: 2
   Would fail: 0
   Skipped: 2
   Space that would be freed: 800.0 MB

💡 This was a simulation. Use --backup-dir or remove --dry-run to actually execute.

Transaction log: execution-log.yaml
```

---

## Step 6: Execute the Plan (For Real)

Once you're confident with the dry run, choose an execution mode based on your safety needs:

### Option A: Backup Mode (Safest)

Move files to a backup directory instead of deleting:

```bash
megamaid execute cleanup-plan.yaml --backup-dir ./backups
```

**Why use this?**
- Preserves all files in a backup location
- Maintains directory structure
- Easy to recover if you change your mind
- Best for first-time users

**Example Output:**
```
🔍 Verifying plan before execution...
✅ Verification passed

🎯 Executing cleanup plan: cleanup-plan.yaml
Mode: Batch (backup to ./backups)

Processing entries... ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100%

Operations:
  ✓ Backed up: rust-project/target → ./backups/rust-project/target (500.0 MB)
  ✓ Backed up: web-app/node_modules → ./backups/web-app/node_modules (300.0 MB)
  ⊘ Skipped (review): videos/presentation.mp4
  ⊘ Skipped (review): datasets/training-data.csv

✅ Execution complete!

📊 Summary:
   Total operations: 2
   Successful: 2
   Failed: 0
   Skipped: 2
   Space freed: 800.0 MB
   Duration: 2.3s

💡 Files backed up to: ./backups
   You can safely delete the backup directory after verification.

Transaction log: execution-log.yaml
```

### Option B: Recycle Bin Mode (Easy Recovery)

Send files to the system recycle bin:

```bash
megamaid execute cleanup-plan.yaml --recycle-bin
```

**Why use this?**
- Files go to Windows Recycle Bin or Linux Trash
- Can restore using your OS tools
- Simpler than backup mode
- Good for typical cleanup operations

### Option C: Interactive Mode (Manual Control)

Confirm each deletion manually:

```bash
megamaid execute cleanup-plan.yaml --interactive
```

**Example Output:**
```
🔍 Verifying plan before execution...
✅ Verification passed

🎯 Executing cleanup plan: cleanup-plan.yaml
Mode: Interactive

Delete this entry? (y/n/a to abort)
  Path: rust-project/target
  Size: 500.0 MB
  Reason: Common build artifact directory
[y/n/a]: y

  ✓ Deleted: rust-project/target (500.0 MB freed)

Delete this entry? (y/n/a to abort)
  Path: web-app/node_modules
  Size: 300.0 MB
  Reason: Common build artifact directory
[y/n/a]: n

  ⊘ Skipped: web-app/node_modules (user declined)

✅ Execution complete!

📊 Summary:
   Total operations: 1
   Successful: 1
   Failed: 0
   Skipped: 3
   Space freed: 500.0 MB
```

### Option D: Batch Mode (Default)

Delete files directly without prompts:

```bash
megamaid execute cleanup-plan.yaml
```

**Why use this?**
- Fastest execution
- No manual intervention
- Best for trusted plans
- **Use only after thorough review!**

---

## Step 7: Review Transaction Logs

Every execution creates a transaction log for audit purposes:

```bash
cat execution-log.yaml
```

**Example Transaction Log:**

```yaml
version: "0.1.0"
execution_id: e52af49c-8b50-450f-92d7-3550a7f62e28
plan_file: cleanup-plan.yaml
started_at: "2025-11-21T15:28:06Z"
completed_at: "2025-11-21T15:28:08Z"
status: completed
mode: batch

options:
  dry_run: false
  backup_dir: null
  use_recycle_bin: false
  fail_fast: false

operations:
  - path: rust-project/target
    action: Delete
    status: Success
    size_freed: 524288000
    error: null
    timestamp: "2025-11-21T15:28:06Z"

  - path: web-app/node_modules
    action: Delete
    status: Success
    size_freed: 314572800
    error: null
    timestamp: "2025-11-21T15:28:07Z"

summary:
  total_operations: 2
  successful: 2
  failed: 0
  skipped: 2
  space_freed: 838860800
  duration_seconds: 2.34
```

**What's in the log?**
- Unique execution ID for tracking
- Exact timestamp and duration
- Every operation performed
- Success/failure status for each file
- Total space freed
- Any errors encountered

**Why are transaction logs important?**
- Audit trail for accountability
- Debugging failed operations
- Verification of what was deleted
- Historical record for compliance

---

## Common Workflows

### Workflow 1: Clean Build Artifacts from All Projects

```bash
# Scan
megamaid scan ~/projects --output build-cleanup.yaml

# Review (all build artifacts are already set to 'delete')
megamaid stats build-cleanup.yaml

# Verify
megamaid verify build-cleanup.yaml

# Execute (recycle bin for safety)
megamaid execute build-cleanup.yaml --recycle-bin
```

### Workflow 2: Find and Review Large Files

```bash
# Scan with lower threshold to catch more files
megamaid scan ~/projects --large-file-threshold 50 --output large-files.yaml

# Review the plan and manually change actions
vim large-files.yaml

# Only delete what you marked as 'delete'
megamaid execute large-files.yaml --backup-dir ./large-file-backups
```

### Workflow 3: Careful Cleanup with Multiple Safety Layers

```bash
# Scan
megamaid scan ~/projects

# Review and edit
vim cleanup-plan.yaml

# Dry run first
megamaid execute cleanup-plan.yaml --dry-run

# Verify again
megamaid verify cleanup-plan.yaml

# Execute with backup
megamaid execute cleanup-plan.yaml --backup-dir ./safety-backup

# Verify your system still works, then delete backup
rm -rf ./safety-backup
```

### Workflow 4: Automated Cleanup (Advanced)

```bash
# For automated scripts (be very careful!)
megamaid scan ~/projects --output auto-cleanup.yaml
megamaid verify auto-cleanup.yaml --fail-fast
megamaid execute auto-cleanup.yaml --skip-verify --log-file "auto-$(date +%Y%m%d).yaml"
```

---

## Safety Best Practices

### ✅ DO:
- **Always run dry-run first** before real execution
- **Review the plan manually** - look at every entry
- **Use backup mode** for your first few cleanups
- **Verify before executing** to catch drift
- **Keep transaction logs** for audit trails
- **Test on a small directory** before cleaning large areas
- **Check your system** after cleanup (builds still work, etc.)

### ❌ DON'T:
- Execute without reviewing the plan first
- Skip verification (--skip-verify) unless necessary
- Delete files from critical system directories
- Trust the plan blindly - always review
- Ignore drift warnings - they indicate changes
- Delete backups immediately - verify first

---

## Troubleshooting

### Problem: "Drift detected - cannot execute"

**Cause:** Files have changed since the plan was created.

**Solution:**
```bash
# Re-scan to create a fresh plan
megamaid scan ~/projects --output cleanup-plan-new.yaml

# Or review the drift report
megamaid verify cleanup-plan.yaml --output drift-report.txt
cat drift-report.txt
```

### Problem: "Permission denied" during execution

**Cause:** Insufficient permissions to delete files.

**Solution:**
```bash
# Check file permissions
ls -la /path/to/file

# Run with elevated privileges if appropriate (be careful!)
sudo megamaid execute cleanup-plan.yaml

# Or use backup mode to identify problem files
megamaid execute cleanup-plan.yaml --backup-dir ./test-backup
```

### Problem: Accidentally deleted important files

**Solution 1: Restore from backup**
```bash
# If you used --backup-dir
cp -r ./backups/* ~/projects/

# Or selectively restore
cp -r ./backups/important-project ~/projects/
```

**Solution 2: Restore from recycle bin**
```bash
# If you used --recycle-bin
# On Windows: Open Recycle Bin and restore
# On Linux: Check ~/.local/share/Trash/
```

**Solution 3: Check transaction log**
```bash
# See exactly what was deleted
cat execution-log.yaml

# Find specific files
grep "path:" execution-log.yaml
```

### Problem: Plan has too many entries

**Solution:**
```bash
# Limit scan depth
megamaid scan ~/projects --max-depth 3

# Increase large file threshold
megamaid scan ~/projects --large-file-threshold 500

# Or manually edit the YAML to remove entries
```

---

## What's Next?

Now that you've completed your first cleanup, explore more features:

1. **Read the full README** - [README.md](../README.md) for all options
2. **Check the architecture** - [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
3. **Customize detection** - Edit plans to match your workflow
4. **Automate cleanups** - Create scripts for regular maintenance
5. **Give feedback** - Report issues or suggest features

---

## Quick Reference

### Essential Commands

```bash
# Scan
megamaid scan <directory>

# View statistics
megamaid stats <plan-file>

# Verify
megamaid verify <plan-file>

# Dry run
megamaid execute <plan-file> --dry-run

# Execute (safest options)
megamaid execute <plan-file> --backup-dir ./backups
megamaid execute <plan-file> --recycle-bin
megamaid execute <plan-file> --interactive
```

### Plan Actions

- `action: delete` - Will be deleted
- `action: review` - Flagged for review (won't be deleted)
- `action: keep` - Will be preserved

### Safety Checklist

- [ ] Reviewed the plan manually
- [ ] Ran dry-run and checked output
- [ ] Verified no drift detected
- [ ] Chose appropriate execution mode
- [ ] Have backups or using backup mode
- [ ] Ready to execute

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
cargo run -- scan . --output debug-plan.yaml
```

### Debug with Options

```bash
# Show help
cargo run -- --help
cargo run -- scan --help

# Scan with all options
cargo run -- scan . \
  --output test.yaml \
  --large-file-threshold 50 \
  --max-depth 3 \
  --skip-hidden

# View plan statistics
cargo run -- stats cleanup-plan.yaml

# Verify a plan
cargo run -- verify cleanup-plan.yaml

# Execute with dry-run
cargo run -- execute cleanup-plan.yaml --dry-run
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
            "args": ["scan", ".", "--output", "test-plan.yaml"],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Megamaid Stats",
            "cargo": {
                "args": ["build", "--bin=megamaid"]
            },
            "args": ["stats", "cleanup-plan.yaml"],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Megamaid Execute",
            "cargo": {
                "args": ["build", "--bin=megamaid"]
            },
            "args": ["execute", "cleanup-plan.yaml", "--dry-run"],
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

## Common Development Workflows

### Quick Development Cycle

```bash
# 1. Make code changes
# 2. Run tests
cargo test

# 3. Check with clippy
cargo clippy

# 4. Test manually
cargo run -- scan . --output test.yaml

# 5. View results
cargo run -- stats test.yaml

# 6. Test execution (dry-run)
cargo run -- execute test.yaml --dry-run
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
cargo run -- scan /tmp/test-megamaid --output test.yaml

# View results
cargo run -- stats test.yaml
cat test.yaml

# Test dry-run execution
cargo run -- execute test.yaml --dry-run

# Clean up
rm -rf /tmp/test-megamaid test.yaml
```

### Testing with Your Own Project

```bash
# Scan the megamaid project itself
cargo run -- scan . --output self-scan.yaml

# See what build artifacts it finds
cargo run -- stats self-scan.yaml

# Should detect target/, node_modules/ (if present), etc.

# Test execution (dry-run only, don't actually delete!)
cargo run -- execute self-scan.yaml --dry-run
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

## Developer Quick Reference

### Most Common Commands

```bash
# Development
cargo run -- scan .                    # Quick test scan
cargo run -- execute plan.yaml --dry-run  # Test execution
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
│   ├── scanner/           # Directory traversal
│   ├── detector/          # Cleanup detection
│   ├── planner/           # Plan generation
│   ├── verifier/          # Drift detection
│   └── executor/          # Safe deletion
├── tests/                  # Integration tests
├── target/
│   ├── debug/             # Debug builds
│   │   └── megamaid       # Debug binary
│   └── release/           # Release builds
│       └── megamaid       # Release binary
├── cleanup-plan.yaml      # Default output (gitignored)
└── Cargo.toml             # Project configuration
```

---

## Additional Resources

- **Full Documentation**: [README.md](../README.md) for complete feature list
- **Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md) for design details
- **Development Guide**: [CLAUDE.md](../CLAUDE.md) for contribution guidelines
- **Milestone Status**: [MILESTONE_1_COMPLETE.md](MILESTONE_1_COMPLETE.md) for progress

---

## Getting Help

- **Documentation**: See `docs/` directory for guides
- **Issues**: Report bugs at [GitHub Issues](https://github.com/yourusername/megamaid/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/yourusername/megamaid/discussions)
- **Tests**: Run `cargo test -- --nocapture` to see usage examples

---

Happy cleaning! 🧹✨
