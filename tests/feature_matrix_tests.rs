//! Feature combination tests that verify multiple features work together.
//!
//! These tests check that combinations of scanner options, detection rules,
//! and plan generation features interact correctly.

use megamaid::detector::engine::{DetectionEngine, ScanContext};
use megamaid::detector::rules::{BuildArtifactRule, SizeThresholdRule};
use megamaid::planner::generator::PlanGenerator;
use megamaid::planner::writer::PlanWriter;
use megamaid::scanner::traversal::{FileScanner, ScanConfig};
use std::fs;
use tempfile::TempDir;

/// Helper to create a project with both visible and hidden artifacts
fn create_project_with_hidden_artifacts(temp: &TempDir) {
    let base = temp.path();

    // Visible build artifacts
    fs::create_dir_all(base.join("target/debug")).unwrap();
    fs::write(base.join("target/debug/myapp.exe"), vec![0u8; 1000]).unwrap();

    fs::create_dir_all(base.join("node_modules")).unwrap();
    fs::write(base.join("node_modules/package.json"), "{}").unwrap();

    // Hidden build artifacts
    fs::create_dir_all(base.join(".next")).unwrap();
    fs::write(base.join(".next/build-manifest.json"), "{}").unwrap();

    // Normal source files
    fs::create_dir_all(base.join("src")).unwrap();
    fs::write(base.join("src/main.rs"), "fn main() {}").unwrap();

    // Hidden source files (should be skipped if skip_hidden=true)
    fs::write(base.join(".env"), "SECRET=value").unwrap();
}

/// Helper to create nested structure with large files at different depths
fn create_deep_structure_with_large_files(temp: &TempDir) {
    let base = temp.path();

    // Large file at depth 1
    fs::write(base.join("large1.bin"), vec![0u8; 15_000_000]).unwrap();

    // Large file at depth 2
    fs::create_dir_all(base.join("level1")).unwrap();
    fs::write(base.join("level1/large2.bin"), vec![0u8; 20_000_000]).unwrap();

    // Large file at depth 3
    fs::create_dir_all(base.join("level1/level2")).unwrap();
    fs::write(base.join("level1/level2/large3.bin"), vec![0u8; 25_000_000]).unwrap();

    // Large file at depth 4 (beyond typical max_depth)
    fs::create_dir_all(base.join("level1/level2/level3")).unwrap();
    fs::write(
        base.join("level1/level2/level3/large4.bin"),
        vec![0u8; 30_000_000],
    )
    .unwrap();

    // Small files at various depths
    fs::write(base.join("small.txt"), "small").unwrap();
    fs::write(base.join("level1/small.txt"), "small").unwrap();
}

#[test]
fn test_scan_skip_hidden_with_build_artifacts() {
    // Combination: skip hidden files + detect build artifacts
    let temp = TempDir::new().unwrap();
    create_project_with_hidden_artifacts(&temp);

    let config = ScanConfig {
        skip_hidden: true,
        ..Default::default()
    };
    let scanner = FileScanner::new(config);
    let entries = scanner.scan(temp.path()).unwrap();

    let engine = DetectionEngine::new();
    let detections = engine.analyze(&entries, &ScanContext::default());

    // Should find visible build artifacts
    assert!(detections
        .iter()
        .any(|d| d.entry.path.to_string_lossy().contains("target")));
    assert!(detections
        .iter()
        .any(|d| d.entry.path.to_string_lossy().contains("node_modules")));

    // Should NOT find hidden build artifacts (because skip_hidden=true)
    assert!(!detections
        .iter()
        .any(|d| d.entry.path.to_string_lossy().contains(".next")));
}

