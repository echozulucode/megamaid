# Milestone 2: Plan Verification & Execution - Phased Implementation Plan

## 🎯 STATUS: READY TO START

**Target Start Date**: November 22, 2025
**Estimated Duration**: 3-4 weeks
**Prerequisites**: ✅ Milestone 1 Complete (87 tests passing)

## Overview

**Goal**: Implement safe plan verification and execution with comprehensive drift detection, transaction logging, and multiple execution modes.

**Success Criteria**:
- ✅ Verify plan files against current filesystem state (drift detection)
- ✅ Execute deletions with dry-run, interactive, and batch modes
- ✅ Transaction logging with rollback capability for failed operations
- ✅ Recycle Bin integration for Windows (optional recovery)
- ✅ All components have >85% test coverage
- ✅ Performance: verify 100K entries in <10 seconds, execute 10K deletions in <60 seconds

## Key Design Decisions

### YAML Format
All plan files use YAML format (migrated from TOML in Milestone 1):
- 60% smaller file size
- Faster parsing
- Better readability for large plans
- Example: `cleanup-plan.yaml`

### Drift Detection Strategy
Before execution, verify each entry:
1. **File existence check**: Does the file still exist at the expected path?
2. **Size verification**: Does current size match recorded size?
3. **Modification time check**: Has mtime changed since plan creation?
4. **File ID check (NTFS-only)**: Has the file been moved/renamed? (Optional, future enhancement)

If drift detected → **halt execution** and generate drift report.

### Execution Modes

1. **Dry-run mode** (`--dry-run`): Simulate execution without actually deleting
   - Verify all files can be deleted (permissions, locks)
   - Report what would be deleted
   - Estimated space savings

2. **Interactive mode** (`--interactive`): Prompt for confirmation on each deletion
   - Show file path, size, and reason
   - User can skip, delete, or abort

3. **Batch mode** (default): Execute all Delete actions automatically
   - Progress bar with file count
   - Stop on first error or continue with `--force`

4. **Backup mode** (`--backup-dir <path>`): Move instead of delete
   - Preserve directory structure
   - Allow recovery if needed

### Transaction Logging

Every execution creates a transaction log (`execution-log.yaml`):
```yaml
version: "0.1.0"
execution_id: "uuid-v4-here"
plan_file: "cleanup-plan.yaml"
started_at: "2025-11-22T10:00:00Z"
completed_at: "2025-11-22T10:05:00Z"
status: completed  # completed, failed, aborted
mode: batch
options:
  dry_run: false
  backup_dir: null

operations:
  - path: "target/debug"
    action: delete
    status: success
    size_freed: 1048576
    timestamp: "2025-11-22T10:00:01Z"

  - path: "node_modules"
    action: delete
    status: failed
    error: "Permission denied"
    timestamp: "2025-11-22T10:00:02Z"

summary:
  total_operations: 15
  successful: 14
  failed: 1
  skipped: 0
  space_freed: 1073741824  # bytes
  duration_seconds: 300
```

### Error Handling

- **Locked files**: Skip with warning, continue unless `--fail-fast`
- **Permission errors**: Skip with warning, log for manual review
- **Partial failures**: Complete what we can, report failures in log
- **Rollback**: If `--atomic` flag set, restore from backup on any failure

---

## Phase 2.1: Plan Verification & Drift Detection (Week 1)

### Objectives
Implement comprehensive plan verification to detect filesystem changes since plan creation.

### Implementation Tasks

#### 2.1.1: Verification Engine Core

Create the verification engine that checks plan entries against current filesystem.

```rust
// src/verifier/engine.rs
use crate::models::{CleanupPlan, CleanupEntry};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct VerificationEngine {
    config: VerificationConfig,
}

pub struct VerificationConfig {
    /// Check modification times
    pub check_mtime: bool,
    /// Check file sizes
    pub check_size: bool,
    /// Fail fast on first drift
    pub fail_fast: bool,
    /// Optional: check NTFS file IDs (Windows only)
    #[cfg(windows)]
    pub check_file_id: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            check_mtime: true,
            check_size: true,
            fail_fast: false,
            #[cfg(windows)]
            check_file_id: false,
        }
    }
}

pub struct VerificationResult {
    pub total_entries: usize,
    pub verified: usize,
    pub drifted: Vec<DriftDetection>,
    pub missing: Vec<PathBuf>,
    pub permission_errors: Vec<PathBuf>,
}

impl VerificationResult {
    pub fn has_drift(&self) -> bool {
        !self.drifted.is_empty() || !self.missing.is_empty()
    }

    pub fn is_safe_to_execute(&self) -> bool {
        // Safe if no drift and no missing files
        // Permission errors are warnings, not blockers
        !self.has_drift()
    }
}

#[derive(Debug, Clone)]
pub struct DriftDetection {
    pub path: PathBuf,
    pub drift_type: DriftType,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DriftType {
    SizeMismatch,
    ModificationTimeMismatch,
    FileIdMismatch,  // NTFS only
}

impl VerificationEngine {
    pub fn new(config: VerificationConfig) -> Self {
        Self { config }
    }

    pub fn verify(&self, plan: &CleanupPlan) -> Result<VerificationResult, VerificationError> {
        let mut result = VerificationResult {
            total_entries: plan.entries.len(),
            verified: 0,
            drifted: Vec::new(),
            missing: Vec::new(),
            permission_errors: Vec::new(),
        };

        for entry in &plan.entries {
            // Skip entries marked as "keep" - we're not going to touch them
            if entry.action == crate::models::CleanupAction::Keep {
                result.verified += 1;
                continue;
            }

            let full_path = plan.base_path.join(&entry.path);

            // Check 1: Does file exist?
            if !full_path.exists() {
                result.missing.push(full_path.clone());
                if self.config.fail_fast {
                    return Ok(result);
                }
                continue;
            }

            // Check 2: Can we read metadata?
            let metadata = match std::fs::metadata(&full_path) {
                Ok(m) => m,
                Err(_) => {
                    result.permission_errors.push(full_path.clone());
                    continue;
                }
            };

            // Check 3: Size verification
            if self.config.check_size {
                let current_size = if metadata.is_dir() {
                    // For directories, we need to calculate recursive size
                    self.calculate_dir_size(&full_path)?
                } else {
                    metadata.len()
                };

                if current_size != entry.size {
                    result.drifted.push(DriftDetection {
                        path: full_path.clone(),
                        drift_type: DriftType::SizeMismatch,
                        expected: format!("{} bytes", entry.size),
                        actual: format!("{} bytes", current_size),
                    });
                    if self.config.fail_fast {
                        return Ok(result);
                    }
                    continue;
                }
            }

            // Check 4: Modification time verification
            if self.config.check_mtime {
                let current_mtime = metadata.modified()?;
                let expected_mtime = chrono::DateTime::parse_from_rfc3339(&entry.modified)
                    .map_err(|e| VerificationError::InvalidTimestamp(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let expected_systime: SystemTime = expected_mtime.into();

                // Allow small time differences (filesystem timestamp precision)
                let time_diff = match current_mtime.duration_since(expected_systime) {
                    Ok(d) => d,
                    Err(e) => e.duration(),
                };

                if time_diff.as_secs() > 2 {
                    result.drifted.push(DriftDetection {
                        path: full_path.clone(),
                        drift_type: DriftType::ModificationTimeMismatch,
                        expected: entry.modified.clone(),
                        actual: chrono::DateTime::<chrono::Utc>::from(current_mtime).to_rfc3339(),
                    });
                    if self.config.fail_fast {
                        return Ok(result);
                    }
                    continue;
                }
            }

            // All checks passed
            result.verified += 1;
        }

        Ok(result)
    }

    fn calculate_dir_size(&self, dir_path: &Path) -> Result<u64, VerificationError> {
        use walkdir::WalkDir;
        let mut total_size = 0u64;

        for entry in WalkDir::new(dir_path).follow_links(false) {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_size = total_size.saturating_add(metadata.len());
            }
        }

        Ok(total_size)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid timestamp in plan: {0}")]
    InvalidTimestamp(String),

    #[error("Walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}
```

