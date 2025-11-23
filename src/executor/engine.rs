//! Execution engine for safe deletion operations.

use crate::models::{CleanupAction, CleanupEntry, CleanupPlan};
use crate::scanner::progress::AdvancedProgress;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};

/// Configuration for execution behavior.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub backup_dir: Option<PathBuf>,
    pub fail_fast: bool,
    pub use_recycle_bin: bool,
    /// Enable parallel deletion (not compatible with Interactive mode)
    pub parallel: bool,
    /// Batch size for parallel processing (default: 100)
    pub batch_size: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Batch,
            backup_dir: None,
            fail_fast: false,
            use_recycle_bin: false,
            parallel: false,
            batch_size: 100,
        }
    }
}

/// Execution modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Simulate execution without actually deleting
    DryRun,
    /// Prompt for confirmation on each deletion
    Interactive,
    /// Execute all Delete actions automatically
    Batch,
}

/// Engine for executing cleanup plans.
pub struct ExecutionEngine {
    config: ExecutionConfig,
    progress: Arc<AdvancedProgress>,
}

/// Result of execution operation.
#[derive(Debug)]
pub struct ExecutionResult {
    pub operations: Vec<OperationResult>,
    pub summary: ExecutionSummary,
}

/// Summary of execution.
#[derive(Debug)]
pub struct ExecutionSummary {
    pub total_operations: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub space_freed: u64,
    pub duration: std::time::Duration,
}

/// Result of a single operation.
#[derive(Debug, Clone)]
pub struct OperationResult {
    pub path: PathBuf,
    pub action: OperationAction,
    pub status: OperationStatus,
    pub size_freed: Option<u64>,
    pub error: Option<String>,
    pub timestamp: SystemTime,
}

/// Action performed on an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationAction {
    Delete,
    MoveToBackup,
    MoveToRecycleBin,
    Skip,
}

/// Status of an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatus {
    Success,
    Failed,
    Skipped,
    DryRun,
}

