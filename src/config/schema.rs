//! Configuration schema definitions.

use crate::models::CleanupAction;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct MegamaidConfig {
    /// Scanner configuration
    pub scanner: ScannerConfig,

    /// Detector configuration
    pub detector: DetectorConfig,

    /// Executor configuration
    pub executor: ExecutorConfig,

    /// Output configuration
    pub output: OutputConfig,

    /// Verifier configuration
    pub verifier: VerifierConfig,
}

/// Scanner configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ScannerConfig {
    /// Maximum directory depth to scan (None = unlimited)
    pub max_depth: Option<usize>,

    /// Skip hidden files and directories
    pub skip_hidden: bool,

    /// Follow symbolic links
    pub follow_symlinks: bool,

    /// Number of threads for parallel scanning (0 = auto-detect)
    pub thread_count: usize,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            skip_hidden: true,
            follow_symlinks: false,
            thread_count: 0,
        }
    }
}

impl From<ScannerConfig> for crate::scanner::traversal::ScanConfig {
    fn from(config: ScannerConfig) -> Self {
        Self {
            follow_links: config.follow_symlinks,
            max_depth: config.max_depth,
            skip_hidden: config.skip_hidden,
        }
    }
}

impl From<ScannerConfig> for crate::scanner::parallel::ScannerConfig {
    fn from(config: ScannerConfig) -> Self {
        Self {
            max_depth: config.max_depth,
            skip_hidden: config.skip_hidden,
            follow_symlinks: config.follow_symlinks,
            thread_count: config.thread_count,
        }
    }
}

/// Detector configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DetectorConfig {
    /// Built-in rules configuration
    pub rules: BuiltInRulesConfig,

    /// Custom detection rules
    pub custom_rules: Vec<CustomRule>,
}

/// Built-in rules configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct BuiltInRulesConfig {
    /// Size threshold rule configuration
    pub size_threshold: SizeThresholdConfig,

    /// Build artifacts rule configuration
    pub build_artifacts: BuildArtifactsConfig,
}

/// Size threshold rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SizeThresholdConfig {
    /// Enable this rule
    pub enabled: bool,

    /// Threshold in megabytes
    pub threshold_mb: u64,

    /// Default action for flagged files
    pub action: CleanupAction,
}

impl Default for SizeThresholdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_mb: 100,
            action: CleanupAction::Review,
        }
    }
}

/// Build artifacts rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BuildArtifactsConfig {
    /// Enable this rule
    pub enabled: bool,

    /// Default action for flagged directories
    pub action: CleanupAction,

    /// Custom patterns in addition to defaults
    pub custom_patterns: Vec<String>,
}

impl Default for BuildArtifactsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            action: CleanupAction::Delete,
            custom_patterns: Vec::new(),
        }
    }
}

/// Custom detection rule definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomRule {
    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Glob pattern to match (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// File extensions to match (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Minimum age in days (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_age_days: Option<u64>,

    /// Minimum size in megabytes (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_size_mb: Option<u64>,

    /// Action to apply
    pub action: CleanupAction,
}

/// Executor configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ExecutorConfig {
    /// Enable parallel execution by default
    pub parallel: bool,

    /// Batch size for parallel processing
    pub batch_size: usize,

    /// Default execution mode
    pub default_mode: ExecutionModeConfig,

    /// Stop on first error
    pub fail_fast: bool,

    /// Use recycle bin by default
    pub use_recycle_bin: bool,

    /// Default backup directory (None = no backup)
    pub backup_dir: Option<PathBuf>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            parallel: false,
            batch_size: 100,
            default_mode: ExecutionModeConfig::DryRun,
            fail_fast: false,
            use_recycle_bin: false,
            backup_dir: None,
        }
    }
}

/// Execution mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionModeConfig {
    /// Dry run mode (simulate)
    DryRun,
    /// Interactive mode (prompt for each)
    Interactive,
    /// Batch mode (execute all)
    Batch,
}

impl From<ExecutionModeConfig> for crate::executor::ExecutionMode {
    fn from(mode: ExecutionModeConfig) -> Self {
        match mode {
            ExecutionModeConfig::DryRun => crate::executor::ExecutionMode::DryRun,
            ExecutionModeConfig::Interactive => crate::executor::ExecutionMode::Interactive,
            ExecutionModeConfig::Batch => crate::executor::ExecutionMode::Batch,
        }
    }
}