#### 2.1.2: Drift Report Generator

Generate human-readable drift reports.

```rust
// src/verifier/report.rs
use crate::verifier::engine::{VerificationResult, DriftType};
use std::io::Write;

pub struct DriftReporter;

impl DriftReporter {
    pub fn generate_report(result: &VerificationResult) -> String {
        let mut report = String::new();

        report.push_str("# Plan Verification Report\n\n");

        // Summary
        report.push_str(&format!("Total entries: {}\n", result.total_entries));
        report.push_str(&format!("Verified: {}\n", result.verified));
        report.push_str(&format!("Drifted: {}\n", result.drifted.len()));
        report.push_str(&format!("Missing: {}\n", result.missing.len()));
        report.push_str(&format!("Permission errors: {}\n\n", result.permission_errors.len()));

        if result.is_safe_to_execute() {
            report.push_str("✅ SAFE TO EXECUTE\n\n");
        } else {
            report.push_str("⚠️  DRIFT DETECTED - NOT SAFE TO EXECUTE\n\n");
        }

        // Missing files
        if !result.missing.is_empty() {
            report.push_str("## Missing Files\n\n");
            for path in &result.missing {
                report.push_str(&format!("- {}\n", path.display()));
            }
            report.push_str("\n");
        }

        // Drifted files
        if !result.drifted.is_empty() {
            report.push_str("## Drifted Files\n\n");
            for drift in &result.drifted {
                report.push_str(&format!("### {}\n", drift.path.display()));
                report.push_str(&format!("Type: {:?}\n", drift.drift_type));
                report.push_str(&format!("Expected: {}\n", drift.expected));
                report.push_str(&format!("Actual: {}\n\n", drift.actual));
            }
        }

        // Permission errors (warnings)
        if !result.permission_errors.is_empty() {
            report.push_str("## Permission Warnings\n\n");
            report.push_str("The following files could not be verified due to permission errors:\n\n");
            for path in &result.permission_errors {
                report.push_str(&format!("- {}\n", path.display()));
            }
            report.push_str("\n");
        }

        report
    }

    pub fn write_report(result: &VerificationResult, path: &std::path::Path) -> std::io::Result<()> {
        let report = Self::generate_report(result);
        let mut file = std::fs::File::create(path)?;
        file.write_all(report.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}
```

### Unit Tests

**Test Suite: `tests/unit/verifier/engine_tests.rs`**
```rust
#[cfg(test)]
mod engine_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_verify_valid_plan() {
        let temp = TempDir::new().unwrap();
        let plan = create_test_plan_with_file(&temp, "test.txt", 100);

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.verified, plan.entries.len());
        assert!(!result.has_drift());
        assert!(result.is_safe_to_execute());
    }

    #[test]
    fn test_detect_missing_file() {
        let temp = TempDir::new().unwrap();
        let plan = create_test_plan_with_file(&temp, "deleted.txt", 100);

        // Delete the file after plan creation
        std::fs::remove_file(temp.path().join("deleted.txt")).unwrap();

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.missing.len(), 1);
        assert!(result.has_drift());
        assert!(!result.is_safe_to_execute());
    }

    #[test]
    fn test_detect_size_mismatch() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("modified.txt");

        // Create file
        std::fs::write(&file_path, "original").unwrap();
        let plan = create_plan_from_path(&temp, &file_path);

        // Modify file (change size)
        std::fs::write(&file_path, "modified content is longer").unwrap();

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.drifted.len(), 1);
        assert_eq!(result.drifted[0].drift_type, DriftType::SizeMismatch);
        assert!(!result.is_safe_to_execute());
    }

    #[test]
    fn test_detect_mtime_change() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("touched.txt");

        std::fs::write(&file_path, "content").unwrap();
        let plan = create_plan_from_path(&temp, &file_path);

        // Wait and touch file
        std::thread::sleep(std::time::Duration::from_secs(3));
        std::fs::write(&file_path, "content").unwrap(); // Same content, different mtime

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert!(result.drifted.len() > 0);
        assert!(!result.is_safe_to_execute());
    }

    #[test]
    fn test_skip_keep_actions() {
        let temp = TempDir::new().unwrap();
        let mut plan = create_test_plan_with_file(&temp, "keep.txt", 100);
        plan.entries[0].action = crate::models::CleanupAction::Keep;

        // Delete the file - verification should still pass because action is Keep
        std::fs::remove_file(temp.path().join("keep.txt")).unwrap();

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.verified, 1);
        assert!(!result.has_drift());
    }

    #[test]
    fn test_fail_fast_mode() {
        let temp = TempDir::new().unwrap();
        let mut plan = create_test_plan(&temp);

        // Add multiple entries, make first one missing
        add_plan_entry(&mut plan, "missing.txt");
        add_plan_entry(&mut plan, "also_missing.txt");

        let config = VerificationConfig {
            fail_fast: true,
            ..Default::default()
        };

        let verifier = VerificationEngine::new(config);
        let result = verifier.verify(&plan).unwrap();

        // Should stop after first missing file
        assert_eq!(result.missing.len(), 1);
    }

    #[test]
    fn test_directory_size_verification() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("test_dir");
        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(dir_path.join("file1.txt"), "a".repeat(100)).unwrap();
        std::fs::write(dir_path.join("file2.txt"), "b".repeat(200)).unwrap();

        let plan = create_plan_from_path(&temp, &dir_path);

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.verified, 1);
        assert!(!result.has_drift());
    }
}
```