#[test]
fn test_scan_include_hidden_with_build_artifacts() {
    // Combination: include hidden files + detect build artifacts
    let temp = TempDir::new().unwrap();
    create_project_with_hidden_artifacts(&temp);

    let config = ScanConfig {
        skip_hidden: false,
        ..Default::default()
    };
    let scanner = FileScanner::new(config);
    let entries = scanner.scan(temp.path()).unwrap();

    let engine = DetectionEngine::new();
    let detections = engine.analyze(&entries, &ScanContext::default());

    // Should find both visible and hidden build artifacts
    assert!(detections
        .iter()
        .any(|d| d.entry.path.to_string_lossy().contains("target")));
    assert!(detections
        .iter()
        .any(|d| d.entry.path.to_string_lossy().contains(".next")));
}

#[test]
fn test_max_depth_with_size_threshold() {
    // Combination: max depth + size threshold
    let temp = TempDir::new().unwrap();
    create_deep_structure_with_large_files(&temp);

    let config = ScanConfig {
        max_depth: Some(3),
        ..Default::default()
    };
    let scanner = FileScanner::new(config);
    let entries = scanner.scan(temp.path()).unwrap();

    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: 10_000_000, // 10MB
    }));

    let detections = engine.analyze(&entries, &ScanContext::default());

    // Should only find large files within depth 3
    for detection in &detections {
        let components = detection
            .entry
            .path
            .strip_prefix(temp.path())
            .unwrap()
            .components()
            .count();
        assert!(
            components <= 3,
            "Found file at depth {} (expected <= 3): {}",
            components,
            detection.entry.path.display()
        );
    }

    // Verify we found some large files
    assert!(
        detections.len() >= 2,
        "Should find at least 2 large files within depth 3"
    );

    // Should NOT find large4.bin (at depth 4)
    assert!(!detections
        .iter()
        .any(|d| d.entry.path.to_string_lossy().contains("large4")));
}

#[test]
fn test_multiple_detection_rules_with_plan_generation() {
    // Combination: multiple rules + plan generation
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create build artifacts
    fs::create_dir_all(base.join("target")).unwrap();
    fs::write(base.join("target/debug.exe"), vec![0u8; 1000]).unwrap();

    // Create large files
    fs::write(base.join("video.mp4"), vec![0u8; 200_000_000]).unwrap();

    // Create normal files
    fs::write(base.join("README.md"), "# Project").unwrap();

    // Scan
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(base).unwrap();

    // Detect with multiple rules
    let engine = DetectionEngine::new();
    let detections = engine.analyze(&entries, &ScanContext::default());

    // Generate plan
    let generator = PlanGenerator::new(base.to_path_buf());
    let plan = generator.generate(detections);

    // Verify plan contains both types of detections
    assert!(plan.entries.iter().any(|e| e.rule_name == "build_artifact"));
    assert!(plan.entries.iter().any(|e| e.rule_name == "large_file"));

    // Verify actions are assigned correctly
    for entry in &plan.entries {
        if entry.rule_name == "build_artifact" {
            assert_eq!(
                entry.action,
                megamaid::models::CleanupAction::Delete,
                "Build artifacts should default to Delete"
            );
        } else if entry.rule_name == "large_file" {
            assert_eq!(
                entry.action,
                megamaid::models::CleanupAction::Review,
                "Large files should default to Review"
            );
        }
    }
}

#[test]
fn test_end_to_end_scan_detect_plan_write() {
    // Full workflow: scan + detect + generate + write + verify
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create test structure
    fs::create_dir_all(base.join("target")).unwrap();
    fs::write(base.join("target/app"), vec![0u8; 5000]).unwrap();
    fs::create_dir_all(base.join("node_modules/lodash")).unwrap();
    fs::write(
        base.join("node_modules/lodash/index.js"),
        "module.exports = {}",
    )
    .unwrap();
    fs::write(base.join("large.zip"), vec![0u8; 150_000_000]).unwrap();

    // Step 1: Scan
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(base).unwrap();
    assert!(entries.len() >= 3);

    // Step 2: Detect
    let engine = DetectionEngine::new();
    let detections = engine.analyze(&entries, &ScanContext::default());
    assert!(detections.len() >= 2); // At least target/ and node_modules/

    // Step 3: Generate plan
    let generator = PlanGenerator::new(base.to_path_buf());
    let plan = generator.generate(detections);

    // Step 4: Write plan
    let plan_path = base.join("cleanup-plan.toml");
    PlanWriter::write(&plan, &plan_path).unwrap();
    assert!(plan_path.exists());

    // Step 5: Verify by reading back
    let content = fs::read_to_string(&plan_path).unwrap();
    let loaded: megamaid::models::CleanupPlan = toml::from_str(&content).unwrap();

    assert_eq!(loaded.entries.len(), plan.entries.len());
    assert_eq!(loaded.base_path, plan.base_path);
}

