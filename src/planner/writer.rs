//! Atomic plan file writing with validation.

use crate::models::CleanupPlan;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during plan writing.
#[derive(Debug, Error)]
pub enum WriteError {
    /// Failed to serialize plan to YAML
    #[error("Failed to serialize plan: {0}")]
    Serialization(#[from] serde_yaml::Error),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Plan validation failed
    #[error("Plan validation failed: {0}")]
    Validation(String),
}

/// Writes cleanup plans to YAML files with atomic operations.
pub struct PlanWriter;

impl PlanWriter {
    /// Writes a cleanup plan to a file atomically.
    ///
    /// The plan is first validated, then serialized to YAML, and finally written
    /// to a temporary file before being renamed to the target path. This ensures
    /// the write is atomic and won't leave a corrupted file on failure.
    ///
    /// # Arguments
    ///
    /// * `plan` - The cleanup plan to write
    /// * `path` - The destination file path
    ///
    /// # Errors
    ///
    /// Returns `WriteError` if validation, serialization, or I/O fails.
    pub fn write(plan: &CleanupPlan, path: &Path) -> Result<(), WriteError> {
        // Validate the plan
        Self::validate(plan)?;

        // Serialize to YAML
        let yaml_content = serde_yaml::to_string(plan)?;

        // Write atomically via temp file
        Self::write_atomic(path, &yaml_content)?;

        Ok(())
    }

    /// Validates a cleanup plan before writing.
    ///
    /// Ensures:
    /// - Plan has at least one entry (optional check)
    /// - All paths are valid
    fn validate(plan: &CleanupPlan) -> Result<(), WriteError> {
        // Validate base path exists
        if plan.base_path.as_os_str().is_empty() {
            return Err(WriteError::Validation(
                "Base path cannot be empty".to_string(),
            ));
        }

        // All entries must have non-empty paths
        for entry in &plan.entries {
            if entry.path.is_empty() {
                return Err(WriteError::Validation(
                    "Entry path cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Writes content to a file atomically using a temporary file.
    fn write_atomic(target: &Path, content: &str) -> Result<(), WriteError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate temp file path in same directory as target
        let temp_path = Self::temp_path(target);

        // Write to temp file
        fs::write(&temp_path, content)?;

        // Atomic rename
        fs::rename(&temp_path, target)?;

        Ok(())
    }

    /// Generates a temporary file path for atomic writes.
    fn temp_path(target: &Path) -> PathBuf {
        let mut temp = target.to_path_buf();
        temp.set_extension("tmp");
        temp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CleanupAction, CleanupEntry};
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_plan() -> CleanupPlan {
        let mut plan = CleanupPlan {
            version: "0.1.0".to_string(),
            created_at: Utc::now(),
            base_path: PathBuf::from("/test"),
            entries: Vec::new(),
        };

        plan.add_entry(CleanupEntry {
            path: "target".to_string(),
            size: 1000,
            modified: "2025-11-19T12:00:00Z".to_string(),
            action: CleanupAction::Delete,
            rule_name: "build_artifact".to_string(),
            reason: "Build artifact".to_string(),
        });

        plan
    }

    #[test]
    fn test_write_and_read_back() {
        let temp_dir = TempDir::new().unwrap();
        let plan_path = temp_dir.path().join("plan.yaml");

        let plan = create_test_plan();

        // Write the plan
        PlanWriter::write(&plan, &plan_path).unwrap();

        // Verify file exists
        assert!(plan_path.exists());

        // Read and deserialize
        let content = fs::read_to_string(&plan_path).unwrap();
        let loaded_plan: CleanupPlan = serde_yaml::from_str(&content).unwrap();

        // Verify content matches
        assert_eq!(loaded_plan.version, plan.version);
        assert_eq!(loaded_plan.base_path, plan.base_path);
        assert_eq!(loaded_plan.entries.len(), plan.entries.len());
    }

    #[test]
    fn test_write_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let plan_path = temp_dir.path().join("subdir/nested/plan.yaml");

        let plan = create_test_plan();

        // Write should create parent directories
        PlanWriter::write(&plan, &plan_path).unwrap();

        assert!(plan_path.exists());
        assert!(plan_path.parent().unwrap().exists());
    }

    #[test]
    fn test_validate_rejects_empty_base_path() {
        let plan = CleanupPlan {
            version: "0.1.0".to_string(),
            created_at: Utc::now(),
            base_path: PathBuf::new(), // Empty path
            entries: Vec::new(),
        };

        let result = PlanWriter::validate(&plan);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WriteError::Validation(_)));
    }

    #[test]
    fn test_validate_rejects_empty_entry_path() {
        let mut plan = CleanupPlan {
            version: "0.1.0".to_string(),
            created_at: Utc::now(),
            base_path: PathBuf::from("/test"),
            entries: Vec::new(),
        };

        plan.add_entry(CleanupEntry {
            path: String::new(), // Empty path
            size: 1000,
            modified: "2025-11-19T12:00:00Z".to_string(),
            action: CleanupAction::Delete,
            rule_name: "test".to_string(),
            reason: "Test".to_string(),
        });

        let result = PlanWriter::validate(&plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_accepts_valid_plan() {
        let plan = create_test_plan();
        let result = PlanWriter::validate(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_temp_path_generation() {
        let target = Path::new("/test/plan.yaml");
        let temp = PlanWriter::temp_path(target);

        assert_eq!(temp, PathBuf::from("/test/plan.tmp"));
    }

    #[test]
    fn test_write_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let plan_path = temp_dir.path().join("plan.yaml");

        // Write initial plan
        let plan1 = create_test_plan();
        PlanWriter::write(&plan1, &plan_path).unwrap();

        // Write second plan (should overwrite)
        let mut plan2 = create_test_plan();
        plan2.version = "0.2.0".to_string();
        PlanWriter::write(&plan2, &plan_path).unwrap();

        // Verify second version is persisted
        let content = fs::read_to_string(&plan_path).unwrap();
        let loaded: CleanupPlan = serde_yaml::from_str(&content).unwrap();
        assert_eq!(loaded.version, "0.2.0");
    }

    #[test]
    fn test_serialization_format() {
        let temp_dir = TempDir::new().unwrap();
        let plan_path = temp_dir.path().join("plan.yaml");

        let plan = create_test_plan();
        PlanWriter::write(&plan, &plan_path).unwrap();

        let content = fs::read_to_string(&plan_path).unwrap();

        // Verify YAML structure contains expected fields
        assert!(content.contains("version"));
        assert!(content.contains("created_at"));
        assert!(content.contains("base_path"));
        assert!(content.contains("entries"));
    }
}