### Acceptance Criteria
- [ ] Verification engine detects all drift types (missing, size, mtime)
- [ ] Fail-fast mode stops on first drift
- [ ] Keep actions are skipped during verification
- [ ] Permission errors reported as warnings, not blockers
- [ ] Directory sizes calculated recursively
- [ ] All unit tests pass with >90% coverage
- [ ] Verification completes in <10 seconds for 100K entries

---

## Phase 2.2: Deletion Engine & Safety (Week 2)

### Objectives
Implement safe deletion with multiple execution modes and comprehensive error handling.

### Implementation Tasks

#### 2.2.1: Deletion Engine Core

```rust
// src/executor/engine.rs
use crate::models::{CleanupPlan, CleanupEntry, CleanupAction};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct ExecutionEngine {
    config: ExecutionConfig,
}

pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub backup_dir: Option<PathBuf>,
    pub fail_fast: bool,
    pub use_recycle_bin: bool,  // Windows only
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    DryRun,
    Interactive,
    Batch,
}

pub struct ExecutionResult {
    pub operations: Vec<OperationResult>,
    pub summary: ExecutionSummary,
}

pub struct ExecutionSummary {
    pub total_operations: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub space_freed: u64,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub path: PathBuf,
    pub action: OperationAction,
    pub status: OperationStatus,
    pub size_freed: Option<u64>,
    pub error: Option<String>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationAction {
    Delete,
    MoveToBackup,
    MoveToRecycleBin,
    Skip,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Success,
    Failed,
    Skipped,
    DryRun,
}

impl ExecutionEngine {
    pub fn new(config: ExecutionConfig) -> Self {
        Self { config }
    }

    pub fn execute(&self, plan: &CleanupPlan) -> Result<ExecutionResult, ExecutionError> {
        let start_time = std::time::Instant::now();
        let mut operations = Vec::new();

        for entry in &plan.entries {
            // Only process Delete actions
            if entry.action != CleanupAction::Delete {
                continue;
            }

            let full_path = plan.base_path.join(&entry.path);

            // Interactive mode: prompt user
            if self.config.mode == ExecutionMode::Interactive {
                if !self.prompt_user(entry)? {
                    operations.push(OperationResult {
                        path: full_path,
                        action: OperationAction::Skip,
                        status: OperationStatus::Skipped,
                        size_freed: None,
                        error: None,
                        timestamp: SystemTime::now(),
                    });
                    continue;
                }
            }

            // Execute operation
            let result = self.execute_single(&full_path, entry);

            // Fail-fast check
            if self.config.fail_fast && result.status == OperationStatus::Failed {
                operations.push(result);
                break;
            }

            operations.push(result);
        }

        let duration = start_time.elapsed();
        let summary = self.compute_summary(&operations, duration);

        Ok(ExecutionResult { operations, summary })
    }

    fn execute_single(&self, path: &Path, entry: &CleanupEntry) -> OperationResult {
        let timestamp = SystemTime::now();

        // Dry-run mode
        if self.config.mode == ExecutionMode::DryRun {
            return OperationResult {
                path: path.to_path_buf(),
                action: OperationAction::Delete,
                status: OperationStatus::DryRun,
                size_freed: Some(entry.size),
                error: None,
                timestamp,
            };
        }

        // Determine action type
        let action = if self.config.use_recycle_bin {
            OperationAction::MoveToRecycleBin
        } else if let Some(ref backup_dir) = self.config.backup_dir {
            OperationAction::MoveToBackup
        } else {
            OperationAction::Delete
        };

        // Execute the operation
        let result = match action {
            OperationAction::Delete => self.delete_path(path),
            OperationAction::MoveToBackup => self.move_to_backup(path, entry),
            OperationAction::MoveToRecycleBin => self.move_to_recycle_bin(path),
            OperationAction::Skip => Ok(()),
        };

        match result {
            Ok(()) => OperationResult {
                path: path.to_path_buf(),
                action,
                status: OperationStatus::Success,
                size_freed: Some(entry.size),
                error: None,
                timestamp,
            },
            Err(e) => OperationResult {
                path: path.to_path_buf(),
                action,
                status: OperationStatus::Failed,
                size_freed: None,
                error: Some(e.to_string()),
                timestamp,
            },
        }
    }

    fn delete_path(&self, path: &Path) -> Result<(), std::io::Error> {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }

    fn move_to_backup(&self, path: &Path, entry: &CleanupEntry) -> Result<(), std::io::Error> {
        let backup_dir = self.config.backup_dir.as_ref().unwrap();
        let dest = backup_dir.join(&entry.path);

        // Create parent directories
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Move the file/directory
        std::fs::rename(path, &dest)?;
        Ok(())
    }

    #[cfg(windows)]
    fn move_to_recycle_bin(&self, path: &Path) -> Result<(), std::io::Error> {
        // Use Windows Shell API to move to Recycle Bin
        // This requires the `windows` crate
        use windows::Win32::UI::Shell::{SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, SHFILEOPSTRUCTW, FO_DELETE};
        use windows::core::PWSTR;

        let mut path_wide: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))  // Null terminator
            .chain(std::iter::once(0))  // Double null for SHFILEOPSTRUCTW
            .collect();

        let mut file_op = SHFILEOPSTRUCTW {
            hwnd: Default::default(),
            wFunc: FO_DELETE,
            pFrom: PWSTR(path_wide.as_mut_ptr()),
            pTo: PWSTR::null(),
            fFlags: FOF_ALLOWUNDO | FOF_NOCONFIRMATION,
            fAnyOperationsAborted: Default::default(),
            hNameMappings: Default::default(),
            lpszProgressTitle: PWSTR::null(),
        };

        unsafe {
            let result = SHFileOperationW(&mut file_op);
            if result.0 != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("SHFileOperation failed with code: {}", result.0),
                ));
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    fn move_to_recycle_bin(&self, path: &Path) -> Result<(), std::io::Error> {
        // On non-Windows platforms, use trash crate
        trash::delete(path).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })
    }

    fn prompt_user(&self, entry: &CleanupEntry) -> Result<bool, ExecutionError> {
        use std::io::{self, Write};

        println!("\n{}", "=".repeat(60));
        println!("Path: {}", entry.path);
        println!("Size: {:.2} MB", entry.size as f64 / 1_048_576.0);
        println!("Reason: {}", entry.reason);
        println!("{}", "=".repeat(60));
        print!("Delete this file/directory? [y/n/a/q]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            "a" | "abort" | "q" | "quit" => Err(ExecutionError::UserAborted),
            _ => {
                println!("Invalid input. Skipping.");
                Ok(false)
            }
        }
    }

    fn compute_summary(&self, operations: &[OperationResult], duration: std::time::Duration) -> ExecutionSummary {
        let total_operations = operations.len();
        let successful = operations.iter().filter(|o| o.status == OperationStatus::Success || o.status == OperationStatus::DryRun).count();
        let failed = operations.iter().filter(|o| o.status == OperationStatus::Failed).count();
        let skipped = operations.iter().filter(|o| o.status == OperationStatus::Skipped).count();
        let space_freed = operations.iter()
            .filter_map(|o| o.size_freed)
            .sum();

        ExecutionSummary {
            total_operations,
            successful,
            failed,
            skipped,
            space_freed,
            duration,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("User aborted execution")]
    UserAborted,
}
```

