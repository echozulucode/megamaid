//! Detection engine that orchestrates rules.

use crate::detector::rules::{BuildArtifactRule, DetectionRule, SizeThresholdRule};
use crate::models::FileEntry;

/// Context information for detection rules.
#[derive(Debug, Default)]
pub struct ScanContext {
    // Future: could include statistics, parent directory info, etc.
}

/// Result of applying detection rules to an entry.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// The file entry that was flagged
    pub entry: FileEntry,

    /// Name of the rule that flagged it
    pub rule_name: String,

    /// Reason it was flagged
    pub reason: String,
}

/// Engine that applies multiple detection rules to identify cleanup candidates.
pub struct DetectionEngine {
    rules: Vec<Box<dyn DetectionRule>>,
}

impl DetectionEngine {
    /// Creates a new DetectionEngine with default rules.
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(SizeThresholdRule {
                    threshold_bytes: 100 * 1_048_576, // 100MB
                }),
                Box::new(BuildArtifactRule::default()),
            ],
        }
    }

    /// Creates an empty DetectionEngine with no rules.
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a rule to the engine.
    pub fn add_rule(&mut self, rule: Box<dyn DetectionRule>) {
        self.rules.push(rule);
    }

    /// Analyzes entries and returns those flagged by any rule.
    ///
    /// Each entry is flagged at most once (first matching rule wins).
    pub fn analyze(&self, entries: &[FileEntry], context: &ScanContext) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        for entry in entries {
            // Try each rule in order; first match wins
            for rule in &self.rules {
                if rule.should_flag(entry, context) {
                    results.push(DetectionResult {
                        entry: entry.clone(),
                        rule_name: rule.name().to_string(),
                        reason: rule.reason(),
                    });
                    break; // Only flag once per entry
                }
            }
        }

        results
    }

    /// Returns the number of rules in this engine.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for DetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntryType;
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
    fn test_engine_applies_multiple_rules() {
        let engine = DetectionEngine::new();

        let entries = vec![
            create_test_entry("large.bin", 200_000_000), // 200MB - size rule
            create_test_entry_dir("target"),             // build artifact rule
            create_test_entry("normal.txt", 1024),       // should not flag
        ];

        let results = engine.analyze(&entries, &ScanContext::default());

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.rule_name == "large_file"));
        assert!(results.iter().any(|r| r.rule_name == "build_artifact"));
    }

    #[test]
    fn test_engine_only_flags_once_per_entry() {
        // Create an engine with overlapping rules
        let mut engine = DetectionEngine::empty();

        // Add two rules that would both match
        engine.add_rule(Box::new(SizeThresholdRule {
            threshold_bytes: 1000,
        }));
        engine.add_rule(Box::new(SizeThresholdRule {
            threshold_bytes: 500,
        }));

        let large_file = create_test_entry("large.bin", 2000);

        let results = engine.analyze(&[large_file], &ScanContext::default());

        // Should only flag once (first rule wins)
        assert_eq!(results.len(), 1, "Should only flag once per entry");
    }

    #[test]
    fn test_custom_rule_integration() {
        struct TestRule;
        impl DetectionRule for TestRule {
            fn name(&self) -> &str {
                "test"
            }
            fn should_flag(&self, _: &FileEntry, _: &ScanContext) -> bool {
                true
            }
            fn reason(&self) -> String {
                "test reason".to_string()
            }
        }

        let mut engine = DetectionEngine::empty();
        engine.add_rule(Box::new(TestRule));

        let entry = create_test_entry("test.txt", 1);
        let results = engine.analyze(&[entry], &ScanContext::default());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_name, "test");
        assert_eq!(results[0].reason, "test reason");
    }

    #[test]
    fn test_empty_engine() {
        let engine = DetectionEngine::empty();

        assert_eq!(engine.rule_count(), 0);

        let entries = vec![
            create_test_entry("file.txt", 1_000_000_000),
            create_test_entry_dir("target"),
        ];

        let results = engine.analyze(&entries, &ScanContext::default());

        // No rules, so nothing should be flagged
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_default_engine_has_rules() {
        let engine = DetectionEngine::default();

        assert_eq!(engine.rule_count(), 2); // SizeThreshold + BuildArtifact
    }

    #[test]
    fn test_detection_result_contains_entry() {
        let engine = DetectionEngine::new();

        let entry = create_test_entry_dir("target");
        let results = engine.analyze(std::slice::from_ref(&entry), &ScanContext::default());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.path, entry.path);
    }

    #[test]
    fn test_analyze_empty_entries() {
        let engine = DetectionEngine::new();
        let results = engine.analyze(&[], &ScanContext::default());

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_rule_priority() {
        // First rule should win when multiple match
        let mut engine = DetectionEngine::empty();

        struct FirstRule;
        impl DetectionRule for FirstRule {
            fn name(&self) -> &str {
                "first"
            }
            fn should_flag(&self, _: &FileEntry, _: &ScanContext) -> bool {
                true
            }
            fn reason(&self) -> String {
                "first".to_string()
            }
        }

        struct SecondRule;
        impl DetectionRule for SecondRule {
            fn name(&self) -> &str {
                "second"
            }
            fn should_flag(&self, _: &FileEntry, _: &ScanContext) -> bool {
                true
            }
            fn reason(&self) -> String {
                "second".to_string()
            }
        }

        engine.add_rule(Box::new(FirstRule));
        engine.add_rule(Box::new(SecondRule));

        let entry = create_test_entry("test.txt", 100);
        let results = engine.analyze(&[entry], &ScanContext::default());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_name, "first"); // First rule wins
    }
}