impl ExecutionEngine {
    /// Create a new execution engine with the given configuration.
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            config,
            progress: Arc::new(AdvancedProgress::new()),
        }
    }

    /// Get a reference to the progress tracker.
    pub fn progress(&self) -> &AdvancedProgress {
        &self.progress
    }

    /// Execute a cleanup plan.
    pub fn execute(&self, plan: &CleanupPlan) -> Result<ExecutionResult, ExecutionError> {
        // Validate: parallel mode not compatible with interactive
        if self.config.parallel && self.config.mode == ExecutionMode::Interactive {
            return Err(ExecutionError::InvalidConfiguration(
                "Parallel execution is not compatible with Interactive mode".to_string(),
            ));
        }

        // Dispatch to parallel or sequential execution
        if self.config.parallel {
            self.execute_parallel(plan)
        } else {
            self.execute_sequential(plan)
        }
    }

    /// Execute plan sequentially (original implementation).
    fn execute_sequential(&self, plan: &CleanupPlan) -> Result<ExecutionResult, ExecutionError> {
        let start_time = Instant::now();
        let mut operations = Vec::new();

        // Filter entries to process
        let entries_to_process: Vec<_> = plan
            .entries
            .iter()
            .filter(|e| e.action == CleanupAction::Delete)
            .collect();

        self.progress.set_total(entries_to_process.len() as u64);

        for entry in entries_to_process {
            let full_path = plan.base_path.join(&entry.path);

            // Interactive mode: prompt user
            if self.config.mode == ExecutionMode::Interactive {
                match self.prompt_user(entry)? {
                    UserChoice::Yes => {
                        // Continue to execute
                    }
                    UserChoice::No => {
                        operations.push(OperationResult {
                            path: full_path,
                            action: OperationAction::Skip,
                            status: OperationStatus::Skipped,
                            size_freed: None,
                            error: None,
                            timestamp: SystemTime::now(),
                        });
                        self.progress.increment();
                        continue;
                    }
                    UserChoice::Abort => {
                        return Err(ExecutionError::UserAborted);
                    }
                }
            }

            // Execute operation
            let result = self.execute_single(&full_path, entry);
            self.progress.increment();

            // Fail-fast check
            if self.config.fail_fast && result.status == OperationStatus::Failed {
                operations.push(result);
                break;
            }

            operations.push(result);
        }

        let duration = start_time.elapsed();
        let summary = self.compute_summary(&operations, duration);

        Ok(ExecutionResult {
            operations,
            summary,
        })
    }

    /// Execute plan in parallel using rayon.
    fn execute_parallel(&self, plan: &CleanupPlan) -> Result<ExecutionResult, ExecutionError> {
        let start_time = Instant::now();

        // Filter entries to process
        let entries_to_process: Vec<_> = plan
            .entries
            .iter()
            .filter(|e| e.action == CleanupAction::Delete)
            .collect();

        self.progress.set_total(entries_to_process.len() as u64);

        // Shared state for collecting results
        let results = Arc::new(Mutex::new(Vec::new()));
        let should_abort = Arc::new(Mutex::new(false));

        // Process in batches for better error handling
        let batches: Vec<_> = entries_to_process.chunks(self.config.batch_size).collect();

        for batch in batches {
            // Check abort signal
            if *should_abort.lock().unwrap() {
                break;
            }

            // Process batch in parallel
            let batch_results: Vec<OperationResult> = batch
                .par_iter()
                .map(|entry| {
                    let full_path = plan.base_path.join(&entry.path);
                    let result = self.execute_single(&full_path, entry);
                    self.progress.increment();
                    result
                })
                .collect();

            // Collect results
            {
                let mut results_guard = results.lock().unwrap();
                results_guard.extend(batch_results);

                // Check for fail-fast condition
                if self.config.fail_fast
                    && results_guard
                        .iter()
                        .any(|r| r.status == OperationStatus::Failed)
                {
                    *should_abort.lock().unwrap() = true;
                }
            }
        }

        let duration = start_time.elapsed();
        let operations = Arc::try_unwrap(results)
            .map(|mutex| mutex.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone());
        let summary = self.compute_summary(&operations, duration);

        Ok(ExecutionResult {
            operations,
            summary,
        })
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
        } else if self.config.backup_dir.is_some() {
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

    fn move_to_recycle_bin(&self, path: &Path) -> Result<(), std::io::Error> {
        // Use trash crate for cross-platform recycle bin support
        trash::delete(path).map_err(std::io::Error::other)
    }

    fn prompt_user(&self, entry: &CleanupEntry) -> Result<UserChoice, ExecutionError> {
        use std::io::{self, Write};

        println!("\n{}", "=".repeat(60));
        println!("Path: {}", entry.path);
        println!("Size: {:.2} MB", entry.size as f64 / 1_048_576.0);
        println!("Reason: {}", entry.reason);
        println!("{}", "=".repeat(60));
        print!("Delete this file/directory? [y/n/a]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(UserChoice::Yes),
            "n" | "no" => Ok(UserChoice::No),
            "a" | "abort" | "q" | "quit" => Ok(UserChoice::Abort),
            _ => {
                println!("Invalid input. Skipping.");
                Ok(UserChoice::No)
            }
        }
    }

    fn compute_summary(
        &self,
        operations: &[OperationResult],
        duration: std::time::Duration,
    ) -> ExecutionSummary {
        let total_operations = operations.len();
        let successful = operations
            .iter()
            .filter(|o| o.status == OperationStatus::Success || o.status == OperationStatus::DryRun)
            .count();
        let failed = operations
            .iter()
            .filter(|o| o.status == OperationStatus::Failed)
            .count();
        let skipped = operations
            .iter()
            .filter(|o| o.status == OperationStatus::Skipped)
            .count();
        let space_freed = operations.iter().filter_map(|o| o.size_freed).sum();

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

enum UserChoice {
    Yes,
    No,
    Abort,
}

/// Errors that can occur during execution.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("User aborted execution")]
    UserAborted,

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_plan(base_path: &Path, entries: Vec<CleanupEntry>) -> CleanupPlan {
        CleanupPlan {
            version: "0.1.0".to_string(),
            created_at: Utc::now(),
            base_path: base_path.to_path_buf(),
            entries,
        }
    }

    fn create_cleanup_entry(path: &str, size: u64, action: CleanupAction) -> CleanupEntry {
        CleanupEntry {
            path: path.to_string(),
            size,
            modified: Utc::now().to_rfc3339(),
            action,
            rule_name: "test".to_string(),
            reason: "test reason".to_string(),
        }
    }

    #[test]
    fn test_dry_run_mode() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let entry = create_cleanup_entry("test.txt", 7, CleanupAction::Delete);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::DryRun,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.total_operations, 1);
        assert_eq!(result.operations[0].status, OperationStatus::DryRun);
        assert!(
            file_path.exists(),
            "File should still exist in dry-run mode"
        );
    }

    #[test]
    fn test_batch_delete_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("delete_me.txt");
        fs::write(&file_path, "content").unwrap();

        let entry = create_cleanup_entry("delete_me.txt", 7, CleanupAction::Delete);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            ..Default::default()
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
        fs::create_dir(&dir_path).unwrap();
        fs::write(dir_path.join("file.txt"), "content").unwrap();

        let entry = create_cleanup_entry("delete_dir", 7, CleanupAction::Delete);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            ..Default::default()
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
        fs::create_dir(&backup_dir).unwrap();

        let file_path = temp.path().join("move_me.txt");
        fs::write(&file_path, "content").unwrap();

        let entry = create_cleanup_entry("move_me.txt", 7, CleanupAction::Delete);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: Some(backup_dir.clone()),
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 1);
        assert!(!file_path.exists(), "Original file should be gone");
        assert!(
            backup_dir.join("move_me.txt").exists(),
            "File should be in backup"
        );
    }

    #[test]
    fn test_backup_preserves_directory_structure() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backups");
        fs::create_dir(&backup_dir).unwrap();

        // Create nested structure
        let nested_dir = temp.path().join("a").join("b");
        fs::create_dir_all(&nested_dir).unwrap();
        let file_path = nested_dir.join("file.txt");
        fs::write(&file_path, "content").unwrap();

        let entry = create_cleanup_entry("a/b/file.txt", 7, CleanupAction::Delete);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            backup_dir: Some(backup_dir.clone()),
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 1);
        assert!(
            backup_dir.join("a/b/file.txt").exists(),
            "Structure should be preserved in backup"
        );
    }

    #[test]
    fn test_fail_fast_on_error() {
        let temp = TempDir::new().unwrap();

        let entries = vec![
            create_cleanup_entry("nonexistent.txt", 100, CleanupAction::Delete),
            create_cleanup_entry("also_should_not_run.txt", 100, CleanupAction::Delete),
        ];

        let plan = create_test_plan(temp.path(), entries);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            fail_fast: true,
            ..Default::default()
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
        fs::write(&file_path, "content").unwrap();

        let entry = create_cleanup_entry("keep_me.txt", 7, CleanupAction::Keep);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.total_operations, 0);
        assert!(
            file_path.exists(),
            "File with Keep action should not be deleted"
        );
    }

    #[test]
    fn test_skip_review_actions() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("review_me.txt");
        fs::write(&file_path, "content").unwrap();

        let entry = create_cleanup_entry("review_me.txt", 7, CleanupAction::Review);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.total_operations, 0);
        assert!(file_path.exists(), "Review action should not delete");
    }

    #[test]
    fn test_execution_summary_accuracy() {
        let temp = TempDir::new().unwrap();

        // Create multiple files
        for i in 0..5 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "x".repeat(1000)).unwrap();
        }

        let mut entries = Vec::new();
        for i in 0..5 {
            entries.push(create_cleanup_entry(
                &format!("file{}.txt", i),
                1000,
                CleanupAction::Delete,
            ));
        }

        let plan = create_test_plan(temp.path(), entries);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 5);
        assert_eq!(result.summary.space_freed, 5000);
        // Duration should be non-zero (use as_nanos for sub-millisecond precision)
        assert!(result.summary.duration.as_nanos() > 0);
    }

    #[test]
    fn test_continue_on_error_without_fail_fast() {
        let temp = TempDir::new().unwrap();

        // Create one valid file
        fs::write(temp.path().join("valid.txt"), "content").unwrap();

        let entries = vec![
            create_cleanup_entry("nonexistent.txt", 100, CleanupAction::Delete),
            create_cleanup_entry("valid.txt", 7, CleanupAction::Delete),
        ];

        let plan = create_test_plan(temp.path(), entries);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            fail_fast: false,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        // Should continue after first failure
        assert_eq!(result.summary.failed, 1);
        assert_eq!(result.summary.successful, 1);
        assert_eq!(result.operations.len(), 2);
    }

    #[test]
    fn test_parallel_execution() {
        let temp = TempDir::new().unwrap();

        // Create multiple files
        for i in 0..20 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let mut entries = Vec::new();
        for i in 0..20 {
            entries.push(create_cleanup_entry(
                &format!("file{}.txt", i),
                7,
                CleanupAction::Delete,
            ));
        }

        let plan = create_test_plan(temp.path(), entries);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            parallel: true,
            batch_size: 5,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 20);
        assert_eq!(result.summary.failed, 0);
        assert_eq!(result.summary.space_freed, 140);

        // Verify all files deleted
        for i in 0..20 {
            assert!(
                !temp.path().join(format!("file{}.txt", i)).exists(),
                "File {} should be deleted",
                i
            );
        }
    }

    #[test]
    fn test_parallel_with_interactive_mode_rejected() {
        let temp = TempDir::new().unwrap();
        let entry = create_cleanup_entry("test.txt", 7, CleanupAction::Delete);
        let plan = create_test_plan(temp.path(), vec![entry]);

        let config = ExecutionConfig {
            mode: ExecutionMode::Interactive,
            parallel: true,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan);

        assert!(result.is_err());
        match result {
            Err(ExecutionError::InvalidConfiguration(_)) => {
                // Expected
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[test]
    fn test_parallel_fail_fast() {
        let temp = TempDir::new().unwrap();

        // Create some files, leave some nonexistent
        for i in 0..5 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let mut entries = Vec::new();
        // Mix of existing and non-existing files
        for i in 0..10 {
            entries.push(create_cleanup_entry(
                &format!("file{}.txt", i),
                7,
                CleanupAction::Delete,
            ));
        }

        let plan = create_test_plan(temp.path(), entries);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            parallel: true,
            fail_fast: true,
            batch_size: 3,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        // With fail-fast, should stop after first batch with errors
        assert!(result.summary.failed > 0);
        // Not all operations should have been processed
        assert!(result.operations.len() < 10);
    }

    #[test]
    fn test_parallel_dry_run() {
        let temp = TempDir::new().unwrap();

        // Create files
        for i in 0..10 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let mut entries = Vec::new();
        for i in 0..10 {
            entries.push(create_cleanup_entry(
                &format!("file{}.txt", i),
                7,
                CleanupAction::Delete,
            ));
        }

        let plan = create_test_plan(temp.path(), entries);

        let config = ExecutionConfig {
            mode: ExecutionMode::DryRun,
            parallel: true,
            batch_size: 5,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        // All should be dry-run status
        assert_eq!(result.operations.len(), 10);
        assert!(result.operations.iter().all(|r| r.status == OperationStatus::DryRun));
        assert_eq!(result.summary.space_freed, 70);

        // Files should still exist
        for i in 0..10 {
            assert!(
                temp.path().join(format!("file{}.txt", i)).exists(),
                "File {} should still exist in dry-run",
                i
            );
        }
    }

    #[test]
    fn test_parallel_progress_tracking() {
        let temp = TempDir::new().unwrap();

        // Create files
        for i in 0..50 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let mut entries = Vec::new();
        for i in 0..50 {
            entries.push(create_cleanup_entry(
                &format!("file{}.txt", i),
                7,
                CleanupAction::Delete,
            ));
        }

        let plan = create_test_plan(temp.path(), entries);

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            parallel: true,
            batch_size: 10,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);

        // Check initial progress state
        assert_eq!(executor.progress().get_total(), 0);
        assert_eq!(executor.progress().get_processed(), 0);

        let result = executor.execute(&plan).unwrap();

        // After execution, progress should be complete
        assert_eq!(executor.progress().get_total(), 50);
        assert_eq!(executor.progress().get_processed(), 50);
        assert_eq!(result.summary.successful, 50);
    }
}