### Unit Tests

**Test Suite: `tests/unit/executor/engine_tests.rs`**
```rust
#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn test_dry_run_mode() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        let plan = create_plan_with_delete_action(&temp, "test.txt");

        let config = ExecutionConfig {
            mode: ExecutionMode::DryRun,
            backup_dir: None,
            fail_fast: false,
            use_recycle_bin: false,
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.total_operations, 1);
        assert_eq!(result.operations[0].status, OperationStatus::DryRun);
        assert!(file_path.exists(), "File should still exist in dry-run mode");
    }

    #[test]
    fn test_batch_delete_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("delete_me.txt");
        std::fs::write(&file_path, "content").unwrap();

        let plan = create_plan_with_delete_action(&temp, "delete_me.txt");

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: None,
            fail_fast: false,
            use_recycle_bin: false,
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 1);
        assert!(!file_path.exists(), "File should be deleted");
    }

    #[test]
    fn test_batch_delete_directory() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("delete_dir");
        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(dir_path.join("file.txt"), "content").unwrap();

        let plan = create_plan_with_delete_action(&temp, "delete_dir");

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: None,
            fail_fast: false,
            use_recycle_bin: false,
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 1);
        assert!(!dir_path.exists(), "Directory should be deleted");
    }

    #[test]
    fn test_backup_mode() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backups");
        std::fs::create_dir(&backup_dir).unwrap();

        let file_path = temp.path().join("move_me.txt");
        std::fs::write(&file_path, "content").unwrap();

        let plan = create_plan_with_delete_action(&temp, "move_me.txt");

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: Some(backup_dir.clone()),
            fail_fast: false,
            use_recycle_bin: false,
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 1);
        assert!(!file_path.exists(), "Original file should be gone");
        assert!(backup_dir.join("move_me.txt").exists(), "File should be in backup");
    }

    #[test]
    fn test_fail_fast_on_error() {
        let temp = TempDir::new().unwrap();

        let mut plan = create_test_plan(&temp);
        // Add entry for nonexistent file
        add_delete_entry(&mut plan, "nonexistent.txt");
        add_delete_entry(&mut plan, "also_should_not_run.txt");

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: None,
            fail_fast: true,
            use_recycle_bin: false,
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        // Should stop after first failure
        assert_eq!(result.summary.failed, 1);
        assert_eq!(result.operations.len(), 1);
    }

    #[test]
    fn test_skip_keep_actions() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("keep_me.txt");
        std::fs::write(&file_path, "content").unwrap();

        let mut plan = create_test_plan(&temp);
        add_entry_with_action(&mut plan, "keep_me.txt", CleanupAction::Keep);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: None,
            fail_fast: false,
            use_recycle_bin: false,
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.total_operations, 0);
        assert!(file_path.exists(), "File with Keep action should not be deleted");
    }

    #[test]
    fn test_execution_summary_accuracy() {
        let temp = TempDir::new().unwrap();

        // Create multiple files
        for i in 0..5 {
            std::fs::write(temp.path().join(format!("file{}.txt", i)), "x".repeat(1000)).unwrap();
        }

        let mut plan = create_test_plan(&temp);
        for i in 0..5 {
            add_delete_entry(&mut plan, &format!("file{}.txt", i));
        }

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: None,
            fail_fast: false,
            use_recycle_bin: false,
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 5);
        assert_eq!(result.summary.space_freed, 5000);
    }
}
```

### Acceptance Criteria
- [ ] Dry-run mode works without modifying filesystem
- [ ] Batch mode deletes all Delete actions
- [ ] Backup mode preserves directory structure
- [ ] Fail-fast stops on first error
- [ ] Keep and Review actions are skipped
- [ ] Summary statistics are accurate
- [ ] All unit tests pass with >85% coverage

---

