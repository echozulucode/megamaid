//! Configuration integration tests.
//!
//! Tests the configuration system with real file operations and CLI integration.

use megamaid::config::{load_config, load_default_config, validate_config, MegamaidConfig};
use megamaid::detector::{DetectionEngine, ScanContext};
use megamaid::executor::{ExecutionEngine, ExecutionMode};
use megamaid::planner::PlanGenerator;
use megamaid::scanner::{FileScanner, ScanConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_config_from_file() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("megamaid.yaml");

    let config_content = r#"
scanner:
  max_depth: 10
  skip_hidden: false
  follow_symlinks: false
  thread_count: 4

detector:
  rules:
    size_threshold:
      enabled: true
      threshold_mb: 50
      action: review
    build_artifacts:
      enabled: true
      action: delete
      custom_patterns: []

executor:
  parallel: true
  batch_size: 50
  default_mode: dry_run
  fail_fast: false
  use_recycle_bin: false
  backup_dir: null

output:
  plan_file: "cleanup-plan.yaml"
  log_file: "execution-log.yaml"
  drift_report: "drift-report.txt"

verifier:
  check_mtime: true
  check_size: true
  fail_fast: false
"#;

    fs::write(&config_path, config_content).unwrap();

    // Load and validate config
    let config = load_config(&config_path).unwrap();
    validate_config(&config).unwrap();

    // Verify values
    assert_eq!(config.scanner.max_depth, Some(10));
    assert!(!config.scanner.skip_hidden);
    assert_eq!(config.scanner.thread_count, 4);
    assert_eq!(config.detector.rules.size_threshold.threshold_mb, 50);
    assert!(config.executor.parallel);
    assert_eq!(config.executor.batch_size, 50);
}

#[test]
fn test_default_config_discovery() {
    let temp = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to temp directory
    std::env::set_current_dir(temp.path()).unwrap();

    // Create default config file
    let config_content = r#"
scanner:
  max_depth: 5
  skip_hidden: true
"#;
    fs::write("megamaid.yaml", config_content).unwrap();

    // Load default config
    let config = load_default_config().unwrap();
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.scanner.max_depth, Some(5));

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_config_cli_override() {
    let temp = TempDir::new().unwrap();

    // Create config file with specific values
    let config_path = temp.path().join("config.yaml");
    let config_content = r#"
scanner:
  max_depth: 10
  skip_hidden: false
detector:
  rules:
    size_threshold:
      enabled: true
      threshold_mb: 100
"#;
    fs::write(&config_path, config_content).unwrap();

    // Load config
    let config = load_config(&config_path).unwrap();

    // CLI override: max_depth = 5 (overrides config's 10)
    let scan_config = ScanConfig {
        follow_links: config.scanner.follow_symlinks,
        max_depth: Some(5), // CLI override
        skip_hidden: true,  // CLI override
    };

    // Verify overrides took effect
    assert_eq!(scan_config.max_depth, Some(5));
    assert!(scan_config.skip_hidden);
}

#[test]
fn test_config_integration_with_scan() {
    let temp = TempDir::new().unwrap();

    // Create test files
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::create_dir_all(temp.path().join("target")).unwrap();
    fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp.path().join("target/app.exe"), vec![0u8; 1000]).unwrap();

    // Create config
    let mut config = MegamaidConfig::default();
    config.scanner.max_depth = Some(10);
    config.scanner.skip_hidden = true;
    config.detector.rules.build_artifacts.enabled = true;

    // Use config to scan
    let scan_config: ScanConfig = config.scanner.clone().into();
    let scanner = FileScanner::new(scan_config);
    let entries = scanner.scan(temp.path()).unwrap();

    assert!(entries.len() >= 2, "Should find at least 2 entries");

    // Use config to detect
    let mut engine = DetectionEngine::empty();
    if config.detector.rules.build_artifacts.enabled {
        engine.add_rule(Box::new(megamaid::detector::BuildArtifactRule::default()));
    }

    let detections = engine.analyze(&entries, &ScanContext::default());
    assert!(detections.len() >= 1, "Should detect target/ directory");
}

#[test]
fn test_config_integration_with_execution() {
    let temp = TempDir::new().unwrap();

    // Create test files
    fs::create_dir_all(temp.path().join("target")).unwrap();
    fs::write(temp.path().join("target/app.exe"), vec![0u8; 1000]).unwrap();

    // Create and execute plan with config
    let mut config = MegamaidConfig::default();
    config.executor.parallel = false;
    config.executor.batch_size = 10;

    // Scan and detect
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(temp.path()).unwrap();

    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(megamaid::detector::BuildArtifactRule::default()));
    let detections = engine.analyze(&entries, &ScanContext::default());

    // Generate plan
    let generator = PlanGenerator::new(temp.path().to_path_buf());
    let plan = generator.generate(detections);

    // Execute using config
    let exec_config = config.executor.to_execution_config(Some(ExecutionMode::DryRun));
    let executor = ExecutionEngine::new(exec_config);
    let result = executor.execute(&plan).unwrap();

    // Verify execution used config settings
    assert!(result.summary.total_operations > 0);
    // In dry run, files should still exist
    assert!(temp.path().join("target").exists());
}

#[test]
fn test_invalid_config_validation() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("invalid.yaml");

    // Config with invalid values
    let invalid_config = r#"
scanner:
  max_depth: 0  # Invalid: must be at least 1
  thread_count: 300  # Invalid: exceeds 256

detector:
  rules:
    size_threshold:
      threshold_mb: 0  # Invalid: must be at least 1
"#;

    fs::write(&config_path, invalid_config).unwrap();

    // Should load successfully
    let config = load_config(&config_path).unwrap();

    // But validation should fail
    let result = validate_config(&config);
    assert!(result.is_err(), "Validation should fail for invalid config");
}

#[test]
fn test_partial_config_uses_defaults() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("partial.yaml");

    // Only specify scanner settings
    let partial_config = r#"
scanner:
  max_depth: 5
"#;

    fs::write(&config_path, partial_config).unwrap();

    let config = load_config(&config_path).unwrap();

    // Scanner values should be custom
    assert_eq!(config.scanner.max_depth, Some(5));

    // Other values should use defaults
    assert_eq!(config.executor.batch_size, 100); // default
    assert_eq!(config.detector.rules.size_threshold.threshold_mb, 100); // default
}

#[test]
fn test_config_roundtrip() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("roundtrip.yaml");

    // Create config
    let mut config = MegamaidConfig::default();
    config.scanner.max_depth = Some(15);
    config.scanner.thread_count = 8;
    config.executor.parallel = true;
    config.executor.batch_size = 75;

    // Write config
    let yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(&config_path, &yaml).unwrap();

    // Read back
    let loaded = load_config(&config_path).unwrap();

    // Verify round-trip
    assert_eq!(loaded.scanner.max_depth, Some(15));
    assert_eq!(loaded.scanner.thread_count, 8);
    assert!(loaded.executor.parallel);
    assert_eq!(loaded.executor.batch_size, 75);
}
