//! Plan generation from detection results.

use crate::detector::DetectionResult;
use crate::models::{CleanupAction, CleanupEntry, CleanupPlan};
use chrono::Utc;
use std::path::{Path, PathBuf};

/// Generates cleanup plans from detection results.
pub struct PlanGenerator {
    base_path: PathBuf,
}

impl PlanGenerator {
    /// Creates a new PlanGenerator for the specified base path.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Generates a cleanup plan from detection results.
    ///
    /// Each detection result is converted to a CleanupEntry with an appropriate
    /// default action based on the rule type.
    pub fn generate(&self, detections: Vec<DetectionResult>) -> CleanupPlan {
        let mut plan = CleanupPlan {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now(),
            base_path: self.base_path.clone(),
            entries: Vec::new(),
        };

        for detection in detections {
            let action = self.default_action_for_rule(&detection.rule_name);

            // Convert absolute path to relative path string
            let relative_path = detection
                .entry
                .path
                .strip_prefix(&self.base_path)
                .unwrap_or(&detection.entry.path)
                .to_string_lossy()
                .to_string();

            // Convert SystemTime to RFC3339 string
            let modified = chrono::DateTime::<Utc>::from(detection.entry.modified).to_rfc3339();

            plan.add_entry(CleanupEntry {
                path: relative_path,
                size: detection.entry.size,
                modified,
                action,
                rule_name: detection.rule_name.clone(),
                reason: detection.reason.clone(),
            });
        }

        plan
    }

    /// Determines the default action based on rule type.
    ///
    /// - Build artifacts default to Delete (safe to regenerate)
    /// - Large files default to Review (user discretion)
    /// - Unknown rules default to Review (conservative)
    fn default_action_for_rule(&self, rule_name: &str) -> CleanupAction {
        match rule_name {
            "build_artifact" => CleanupAction::Delete,
            "large_file" => CleanupAction::Review,
            _ => CleanupAction::Review,
        }
    }

    /// Returns the base path for this generator.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntryType, FileEntry};
    use std::time::SystemTime;

    fn create_test_detection(
        path: &str,
        size: u64,
        rule_name: &str,
        reason: &str,
    ) -> DetectionResult {
        DetectionResult {
            entry: FileEntry::new(
                PathBuf::from(path),
                size,
                SystemTime::now(),
                EntryType::File,
            ),
            rule_name: rule_name.to_string(),
            reason: reason.to_string(),
        }
    }

    #[test]
    fn test_generate_empty_plan() {
        let generator = PlanGenerator::new(PathBuf::from("/test"));
        let plan = generator.generate(Vec::new());

        assert_eq!(plan.entries.len(), 0);
        assert_eq!(plan.base_path, PathBuf::from("/test"));
        assert_eq!(plan.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_generate_with_detections() {
        let generator = PlanGenerator::new(PathBuf::from("/test"));

        let detections = vec![
            create_test_detection("/test/target", 1000, "build_artifact", "Build artifact"),
            create_test_detection("/test/large.bin", 200_000_000, "large_file", "Large file"),
        ];

        let plan = generator.generate(detections);

        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.base_path, PathBuf::from("/test"));
    }

    #[test]
    fn test_build_artifact_defaults_to_delete() {
        let generator = PlanGenerator::new(PathBuf::from("/test"));

        let detections = vec![create_test_detection(
            "/test/target",
            1000,
            "build_artifact",
            "Build artifact",
        )];

        let plan = generator.generate(detections);

        assert_eq!(plan.entries[0].action, CleanupAction::Delete);
    }

    #[test]
    fn test_large_file_defaults_to_review() {
        let generator = PlanGenerator::new(PathBuf::from("/test"));

        let detections = vec![create_test_detection(
            "/test/large.bin",
            200_000_000,
            "large_file",
            "Large file",
        )];

        let plan = generator.generate(detections);

        assert_eq!(plan.entries[0].action, CleanupAction::Review);
    }

    #[test]
    fn test_unknown_rule_defaults_to_review() {
        let generator = PlanGenerator::new(PathBuf::from("/test"));

        let detections = vec![create_test_detection(
            "/test/file.txt",
            1000,
            "unknown_rule",
            "Unknown",
        )];

        let plan = generator.generate(detections);

        assert_eq!(plan.entries[0].action, CleanupAction::Review);
    }

    #[test]
    fn test_preserves_detection_metadata() {
        let generator = PlanGenerator::new(PathBuf::from("/test"));

        let detections = vec![create_test_detection(
            "/test/file.txt",
            12345,
            "test_rule",
            "Test reason",
        )];

        let plan = generator.generate(detections);

        // Path should be relative to base path
        assert_eq!(plan.entries[0].path, "file.txt");
        assert_eq!(plan.entries[0].size, 12345);
        assert_eq!(plan.entries[0].rule_name, "test_rule");
        assert_eq!(plan.entries[0].reason, "Test reason");
    }

    #[test]
    fn test_base_path_accessor() {
        let generator = PlanGenerator::new(PathBuf::from("/test"));

        assert_eq!(generator.base_path(), Path::new("/test"));
    }
}