## Phase 2.3: Transaction Logging & Rollback (Week 2-3)

### Objectives
Implement comprehensive transaction logging for audit trail and potential rollback.

### Implementation Tasks

#### 2.3.1: Transaction Logger

```rust
// src/executor/transaction.rs
use crate::executor::engine::{ExecutionResult, OperationResult};
use crate::models::CleanupPlan;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionLog {
    pub version: String,
    pub execution_id: String,
    pub plan_file: PathBuf,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: TransactionStatus,
    pub mode: String,
    pub options: TransactionOptions,
    pub operations: Vec<LoggedOperation>,
    pub summary: Option<ExecutionSummaryLog>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    InProgress,
    Completed,
    Failed,
    Aborted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionOptions {
    pub dry_run: bool,
    pub backup_dir: Option<PathBuf>,
    pub use_recycle_bin: bool,
    pub fail_fast: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggedOperation {
    pub path: String,
    pub action: String,
    pub status: String,
    pub size_freed: Option<u64>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionSummaryLog {
    pub total_operations: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub space_freed: u64,
    pub duration_seconds: f64,
}

pub struct TransactionLogger {
    log_path: PathBuf,
    log: TransactionLog,
}

impl TransactionLogger {
    pub fn new(plan_file: &Path, log_path: PathBuf, options: TransactionOptions) -> Self {
        let log = TransactionLog {
            version: env!("CARGO_PKG_VERSION").to_string(),
            execution_id: Uuid::new_v4().to_string(),
            plan_file: plan_file.to_path_buf(),
            started_at: Utc::now(),
            completed_at: None,
            status: TransactionStatus::InProgress,
            mode: if options.dry_run {
                "dry_run".to_string()
            } else {
                "batch".to_string()
            },
            options,
            operations: Vec::new(),
            summary: None,
        };

        Self { log_path, log }
    }

    pub fn log_operation(&mut self, operation: &OperationResult) {
        self.log.operations.push(LoggedOperation {
            path: operation.path.to_string_lossy().to_string(),
            action: format!("{:?}", operation.action),
            status: format!("{:?}", operation.status),
            size_freed: operation.size_freed,
            error: operation.error.clone(),
            timestamp: operation.timestamp.into(),
        });
    }

    pub fn finalize(&mut self, result: &ExecutionResult, status: TransactionStatus) -> std::io::Result<()> {
        self.log.completed_at = Some(Utc::now());
        self.log.status = status;
        self.log.summary = Some(ExecutionSummaryLog {
            total_operations: result.summary.total_operations,
            successful: result.summary.successful,
            failed: result.summary.failed,
            skipped: result.summary.skipped,
            space_freed: result.summary.space_freed,
            duration_seconds: result.summary.duration.as_secs_f64(),
        });

        self.write()
    }

    pub fn write(&self) -> std::io::Result<()> {
        let yaml_content = serde_yaml::to_string(&self.log)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Atomic write
        let temp_path = self.log_path.with_extension("tmp");
        std::fs::write(&temp_path, yaml_content)?;
        std::fs::rename(temp_path, &self.log_path)?;

        Ok(())
    }

    pub fn read(path: &Path) -> std::io::Result<TransactionLog> {
        let content = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
```

### Unit Tests

**Test Suite: `tests/unit/executor/transaction_tests.rs`**
```rust
#[cfg(test)]
mod transaction_tests {
    use super::*;

    #[test]
    fn test_create_transaction_log() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");
        let plan_path = temp.path().join("plan.yaml");

        let options = TransactionOptions {
            dry_run: false,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let logger = TransactionLogger::new(&plan_path, log_path.clone(), options);
        logger.write().unwrap();

        assert!(log_path.exists());
    }

    #[test]
    fn test_log_operations() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");

        let mut logger = create_test_logger(&log_path);

        let op = create_test_operation("test.txt", OperationStatus::Success, Some(1000));
        logger.log_operation(&op);

        assert_eq!(logger.log.operations.len(), 1);
        assert_eq!(logger.log.operations[0].path, "test.txt");
    }

    #[test]
    fn test_finalize_transaction() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");

        let mut logger = create_test_logger(&log_path);
        let result = create_test_execution_result();

        logger.finalize(&result, TransactionStatus::Completed).unwrap();

        assert_eq!(logger.log.status, TransactionStatus::Completed);
        assert!(logger.log.completed_at.is_some());
        assert!(logger.log.summary.is_some());
    }

    #[test]
    fn test_read_transaction_log() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");

        let logger = create_test_logger(&log_path);
        logger.write().unwrap();

        let loaded = TransactionLogger::read(&log_path).unwrap();
        assert_eq!(loaded.execution_id, logger.log.execution_id);
        assert_eq!(loaded.version, logger.log.version);
    }

    #[test]
    fn test_transaction_log_serialization() {
        let log = create_test_transaction_log();
        let yaml = serde_yaml::to_string(&log).unwrap();
        let roundtrip: TransactionLog = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(log.execution_id, roundtrip.execution_id);
        assert_eq!(log.status, roundtrip.status);
    }
}
```

### Acceptance Criteria
- [ ] Transaction logs created with unique execution IDs
- [ ] All operations logged with timestamps
- [ ] Logs serialized to YAML successfully
- [ ] Logs can be read back for audit
- [ ] Atomic writes prevent corruption
- [ ] All unit tests pass

---

## Phase 2.4: CLI Commands (Week 3)

### Objectives
Implement user-facing CLI commands for verification and execution.

### Implementation Tasks

#### 2.4.1: Verify Command

