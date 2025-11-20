//! Detection rule implementations.

use crate::models::{EntryType, FileEntry};
use crate::detector::engine::ScanContext;

/// Trait for detection rules that identify cleanup candidates.
pub trait DetectionRule: Send + Sync {
    /// Returns the name of this rule.
    fn name(&self) -> &str;

    /// Determines if an entry should be flagged by this rule.
    fn should_flag(&self, entry: &FileEntry, context: &ScanContext) -> bool;

    /// Returns the reason why this entry was flagged.
    fn reason(&self) -> String;
}

/// Rule that flags files exceeding a size threshold.
pub struct SizeThresholdRule {
    /// Minimum size in bytes to flag
    pub threshold_bytes: u64,
}

impl DetectionRule for SizeThresholdRule {
    fn name(&self) -> &str {
        "large_file"
    }

    fn should_flag(&self, entry: &FileEntry, _context: &ScanContext) -> bool {
        entry.size >= self.threshold_bytes
    }

    fn reason(&self) -> String {
        format!(
            "File exceeds size threshold of {} MB",
            self.threshold_bytes / 1_048_576
        )
    }
}

/// Rule that flags common build artifact directories.
pub struct BuildArtifactRule {
    patterns: Vec<&'static str>,
}

impl Default for BuildArtifactRule {
    fn default() -> Self {
        Self {
            patterns: vec![
                "target",        // Rust
                "node_modules",  // Node.js
                "build",         // Generic
                ".next",         // Next.js
                "dist",          // Build outputs
                "__pycache__",   // Python
                ".pytest_cache", // pytest
                "bin",           // Binaries
                "obj",           // C#/C++
            ],
        }
    }
}

impl BuildArtifactRule {
    /// Creates a new BuildArtifactRule with default patterns.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a BuildArtifactRule with custom patterns.
    pub fn with_patterns(patterns: Vec<&'static str>) -> Self {
        Self { patterns }
    }
}

impl DetectionRule for BuildArtifactRule {
    fn name(&self) -> &str {
        "build_artifact"
    }

    fn should_flag(&self, entry: &FileEntry, _context: &ScanContext) -> bool {
        // Only apply to directories
        if entry.entry_type != EntryType::Directory {
            return false;
        }

        let dir_name = entry
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        self.patterns.iter().any(|&pattern| dir_name == pattern)
    }

    fn reason(&self) -> String {
        "Common build artifact directory".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn create_test_entry(path: &str, size: u64) -> FileEntry {
        FileEntry::new(
            PathBuf::from(path),
            size,
            SystemTime::now(),
            EntryType::File,
        )
    }

    fn create_test_entry_dir(path: &str) -> FileEntry {
        FileEntry::new(
            PathBuf::from(path),
            0,
            SystemTime::now(),
            EntryType::Directory,
        )
    }

    #[test]
    fn test_size_threshold_rule_flags_large_files() {
        let rule = SizeThresholdRule {
            threshold_bytes: 1_048_576,
        }; // 1MB

        let small_file = create_test_entry("small.txt", 1024);
        let large_file = create_test_entry("large.bin", 2_097_152);

        let context = ScanContext::default();

        assert!(!rule.should_flag(&small_file, &context));
        assert!(rule.should_flag(&large_file, &context));
    }

    #[test]
    fn test_size_threshold_rule_name() {
        let rule = SizeThresholdRule {
            threshold_bytes: 100_000_000,
        };

        assert_eq!(rule.name(), "large_file");
    }

    #[test]
    fn test_size_threshold_rule_reason() {
        let rule = SizeThresholdRule {
            threshold_bytes: 100 * 1_048_576, // Exactly 100 MB
        };

        let reason = rule.reason();
        assert!(reason.contains("100"));
        assert!(reason.contains("MB"));
    }

    #[test]
    fn test_build_artifact_rule_detects_rust_target() {
        let rule = BuildArtifactRule::default();

        let target_dir = create_test_entry_dir("/project/target");
        let src_dir = create_test_entry_dir("/project/src");

        let context = ScanContext::default();

        assert!(rule.should_flag(&target_dir, &context));
        assert!(!rule.should_flag(&src_dir, &context));
    }

    #[test]
    fn test_build_artifact_rule_detects_node_modules() {
        let rule = BuildArtifactRule::default();
        let node_modules = create_test_entry_dir("/project/node_modules");

        let context = ScanContext::default();

        assert!(rule.should_flag(&node_modules, &context));
    }

    #[test]
    fn test_build_artifact_rule_detects_all_patterns() {
        let rule = BuildArtifactRule::default();
        let context = ScanContext::default();

        let patterns = vec![
            "target",
            "node_modules",
            "build",
            ".next",
            "dist",
            "__pycache__",
            ".pytest_cache",
            "bin",
            "obj",
        ];

        for pattern in patterns {
            let dir = create_test_entry_dir(&format!("/project/{}", pattern));
            assert!(
                rule.should_flag(&dir, &context),
                "Should flag {}",
                pattern
            );
        }
    }

    #[test]
    fn test_build_artifact_rule_case_sensitive() {
        let rule = BuildArtifactRule::default();
        let target_upper = create_test_entry_dir("/project/TARGET");

        let context = ScanContext::default();

        // Should NOT match - case sensitive
        assert!(!rule.should_flag(&target_upper, &context));
    }

    #[test]
    fn test_build_artifact_rule_only_applies_to_directories() {
        let rule = BuildArtifactRule::default();
        let target_file = create_test_entry("/project/target", 100); // File named "target"

        let context = ScanContext::default();

        assert!(!rule.should_flag(&target_file, &context));
    }

    #[test]
    fn test_build_artifact_rule_custom_patterns() {
        let rule = BuildArtifactRule::with_patterns(vec!["custom", "patterns"]);

        let custom_dir = create_test_entry_dir("/project/custom");
        let target_dir = create_test_entry_dir("/project/target");

        let context = ScanContext::default();

        assert!(rule.should_flag(&custom_dir, &context));
        assert!(!rule.should_flag(&target_dir, &context)); // Not in custom patterns
    }
}