#[test]
fn test_custom_size_threshold_with_plan_filtering() {
    // Combination: custom size threshold + selective detection
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create files of various sizes
    fs::write(base.join("tiny.txt"), vec![0u8; 1000]).unwrap(); // 1KB
    fs::write(base.join("small.dat"), vec![0u8; 1_000_000]).unwrap(); // 1MB
    fs::write(base.join("medium.bin"), vec![0u8; 50_000_000]).unwrap(); // 50MB
    fs::write(base.join("large.iso"), vec![0u8; 150_000_000]).unwrap(); // 150MB

    // Scan
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(base).unwrap();

    // Detect with custom 100MB threshold
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: 100 * 1_048_576, // 100MB
    }));

    let detections = engine.analyze(&entries, &ScanContext::default());

    // Should only flag the 150MB file
    assert_eq!(detections.len(), 1, "Should only detect one file > 100MB");
    assert!(detections[0]
        .entry
        .path
        .to_string_lossy()
        .contains("large.iso"));
}

#[test]
fn test_skip_hidden_with_max_depth() {
    // Combination: skip hidden + max depth
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create nested structure with hidden files
    fs::create_dir_all(base.join("level1/level2")).unwrap();
    fs::write(base.join(".hidden1"), "secret").unwrap();
    fs::write(base.join("level1/.hidden2"), "secret").unwrap();
    fs::write(base.join("level1/level2/.hidden3"), "secret").unwrap();
    fs::write(base.join("level1/visible.txt"), "public").unwrap(); // At depth 2
    fs::write(base.join("level1/level2/deep.txt"), "deep file").unwrap(); // At depth 3, should not be found

    let config = ScanConfig {
        skip_hidden: true,
        max_depth: Some(2),
        ..Default::default()
    };
    let scanner = FileScanner::new(config);
    let entries = scanner.scan(base).unwrap();

    // Should not find any hidden files
    assert!(!entries.iter().any(|e| {
        e.path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }));

    // Should find visible.txt (at depth 2)
    assert!(entries
        .iter()
        .any(|e| e.path.to_string_lossy().contains("visible.txt")));

    // Should NOT find deep.txt (at depth 3)
    assert!(!entries
        .iter()
        .any(|e| e.path.to_string_lossy().contains("deep.txt")));
}

#[test]
fn test_build_artifact_detection_with_custom_patterns() {
    // Verify build artifact rule works with multiple pattern types
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create various build artifacts
    fs::create_dir_all(base.join("target")).unwrap();
    fs::create_dir_all(base.join("node_modules")).unwrap();
    fs::create_dir_all(base.join("dist")).unwrap();
    fs::create_dir_all(base.join("__pycache__")).unwrap();

    // Scan
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(base).unwrap();

    // Detect with build artifact rule
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(BuildArtifactRule::default()));

    let detections = engine.analyze(&entries, &ScanContext::default());

    // Should detect all build artifact directories
    assert!(
        detections.len() >= 4,
        "Should detect at least 4 build artifact directories"
    );

    let detected_names: Vec<String> = detections
        .iter()
        .map(|d| {
            d.entry
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    assert!(detected_names.contains(&"target".to_string()));
    assert!(detected_names.contains(&"node_modules".to_string()));
    assert!(detected_names.contains(&"dist".to_string()));
    assert!(detected_names.contains(&"__pycache__".to_string()));
}