```rust
// src/cli/commands.rs (additions)
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    // ... existing Scan command ...

    /// Verify a cleanup plan against current filesystem state
    Verify {
        /// Path to the cleanup plan file
        #[arg(value_name = "PLAN")]
        plan: PathBuf,

        /// Output drift report to file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Fail fast on first drift detection
        #[arg(long)]
        fail_fast: bool,

        /// Skip modification time checks
        #[arg(long)]
        skip_mtime: bool,
    },

    /// Execute a cleanup plan
    Execute {
        /// Path to the cleanup plan file
        #[arg(value_name = "PLAN")]
        plan: PathBuf,

        /// Dry-run mode (simulate without deleting)
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode (prompt for each deletion)
        #[arg(short, long)]
        interactive: bool,

        /// Backup directory (move instead of delete)
        #[arg(long, value_name = "DIR")]
        backup_dir: Option<PathBuf>,

        /// Use system recycle bin (Windows)
        #[arg(long)]
        recycle_bin: bool,

        /// Stop on first error
        #[arg(long)]
        fail_fast: bool,

        /// Skip verification before execution
        #[arg(long)]
        skip_verify: bool,

        /// Transaction log file
        #[arg(long, value_name = "FILE", default_value = "execution-log.yaml")]
        log_file: PathBuf,
    },
}
```

#### 2.4.2: Command Orchestration

```rust
// src/cli/orchestrator.rs (additions)
fn run_verify(
    plan_path: &Path,
    output: Option<PathBuf>,
    fail_fast: bool,
    skip_mtime: bool,
) -> Result<()> {
    println!("📋 Verifying cleanup plan: {}", plan_path.display());
    println!();

    // Load plan
    let content = fs::read_to_string(plan_path)
        .context(format!("Failed to read plan file: {}", plan_path.display()))?;
    let plan: CleanupPlan = serde_yaml::from_str(&content)
        .context("Failed to parse plan file")?;

    // Configure verification
    let config = VerificationConfig {
        check_mtime: !skip_mtime,
        check_size: true,
        fail_fast,
        #[cfg(windows)]
        check_file_id: false,
    };

    // Run verification
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Verifying entries...");

    let verifier = VerificationEngine::new(config);
    let result = verifier.verify(&plan)?;

    spinner.finish_with_message(format!(
        "✓ Verified {} of {} entries",
        result.verified,
        result.total_entries
    ));
    println!();

    // Print report
    let report = DriftReporter::generate_report(&result);
    println!("{}", report);

    // Write report file if requested
    if let Some(output_path) = output {
        DriftReporter::write_report(&result, &output_path)?;
        println!("📄 Drift report written to: {}", output_path.display());
    }

    // Exit with error if drift detected
    if !result.is_safe_to_execute() {
        anyhow::bail!("Drift detected - plan is not safe to execute");
    }

    Ok(())
}

fn run_execute(
    plan_path: &Path,
    dry_run: bool,
    interactive: bool,
    backup_dir: Option<PathBuf>,
    recycle_bin: bool,
    fail_fast: bool,
    skip_verify: bool,
    log_file: PathBuf,
) -> Result<()> {
    println!("🗑️  Executing cleanup plan: {}", plan_path.display());
    println!();

    // Load plan
    let content = fs::read_to_string(plan_path)
        .context(format!("Failed to read plan file: {}", plan_path.display()))?;
    let plan: CleanupPlan = serde_yaml::from_str(&content)
        .context("Failed to parse plan file")?;

    // Verify unless skipped
    if !skip_verify && !dry_run {
        println!("🔍 Verifying plan before execution...");
        let verifier = VerificationEngine::new(VerificationConfig::default());
        let verification = verifier.verify(&plan)?;

        if !verification.is_safe_to_execute() {
            let report = DriftReporter::generate_report(&verification);
            println!("{}", report);
            anyhow::bail!("Drift detected - cannot execute. Use --skip-verify to override.");
        }
        println!("✓ Verification passed\n");
    }

    // Configure execution
    let mode = if dry_run {
        ExecutionMode::DryRun
    } else if interactive {
        ExecutionMode::Interactive
    } else {
        ExecutionMode::Batch
    };

    let config = ExecutionConfig {
        mode,
        backup_dir: backup_dir.clone(),
        fail_fast,
        use_recycle_bin: recycle_bin,
    };

    // Create transaction logger
    let options = TransactionOptions {
        dry_run,
        backup_dir,
        use_recycle_bin: recycle_bin,
        fail_fast,
    };
    let mut logger = TransactionLogger::new(plan_path, log_file.clone(), options);

    // Execute
    let executor = ExecutionEngine::new(config);

    if dry_run {
        println!("🔄 DRY RUN MODE - No files will be deleted");
    }
    println!();

    let progress = ProgressBar::new(plan.entries.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let result = executor.execute(&plan)?;

    // Log all operations
    for operation in &result.operations {
        logger.log_operation(operation);
    }

    // Finalize transaction
    let status = if result.summary.failed > 0 {
        TransactionStatus::Failed
    } else {
        TransactionStatus::Completed
    };
    logger.finalize(&result, status)?;

    progress.finish_with_message("Done");
    println!();

    // Print summary
    print_execution_summary(&result.summary, dry_run);
    println!();
    println!("📋 Transaction log written to: {}", log_file.display());

    Ok(())
}

fn print_execution_summary(summary: &ExecutionSummary, dry_run: bool) {
    println!("Summary:");
    println!("  Total operations: {}", summary.total_operations);
    println!("  Successful: {}", summary.successful);
    println!("  Failed: {}", summary.failed);
    println!("  Skipped: {}", summary.skipped);
    println!("  Space freed: {:.2} GB", summary.space_freed as f64 / 1_073_741_824.0);
    println!("  Duration: {:.2}s", summary.duration.as_secs_f64());

    if dry_run {
        println!();
        println!("This was a dry run. No files were actually deleted.");
    }
}
```

### End-to-End Tests

**Test Suite: `tests/e2e/milestone2_e2e_tests.rs`**
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_verify_command_valid_plan() {
    let temp = TempDir::new().unwrap();
    create_test_structure(&temp);

    // Generate plan
    let plan_path = temp.path().join("plan.yaml");
    generate_plan(&temp, &plan_path);

    // Verify plan
    let mut cmd = Command::cargo_bin("megamaid").unwrap();
    cmd.arg("verify").arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SAFE TO EXECUTE"));
}

#[test]
fn test_verify_command_with_drift() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std::fs::write(&file_path, "original").unwrap();

    // Generate plan
    let plan_path = temp.path().join("plan.yaml");
    generate_plan(&temp, &plan_path);

    // Modify file (introduce drift)
    std::fs::write(&file_path, "modified").unwrap();

    // Verify should detect drift
    let mut cmd = Command::cargo_bin("megamaid").unwrap();
    cmd.arg("verify").arg(&plan_path);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("DRIFT DETECTED"));
}

