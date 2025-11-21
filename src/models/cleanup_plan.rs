//! Cleanup plan representation for YAML serialization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A cleanup plan containing entries to be processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPlan {
    /// Plan format version for future compatibility
    pub version: String,

    /// Timestamp when the plan was created
    pub created_at: DateTime<Utc>,

    /// Base directory that was scanned
    pub base_path: PathBuf,

    /// List of cleanup entries
    pub entries: Vec<CleanupEntry>,
}

/// A single entry in a cleanup plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupEntry {
    /// Path relative to base_path
    pub path: String,

    /// Size in bytes
    pub size: u64,

    /// Last modification time in RFC3339 format
    pub modified: String,

    /// Action to take on this entry
    pub action: CleanupAction,

    /// Name of the detection rule that flagged this entry
    pub rule_name: String,

    /// Reason why this was flagged
    pub reason: String,
}

/// Action to perform on a cleanup entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CleanupAction {
    /// Delete the file or directory
    Delete,

    /// Keep the file or directory
    Keep,

    /// Review manually before deciding
    Review,
}

impl CleanupPlan {
    /// Creates a new cleanup plan.
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            version: "1.0".to_string(),
            created_at: Utc::now(),
            base_path,
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the plan.
    pub fn add_entry(&mut self, entry: CleanupEntry) {
        self.entries.push(entry);
    }

    /// Returns the total size of all entries in bytes.
    pub fn total_size(&self) -> u64 {
        self.entries.iter().map(|e| e.size).sum()
    }

    /// Returns the number of entries marked for deletion.
    pub fn delete_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.action, CleanupAction::Delete))
            .count()
    }

    /// Returns the number of entries marked for review.
    pub fn review_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.action, CleanupAction::Review))
            .count()
    }

    /// Returns the number of entries marked to keep.
    pub fn keep_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.action, CleanupAction::Keep))
            .count()
    }
}

impl CleanupEntry {
    /// Creates a new cleanup entry.
    pub fn new(
        path: String,
        size: u64,
        modified: String,
        action: CleanupAction,
        rule_name: String,
        reason: String,
    ) -> Self {
        Self {
            path,
            size,
            modified,
            action,
            rule_name,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_plan_creation() {
        let base_path = PathBuf::from("/test/project");
        let plan = CleanupPlan::new(base_path.clone());

        assert_eq!(plan.version, "1.0");
        assert_eq!(plan.base_path, base_path);
        assert_eq!(plan.entries.len(), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut plan = CleanupPlan::new(PathBuf::from("/test"));

        let entry = CleanupEntry::new(
            "target/debug".to_string(),
            1_000_000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Delete,
            "build_artifact".to_string(),
            "Build artifact".to_string(),
        );

        plan.add_entry(entry);
        assert_eq!(plan.entries.len(), 1);
    }

    #[test]
    fn test_total_size() {
        let mut plan = CleanupPlan::new(PathBuf::from("/test"));

        plan.add_entry(CleanupEntry::new(
            "file1.txt".to_string(),
            1000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Delete,
            "test_rule".to_string(),
            "Test".to_string(),
        ));

        plan.add_entry(CleanupEntry::new(
            "file2.txt".to_string(),
            2000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Review,
            "test_rule".to_string(),
            "Test".to_string(),
        ));

        assert_eq!(plan.total_size(), 3000);
    }

    #[test]
    fn test_action_counts() {
        let mut plan = CleanupPlan::new(PathBuf::from("/test"));

        plan.add_entry(CleanupEntry::new(
            "file1.txt".to_string(),
            1000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Delete,
            "test_rule".to_string(),
            "Test".to_string(),
        ));

        plan.add_entry(CleanupEntry::new(
            "file2.txt".to_string(),
            2000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Review,
            "test_rule".to_string(),
            "Test".to_string(),
        ));

        plan.add_entry(CleanupEntry::new(
            "file3.txt".to_string(),
            3000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Keep,
            "test_rule".to_string(),
            "Test".to_string(),
        ));

        assert_eq!(plan.delete_count(), 1);
        assert_eq!(plan.review_count(), 1);
        assert_eq!(plan.keep_count(), 1);
    }

    #[test]
    fn test_cleanup_action_string_representation() {
        // Test serde serialization produces correct strings
        let delete = CleanupAction::Delete;
        let keep = CleanupAction::Keep;
        let review = CleanupAction::Review;

        let delete_json = serde_json::to_string(&delete).unwrap();
        let keep_json = serde_json::to_string(&keep).unwrap();
        let review_json = serde_json::to_string(&review).unwrap();

        assert_eq!(delete_json, "\"delete\"");
        assert_eq!(keep_json, "\"keep\"");
        assert_eq!(review_json, "\"review\"");
    }

    #[test]
    fn test_cleanup_plan_serialization() {
        let mut plan = CleanupPlan::new(PathBuf::from("/test"));

        plan.add_entry(CleanupEntry::new(
            "target".to_string(),
            1_000_000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Delete,
            "build_artifact".to_string(),
            "Build artifact".to_string(),
        ));

        // Test YAML serialization
        let yaml_string = serde_yaml::to_string(&plan).unwrap();
        assert!(yaml_string.contains("version"));
        assert!(yaml_string.contains("base_path"));
        assert!(yaml_string.contains("entries"));
    }

    #[test]
    fn test_cleanup_plan_deserialization() {
        let yaml_str = r#"
version: "1.0"
created_at: "2025-11-19T12:00:00Z"
base_path: "/test"
entries:
  - path: "target"
    size: 1000000
    modified: "2025-11-19T12:00:00Z"
    action: delete
    rule_name: build_artifact
    reason: Build artifact
        "#;

        let plan: CleanupPlan = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(plan.version, "1.0");
        assert_eq!(plan.base_path, PathBuf::from("/test"));
        assert_eq!(plan.entries.len(), 1);
        assert_eq!(plan.entries[0].path, "target");
        assert_eq!(plan.entries[0].action, CleanupAction::Delete);
    }

    #[test]
    fn test_relative_path_handling() {
        let entry = CleanupEntry::new(
            "subfolder/target/debug".to_string(),
            1000,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Delete,
            "test_rule".to_string(),
            "Test".to_string(),
        );

        // Paths should be stored as strings, relative to base_path
        assert_eq!(entry.path, "subfolder/target/debug");
        assert!(!entry.path.starts_with('/'));
    }
}
