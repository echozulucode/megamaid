//! Transaction logging for execution audit trails.

use crate::executor::engine::{ExecutionResult, OperationResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Transaction log for execution operations.
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

/// Status of a transaction.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    InProgress,
    Completed,
    Failed,
    Aborted,
}

/// Options used for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOptions {
    pub dry_run: bool,
    pub backup_dir: Option<PathBuf>,
    pub use_recycle_bin: bool,
    pub fail_fast: bool,
}

/// A logged operation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggedOperation {
    pub path: String,
    pub action: String,
    pub status: String,
    pub size_freed: Option<u64>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Summary of execution in the log.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionSummaryLog {
    pub total_operations: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub space_freed: u64,
    pub duration_seconds: f64,
}

/// Logger for transaction operations.
pub struct TransactionLogger {
    log_path: PathBuf,
    log: TransactionLog,
}

impl TransactionLogger {
    /// Create a new transaction logger.
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

    /// Get the execution ID.
    pub fn execution_id(&self) -> &str {
        &self.log.execution_id
    }

    /// Log a single operation.
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

    /// Finalize the transaction log with execution results.
    pub fn finalize(
        &mut self,
        result: &ExecutionResult,
        status: TransactionStatus,
    ) -> std::io::Result<()> {
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

    /// Write the transaction log to disk.
    pub fn write(&self) -> std::io::Result<()> {
        let yaml_content = serde_yaml::to_string(&self.log)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Atomic write: write to temp file, then rename
        let temp_path = self.log_path.with_extension("tmp");
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(yaml_content.as_bytes())?;
        file.sync_all()?;

        std::fs::rename(temp_path, &self.log_path)?;
        Ok(())
    }

    /// Read a transaction log from disk.
    pub fn read(path: &Path) -> std::io::Result<TransactionLog> {
        let content = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::engine::{
        ExecutionSummary, OperationAction, OperationStatus,
    };
    use std::time::SystemTime;
    use tempfile::TempDir;

    fn create_test_operation(
        path: &str,
        status: OperationStatus,
        size_freed: Option<u64>,
    ) -> OperationResult {
        OperationResult {
            path: PathBuf::from(path),
            action: OperationAction::Delete,
            status,
            size_freed,
            error: None,
            timestamp: SystemTime::now(),
        }
    }

    fn create_test_execution_result() -> ExecutionResult {
        ExecutionResult {
            operations: vec![
                create_test_operation("test1.txt", OperationStatus::Success, Some(1000)),
                create_test_operation("test2.txt", OperationStatus::Success, Some(2000)),
            ],
            summary: ExecutionSummary {
                total_operations: 2,
                successful: 2,
                failed: 0,
                skipped: 0,
                space_freed: 3000,
                duration: std::time::Duration::from_secs(5),
            },
        }
    }

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
        let plan_path = temp.path().join("plan.yaml");

        let options = TransactionOptions {
            dry_run: false,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let mut logger = TransactionLogger::new(&plan_path, log_path, options);

        let op = create_test_operation("test.txt", OperationStatus::Success, Some(1000));
        logger.log_operation(&op);

        assert_eq!(logger.log.operations.len(), 1);
        assert_eq!(logger.log.operations[0].path, "test.txt");
    }

    #[test]
    fn test_finalize_transaction() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");
        let plan_path = temp.path().join("plan.yaml");

        let options = TransactionOptions {
            dry_run: false,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let mut logger = TransactionLogger::new(&plan_path, log_path, options);
        let result = create_test_execution_result();

        logger.finalize(&result, TransactionStatus::Completed).unwrap();

        assert_eq!(logger.log.status, TransactionStatus::Completed);
        assert!(logger.log.completed_at.is_some());
        assert!(logger.log.summary.is_some());

        let summary = logger.log.summary.unwrap();
        assert_eq!(summary.total_operations, 2);
        assert_eq!(summary.successful, 2);
        assert_eq!(summary.space_freed, 3000);
    }

    #[test]
    fn test_read_transaction_log() {
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

        let loaded = TransactionLogger::read(&log_path).unwrap();
        assert_eq!(loaded.execution_id, logger.log.execution_id);
        assert_eq!(loaded.version, logger.log.version);
    }

    #[test]
    fn test_transaction_log_serialization() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.yaml");

        let log = TransactionLog {
            version: "0.1.0".to_string(),
            execution_id: Uuid::new_v4().to_string(),
            plan_file: plan_path,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            status: TransactionStatus::Completed,
            mode: "batch".to_string(),
            options: TransactionOptions {
                dry_run: false,
                backup_dir: None,
                use_recycle_bin: false,
                fail_fast: false,
            },
            operations: vec![],
            summary: Some(ExecutionSummaryLog {
                total_operations: 1,
                successful: 1,
                failed: 0,
                skipped: 0,
                space_freed: 1000,
                duration_seconds: 1.5,
            }),
        };

        let yaml = serde_yaml::to_string(&log).unwrap();
        let roundtrip: TransactionLog = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(log.execution_id, roundtrip.execution_id);
        assert_eq!(log.status, roundtrip.status);
    }

    #[test]
    fn test_atomic_write() {
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

        // Temp file should not exist after write
        assert!(!log_path.with_extension("tmp").exists());
        assert!(log_path.exists());
    }

    #[test]
    fn test_execution_id_generation() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");
        let plan_path = temp.path().join("plan.yaml");

        let options = TransactionOptions {
            dry_run: false,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let logger1 = TransactionLogger::new(&plan_path, log_path.clone(), options.clone());
        let logger2 = TransactionLogger::new(&plan_path, log_path, options);

        // Each logger should have a unique execution ID
        assert_ne!(logger1.execution_id(), logger2.execution_id());
    }

    #[test]
    fn test_log_operation_with_error() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");
        let plan_path = temp.path().join("plan.yaml");

        let options = TransactionOptions {
            dry_run: false,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let mut logger = TransactionLogger::new(&plan_path, log_path, options);

        let mut op = create_test_operation("test.txt", OperationStatus::Failed, None);
        op.error = Some("Permission denied".to_string());

        logger.log_operation(&op);

        assert_eq!(logger.log.operations.len(), 1);
        assert_eq!(logger.log.operations[0].status, "Failed");
        assert_eq!(logger.log.operations[0].error, Some("Permission denied".to_string()));
    }

    #[test]
    fn test_dry_run_mode_in_log() {
        let temp = TempDir::new().unwrap();
        let log_path = temp.path().join("transaction.yaml");
        let plan_path = temp.path().join("plan.yaml");

        let options = TransactionOptions {
            dry_run: true,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let logger = TransactionLogger::new(&plan_path, log_path, options);

        assert_eq!(logger.log.mode, "dry_run");
        assert!(logger.log.options.dry_run);
    }
}