#[test]
fn test_execute_dry_run() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("delete_me.txt");
    std::fs::write(&file_path, "content").unwrap();

    let plan_path = temp.path().join("plan.yaml");
    generate_delete_plan(&temp, &plan_path, "delete_me.txt");

    // Execute dry-run
    let mut cmd = Command::cargo_bin("megamaid").unwrap();
    cmd.arg("execute")
        .arg(&plan_path)
        .arg("--dry-run");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"))
        .stdout(predicate::str::contains("No files were actually deleted"));

    assert!(file_path.exists(), "File should still exist after dry-run");
}

#[test]
fn test_execute_batch_mode() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("delete_me.txt");
    std::fs::write(&file_path, "content").unwrap();

    let plan_path = temp.path().join("plan.yaml");
    generate_delete_plan(&temp, &plan_path, "delete_me.txt");

    let log_path = temp.path().join("execution.yaml");

    // Execute batch
    let mut cmd = Command::cargo_bin("megamaid").unwrap();
    cmd.arg("execute")
        .arg(&plan_path)
        .arg("--log-file").arg(&log_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successful: 1"));

    assert!(!file_path.exists(), "File should be deleted");
    assert!(log_path.exists(), "Transaction log should exist");
}

#[test]
fn test_execute_with_backup() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backups");
    std::fs::create_dir(&backup_dir).unwrap();

    let file_path = temp.path().join("move_me.txt");
    std::fs::write(&file_path, "content").unwrap();

    let plan_path = temp.path().join("plan.yaml");
    generate_delete_plan(&temp, &plan_path, "move_me.txt");

    // Execute with backup
    let mut cmd = Command::cargo_bin("megamaid").unwrap();
    cmd.arg("execute")
        .arg(&plan_path)
        .arg("--backup-dir").arg(&backup_dir);

    cmd.assert().success();

    assert!(!file_path.exists(), "Original should be gone");
    assert!(backup_dir.join("move_me.txt").exists(), "Backup should exist");
}

#[test]
fn test_execute_skip_verify_with_drift() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std::fs::write(&file_path, "original").unwrap();

    let plan_path = temp.path().join("plan.yaml");
    generate_delete_plan(&temp, &plan_path, "test.txt");

    // Introduce drift
    std::fs::write(&file_path, "modified").unwrap();

    // Execute with skip-verify should succeed
    let mut cmd = Command::cargo_bin("megamaid").unwrap();
    cmd.arg("execute")
        .arg(&plan_path)
        .arg("--skip-verify");

    cmd.assert().success();
    assert!(!file_path.exists());
}
```

### Acceptance Criteria
- [ ] `verify` command detects all types of drift
- [ ] `execute --dry-run` shows what would be deleted without deleting
- [ ] `execute` with no flags performs batch deletion
- [ ] `execute --backup-dir` moves files to backup location
- [ ] `execute --interactive` prompts for each file (manual test)
- [ ] Transaction logs created for all executions
- [ ] All E2E tests pass

---

## Phase 2.5: Testing & Validation (Week 4)

### Objectives
Comprehensive testing, integration validation, and documentation updates.

### Implementation Tasks

#### 2.5.1: Integration Test Matrix

```rust
// tests/integration/milestone2_integration_tests.rs

#[test]
fn test_full_workflow_verify_then_execute() {
    let temp = TempDir::new().unwrap();
    create_realistic_project(&temp);

    // Step 1: Scan
    let plan_path = temp.path().join("plan.yaml");
    scan_and_generate_plan(&temp, &plan_path);

    // Step 2: Verify
    let verifier = VerificationEngine::new(VerificationConfig::default());
    let plan = load_plan(&plan_path).unwrap();
    let verification = verifier.verify(&plan).unwrap();
    assert!(verification.is_safe_to_execute());

    // Step 3: Execute
    let config = ExecutionConfig {
        mode: ExecutionMode::Batch,
        backup_dir: None,
        fail_fast: false,
        use_recycle_bin: false,
    };
    let executor = ExecutionEngine::new(config);
    let result = executor.execute(&plan).unwrap();

    assert!(result.summary.successful > 0);
    assert_eq!(result.summary.failed, 0);
}

#[test]
fn test_verification_prevents_stale_execution() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("stale.txt");
    std::fs::write(&file_path, "original").unwrap();

    let plan = generate_plan_for_file(&temp, "stale.txt");

    // Modify file
    std::fs::write(&file_path, "modified").unwrap();

    // Verification should fail
    let verifier = VerificationEngine::new(VerificationConfig::default());
    let result = verifier.verify(&plan).unwrap();
    assert!(!result.is_safe_to_execute());
}

#[test]
fn test_transaction_log_audit_trail() {
    let temp = TempDir::new().unwrap();
    create_files_for_deletion(&temp, 10);

    let plan = generate_deletion_plan(&temp);
    let log_path = temp.path().join("transaction.yaml");

    // Execute with logging
    let mut logger = create_logger(&plan, &log_path);
    let executor = create_executor();
    let result = executor.execute(&plan).unwrap();

    for op in &result.operations {
        logger.log_operation(op);
    }
    logger.finalize(&result, TransactionStatus::Completed).unwrap();

    // Verify log contains all operations
    let log = TransactionLogger::read(&log_path).unwrap();
    assert_eq!(log.operations.len(), result.operations.len());
    assert_eq!(log.status, TransactionStatus::Completed);
}