impl ExecutorConfig {
    /// Converts to ExecutionConfig, optionally overriding mode.
    pub fn to_execution_config(
        &self,
        mode_override: Option<crate::executor::ExecutionMode>,
    ) -> crate::executor::ExecutionConfig {
        crate::executor::ExecutionConfig {
            mode: mode_override.unwrap_or_else(|| self.default_mode.clone().into()),
            backup_dir: self.backup_dir.clone(),
            fail_fast: self.fail_fast,
            use_recycle_bin: self.use_recycle_bin,
            parallel: self.parallel,
            batch_size: self.batch_size,
        }
    }
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct OutputConfig {
    /// Default cleanup plan filename
    pub plan_file: String,

    /// Default transaction log filename
    pub log_file: String,

    /// Default drift report filename
    pub drift_report: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            plan_file: "cleanup-plan.yaml".to_string(),
            log_file: "execution-log.yaml".to_string(),
            drift_report: "drift-report.txt".to_string(),
        }
    }
}

/// Verifier configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VerifierConfig {
    /// Check modification time during verification
    pub check_mtime: bool,

    /// Check file size during verification
    pub check_size: bool,

    /// Stop verification on first drift detection
    pub fail_fast: bool,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            check_mtime: true,
            check_size: true,
            fail_fast: false,
        }
    }
}

impl From<VerifierConfig> for crate::verifier::VerificationConfig {
    fn from(config: VerifierConfig) -> Self {
        Self {
            check_mtime: config.check_mtime,
            check_size: config.check_size,
            fail_fast: config.fail_fast,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MegamaidConfig::default();

        assert!(config.scanner.skip_hidden);
        assert!(!config.scanner.follow_symlinks);
        assert_eq!(config.scanner.thread_count, 0);
        assert_eq!(config.scanner.max_depth, None);

        assert!(config.detector.rules.size_threshold.enabled);
        assert_eq!(config.detector.rules.size_threshold.threshold_mb, 100);
        assert!(config.detector.rules.build_artifacts.enabled);

        assert!(!config.executor.parallel);
        assert_eq!(config.executor.batch_size, 100);
        assert!(!config.executor.fail_fast);

        assert_eq!(config.output.plan_file, "cleanup-plan.yaml");
        assert_eq!(config.output.log_file, "execution-log.yaml");

        assert!(config.verifier.check_mtime);
        assert!(config.verifier.check_size);
    }

    #[test]
    fn test_config_serialization() {
        let config = MegamaidConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();

        assert!(yaml.contains("scanner:"));
        assert!(yaml.contains("detector:"));
        assert!(yaml.contains("executor:"));
        assert!(yaml.contains("output:"));
        assert!(yaml.contains("verifier:"));
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
scanner:
  max_depth: 5
  skip_hidden: false
  thread_count: 4
detector:
  rules:
    size_threshold:
      enabled: true
      threshold_mb: 200
executor:
  parallel: true
  batch_size: 50
"#;

        let config: MegamaidConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.scanner.max_depth, Some(5));
        assert!(!config.scanner.skip_hidden);
        assert_eq!(config.scanner.thread_count, 4);
        assert_eq!(config.detector.rules.size_threshold.threshold_mb, 200);
        assert!(config.executor.parallel);
        assert_eq!(config.executor.batch_size, 50);
    }

    #[test]
    fn test_custom_rule_deserialization() {
        let yaml = r#"
name: "old_logs"
description: "Log files older than 30 days"
pattern: "*.log"
min_age_days: 30
action: delete
"#;

        let rule: CustomRule = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(rule.name, "old_logs");
        assert_eq!(rule.description, "Log files older than 30 days");
        assert_eq!(rule.pattern, Some("*.log".to_string()));
        assert_eq!(rule.min_age_days, Some(30));
        assert_eq!(rule.action, CleanupAction::Delete);
    }

    #[test]
    fn test_partial_config() {
        let yaml = r#"
scanner:
  max_depth: 10
"#;

        let config: MegamaidConfig = serde_yaml::from_str(yaml).unwrap();

        // Should use defaults for unspecified fields
        assert_eq!(config.scanner.max_depth, Some(10));
        assert!(config.scanner.skip_hidden); // default
        assert_eq!(config.executor.batch_size, 100); // default
    }

    #[test]
    fn test_execution_mode_serialization() {
        let mode = ExecutionModeConfig::DryRun;
        let yaml = serde_yaml::to_string(&mode).unwrap();
        assert!(yaml.contains("dry_run"));

        let mode = ExecutionModeConfig::Interactive;
        let yaml = serde_yaml::to_string(&mode).unwrap();
        assert!(yaml.contains("interactive"));

        let mode = ExecutionModeConfig::Batch;
        let yaml = serde_yaml::to_string(&mode).unwrap();
        assert!(yaml.contains("batch"));
    }

    #[test]
    fn test_cleanup_action_in_config() {
        let yaml = r#"
enabled: true
threshold_mb: 100
action: delete
"#;

        let config: SizeThresholdConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.action, CleanupAction::Delete);

        let yaml = r#"
enabled: true
threshold_mb: 100
action: review
"#;

        let config: SizeThresholdConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.action, CleanupAction::Review);
    }
}
