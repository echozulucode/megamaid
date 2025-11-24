//! Verification engine for detecting filesystem drift.

use crate::models::{CleanupAction, CleanupPlan};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

/// Configuration for verification behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Check modification times
    pub check_mtime: bool,
    /// Check file sizes
    pub check_size: bool,
    /// Fail fast on first drift
    pub fail_fast: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            check_mtime: true,
            check_size: true,
            fail_fast: false,
        }
    }
}

/// Engine for verifying cleanup plans against current filesystem state.
pub struct VerificationEngine {
    config: VerificationConfig,
}

/// Result of verification operation.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    pub total_entries: usize,
    pub verified: usize,
    pub drifted: Vec<DriftDetection>,
    pub missing: Vec<PathBuf>,
    pub permission_errors: Vec<PathBuf>,
}

impl VerificationResult {
    /// Check if any drift was detected.
    pub fn has_drift(&self) -> bool {
        !self.drifted.is_empty() || !self.missing.is_empty()
    }

    /// Check if plan is safe to execute.
    ///
    /// A plan is safe if no drift detected. Permission errors are warnings only.
    pub fn is_safe_to_execute(&self) -> bool {
        !self.has_drift()
    }
}

/// Details of a detected drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetection {
    pub path: PathBuf,
    pub drift_type: DriftType,
    pub expected: String,
    pub actual: String,
}

/// Types of drift that can be detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftType {
    SizeMismatch,
    ModificationTimeMismatch,
}

impl VerificationEngine {
    /// Create a new verification engine with the given configuration.
    pub fn new(config: VerificationConfig) -> Self {
        Self { config }
    }

    /// Verify a cleanup plan against the current filesystem state.
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
            if entry.action == CleanupAction::Keep {
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
                    // For directories, calculate recursive size
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
                // Some filesystems only have 2-second precision
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

    /// Calculate the total size of all files in a directory recursively.
    fn calculate_dir_size(&self, dir_path: &Path) -> Result<u64, VerificationError> {
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

/// Errors that can occur during verification.
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid timestamp in plan: {0}")]
    InvalidTimestamp(String),

    #[error("Walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CleanupEntry;
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
    fn test_verify_valid_plan() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let entry = CleanupEntry {
            path: "test.txt".to_string(),
            size: metadata.len(),
            modified: chrono::DateTime::<chrono::Utc>::from(metadata.modified().unwrap())
                .to_rfc3339(),
            action: CleanupAction::Delete,
            rule_name: "test".to_string(),
            reason: "test".to_string(),
        };

        let plan = create_test_plan(temp.path(), vec![entry]);

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.verified, 1);
        assert!(!result.has_drift());
        assert!(result.is_safe_to_execute());
    }

    #[test]
    fn test_detect_missing_file() {
        let temp = TempDir::new().unwrap();
        let entry = create_cleanup_entry("nonexistent.txt", 100, CleanupAction::Delete);
        let plan = create_test_plan(temp.path(), vec![entry]);

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
        fs::write(&file_path, "original").unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let entry = CleanupEntry {
            path: "modified.txt".to_string(),
            size: metadata.len(),
            modified: chrono::DateTime::<chrono::Utc>::from(metadata.modified().unwrap())
                .to_rfc3339(),
            action: CleanupAction::Delete,
            rule_name: "test".to_string(),
            reason: "test".to_string(),
        };

        let plan = create_test_plan(temp.path(), vec![entry]);

        // Modify file (change size)
        fs::write(&file_path, "modified content is much longer").unwrap();

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
        fs::write(&file_path, "content").unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let entry = CleanupEntry {
            path: "touched.txt".to_string(),
            size: metadata.len(),
            modified: chrono::DateTime::<chrono::Utc>::from(metadata.modified().unwrap())
                .to_rfc3339(),
            action: CleanupAction::Delete,
            rule_name: "test".to_string(),
            reason: "test".to_string(),
        };

        let plan = create_test_plan(temp.path(), vec![entry]);

        // Wait and touch file (same content, different mtime)
        // Sleep for 5 seconds to exceed the 2-second tolerance in verifier
        std::thread::sleep(std::time::Duration::from_secs(5));
        fs::write(&file_path, "content").unwrap();

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert!(!result.drifted.is_empty());
        assert!(!result.is_safe_to_execute());
    }

    #[test]
    fn test_skip_keep_actions() {
        let temp = TempDir::new().unwrap();
        let entry = create_cleanup_entry("keep.txt", 100, CleanupAction::Keep);
        let plan = create_test_plan(temp.path(), vec![entry]);

        // Don't create the file - verification should still pass because action is Keep

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.verified, 1);
        assert!(!result.has_drift());
    }

    #[test]
    fn test_fail_fast_mode() {
        let temp = TempDir::new().unwrap();

        let entries = vec![
            create_cleanup_entry("missing1.txt", 100, CleanupAction::Delete),
            create_cleanup_entry("missing2.txt", 100, CleanupAction::Delete),
        ];

        let plan = create_test_plan(temp.path(), entries);

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
        fs::create_dir(&dir_path).unwrap();
        fs::write(dir_path.join("file1.txt"), "a".repeat(100)).unwrap();
        fs::write(dir_path.join("file2.txt"), "b".repeat(200)).unwrap();

        let metadata = fs::metadata(&dir_path).unwrap();
        let entry = CleanupEntry {
            path: "test_dir".to_string(),
            size: 300, // 100 + 200
            modified: chrono::DateTime::<chrono::Utc>::from(metadata.modified().unwrap())
                .to_rfc3339(),
            action: CleanupAction::Delete,
            rule_name: "test".to_string(),
            reason: "test".to_string(),
        };

        let plan = create_test_plan(temp.path(), vec![entry]);

        let verifier = VerificationEngine::new(VerificationConfig::default());
        let result = verifier.verify(&plan).unwrap();

        assert_eq!(result.verified, 1);
        assert!(!result.has_drift());
    }

    #[test]
    fn test_skip_mtime_check() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let entry = CleanupEntry {
            path: "test.txt".to_string(),
            size: metadata.len(),
            modified: chrono::DateTime::<chrono::Utc>::from(metadata.modified().unwrap())
                .to_rfc3339(),
            action: CleanupAction::Delete,
            rule_name: "test".to_string(),
            reason: "test".to_string(),
        };

        let plan = create_test_plan(temp.path(), vec![entry]);

        // Wait and touch file
        std::thread::sleep(std::time::Duration::from_secs(3));
        fs::write(&file_path, "content").unwrap();

        // Verify with mtime checking disabled
        let config = VerificationConfig {
            check_mtime: false,
            ..Default::default()
        };

        let verifier = VerificationEngine::new(config);
        let result = verifier.verify(&plan).unwrap();

        // Should not detect drift because mtime check is disabled
        assert_eq!(result.verified, 1);
        assert!(!result.has_drift());
    }
}