#[test]
fn test_backup_mode_preserves_structure() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backups");
    std::fs::create_dir(&backup_dir).unwrap();

    // Create nested structure
    std::fs::create_dir_all(temp.path().join("a/b/c")).unwrap();
    std::fs::write(temp.path().join("a/b/c/file.txt"), "content").unwrap();

    let plan = generate_deletion_plan(&temp);
    let config = ExecutionConfig {
        mode: ExecutionMode::Batch,
        backup_dir: Some(backup_dir.clone()),
        fail_fast: false,
        use_recycle_bin: false,
    };

    let executor = ExecutionEngine::new(config);
    executor.execute(&plan).unwrap();

    // Verify structure preserved in backup
    assert!(backup_dir.join("a/b/c/file.txt").exists());
}
```

#### 2.5.2: Property-Based Tests

```rust
// tests/property/milestone2_properties.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_verification_always_detects_missing_files(
        file_count in 1usize..50,
        missing_index in 0usize..49,
    ) {
        let temp = TempDir::new().unwrap();
        let plan = create_plan_with_n_files(&temp, file_count);

        // Delete one file
        if missing_index < file_count {
            let file_to_delete = temp.path().join(format!("file{}.txt", missing_index));
            if file_to_delete.exists() {
                std::fs::remove_file(file_to_delete).unwrap();
            }
        }

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        if missing_index < file_count {
            prop_assert!(!result.is_safe_to_execute());
            prop_assert!(result.missing.len() > 0);
        }
    }

    #[test]
    fn test_transaction_log_matches_execution_result(
        operation_count in 1usize..100,
    ) {
        let temp = TempDir::new().unwrap();
        let plan = create_plan_with_n_operations(&temp, operation_count);
        let log_path = temp.path().join("transaction.yaml");

        let mut logger = create_test_logger(&log_path);
        let result = execute_plan(&plan);

        for op in &result.operations {
            logger.log_operation(op);
        }
        logger.finalize(&result, TransactionStatus::Completed).unwrap();

        let log = TransactionLogger::read(&log_path).unwrap();

        prop_assert_eq!(log.operations.len(), result.operations.len());
        prop_assert_eq!(log.summary.unwrap().successful, result.summary.successful);
    }
}
```

#### 2.5.3: Performance Tests

```rust
// tests/performance/milestone2_performance.rs

#[test]
#[ignore]
fn test_verify_100k_entries_performance() {
    let temp = TempDir::new().unwrap();
    let plan = create_large_plan(&temp, 100_000);

    let start = Instant::now();
    let verifier = VerificationEngine::new(VerificationConfig::default());
    let result = verifier.verify(&plan).unwrap();
    let duration = start.elapsed();

    println!("Verified {} entries in {:?}", result.total_entries, duration);
    assert!(duration.as_secs() < 10, "Should verify 100K entries in <10s");
}

#[test]
#[ignore]
fn test_execute_10k_deletions_performance() {
    let temp = TempDir::new().unwrap();
    create_files(&temp, 10_000);
    let plan = generate_deletion_plan(&temp);

    let start = Instant::now();
    let executor = create_executor();
    let result = executor.execute(&plan).unwrap();
    let duration = start.elapsed();

    println!("Deleted {} files in {:?}", result.summary.successful, duration);
    assert!(duration.as_secs() < 60, "Should delete 10K files in <60s");
}
```

### Documentation Updates

#### Update README.md
```markdown
## Usage

### Scan a directory
```bash
megamaid scan /path/to/project
```

### Verify the plan
```bash
megamaid verify cleanup-plan.yaml
```

### Execute the plan (dry-run first!)
```bash
# Dry-run to see what would happen
megamaid execute cleanup-plan.yaml --dry-run

# Interactive mode
megamaid execute cleanup-plan.yaml --interactive

# Batch execution
megamaid execute cleanup-plan.yaml

# With backup
megamaid execute cleanup-plan.yaml --backup-dir ./backups
```
```

### Acceptance Criteria
- [ ] All integration tests pass
- [ ] Property-based tests pass 1000+ iterations
- [ ] Performance tests meet targets
- [ ] Documentation updated
- [ ] All 100+ tests passing (Milestone 1 + 2)
- [ ] Code coverage >85% for all new modules

---

## Testing Summary for Milestone 2

### Test Coverage Requirements

| Component | Unit Tests | Integration Tests | E2E Tests | Coverage Target |
|-----------|-----------|-------------------|-----------|----------------|
| Verifier | ✓ | ✓ | ✓ | 90% |
| Executor | ✓ | ✓ | ✓ | 85% |
| Transaction Logger | ✓ | ✓ | - | 85% |
| CLI Commands | ✓ | - | ✓ | 80% |

**Expected Test Count**: 150+ total (87 from M1 + 60+ new for M2)

---

## Success Metrics

### Functional
- [ ] Verification detects all types of drift with <0.1% false positives
- [ ] Dry-run mode produces accurate predictions
- [ ] Backup mode preserves directory structure
- [ ] Transaction logs provide complete audit trail
- [ ] All tests pass on Windows 11

### Performance
- [ ] Verify 100K entries in <10 seconds
- [ ] Execute 10K deletions in <60 seconds
- [ ] Transaction log write <1 second for 10K operations
- [ ] Memory usage <150MB for 100K entry verification

### Quality
- [ ] Code coverage >85% for all Milestone 2 modules
- [ ] Zero clippy warnings
- [ ] All documentation updated
- [ ] Property tests pass 1000+ iterations

---

## Risk Mitigation

### Identified Risks

1. **Accidental Data Loss**
   - Mitigation: Mandatory verification by default, dry-run mode, backup option
   - Fallback: Transaction logs for audit trail

2. **Locked Files on Windows**
   - Mitigation: Skip with warning, log for manual review
   - Alternative: Retry mechanism with exponential backoff (future)

3. **Recycle Bin Integration Complexity**
   - Mitigation: Make it optional, graceful fallback to regular deletion
   - Testing: Extensive Windows-specific tests

4. **Large Transaction Logs**
   - Mitigation: YAML format is reasonably compact
   - Alternative: Optional compression for logs >10MB (future)

---

## Deliverables Checklist

- [ ] Verification engine with drift detection
- [ ] Execution engine with multiple modes
- [ ] Transaction logging system
- [ ] CLI verify and execute commands
- [ ] 60+ unit tests for new modules
- [ ] 15+ integration tests
- [ ] 8+ E2E tests
- [ ] 5+ property-based tests
- [ ] 2+ performance benchmarks
- [ ] Updated user documentation
- [ ] Updated developer documentation

---

## Next Steps (Milestone 3 Preview)

After Milestone 2 completion:
- Parallel scanning and deletion (rayon integration)
- Advanced progress reporting with ETA
- Configuration file support
- Custom detection rules from config
- Archive mode (ZIP/TAR instead of delete)
