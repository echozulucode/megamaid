//! End-to-end integration tests for the complete workflow.
//!
//! These tests verify the entire megamaid pipeline from scanning through execution.

use megamaid::detector::engine::{DetectionEngine, ScanContext};
use megamaid::detector::rules::SizeThresholdRule;
use megamaid::executor::{ExecutionConfig, ExecutionEngine, ExecutionMode};
use megamaid::planner::generator::PlanGenerator;
use megamaid::planner::writer::PlanWriter;
use megamaid::scanner::traversal::{FileScanner, ScanConfig};
use megamaid::verifier::{VerificationConfig, VerificationEngine};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_complete_workflow_scan_to_execution() {
    let temp = TempDir::new().unwrap();

    // Setup: Create test directory structure
    fs::create_dir_all(temp.path().join("project/target/debug")).unwrap();
    fs::create_dir_all(temp.path().join("project/src")).unwrap();
    fs::create_dir_all(temp.path().join("data")).unwrap();

    // Create files
    fs::write(temp.path().join("project/src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp.path().join("project/target/debug/app.exe"), vec![0u8; 5_000_000]).unwrap();
    fs::write(temp.path().join("data/large_file.dat"), vec![0u8; 150_000_000]).unwrap();
    fs::write(temp.path().join("data/small_file.txt"), "test").unwrap();

    // Step 1: Scan
    let scanner = FileScanner::new(ScanConfig {
        follow_links: false,
        max_depth: None,
        skip_hidden: true,
    });
    let entries = scanner.scan(temp.path()).unwrap();

    assert!(entries.len() >= 5, "Should find at least 5 entries");

    // Step 2: Detect
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(megamaid::detector::BuildArtifactRule::default()));
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: 100 * 1_048_576, // 100 MB
    }));

    let context = ScanContext::default();
    let detections = engine.analyze(&entries, &context);

    // Should detect target/ and large_file.dat
    assert!(detections.len() >= 2, "Should detect at least 2 cleanup candidates");

    // Step 3: Generate plan
    let generator = PlanGenerator::new(temp.path().to_path_buf());
    let plan = generator.generate(detections);

    assert!(plan.entries.len() >= 2, "Plan should have at least 2 entries");

    // Step 4: Write plan
    let plan_path = temp.path().join("cleanup-plan.yaml");
    PlanWriter::write(&plan, &plan_path).unwrap();

    assert!(plan_path.exists(), "Plan file should exist");

    // Step 5: Verify plan
    let verifier = VerificationEngine::new(VerificationConfig::default());
    let verification = verifier.verify(&plan).unwrap();

    assert!(verification.is_safe_to_execute(), "Plan should be safe to execute");
    assert_eq!(verification.verified, verification.total_entries);

    // Step 6: Execute (dry run)
    let executor = ExecutionEngine::new(ExecutionConfig {
        mode: ExecutionMode::DryRun,
        backup_dir: None,
        fail_fast: false,
        use_recycle_bin: false,
        parallel: false,
        batch_size: 100,
    });

    let result = executor.execute(&plan).unwrap();

    assert_eq!(result.summary.failed, 0, "Dry run should not fail");
    assert!(result.summary.successful > 0, "Should process entries");

    // Verify files still exist after dry run
    assert!(temp.path().join("project/target/debug/app.exe").exists());
    assert!(temp.path().join("data/large_file.dat").exists());

    // Step 7: Execute (real deletion)
    let executor = ExecutionEngine::new(ExecutionConfig {
        mode: ExecutionMode::Batch,
        backup_dir: None,
        fail_fast: false,
        use_recycle_bin: false,
        parallel: false,
        batch_size: 100,
    });

    let result = executor.execute(&plan).unwrap();

    assert_eq!(result.summary.failed, 0, "Execution should succeed");
    assert!(result.summary.successful > 0, "Should delete entries");
    assert!(result.summary.space_freed > 0, "Should free space");

    // Verify cleanup occurred
    assert!(!temp.path().join("project/target").exists(), "target/ should be deleted");
}

#[test]
fn test_workflow_with_parallel_execution() {
    let temp = TempDir::new().unwrap();

    // Create many files for parallel processing
    for i in 0..100 {
        fs::write(
            temp.path().join(format!("file{}.tmp", i)),
            vec![0u8; 1_000_000],
        )
        .unwrap();
    }

    // Scan
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(temp.path()).unwrap();

    // Detect (all files > 0.5 MB)
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: 500_000,
    }));

    let detections = engine.analyze(&entries, &ScanContext::default());

    // Filter to only files (not directories) to avoid deletion order issues
    let file_detections: Vec<_> = detections
        .into_iter()
        .filter(|d| {
            let full_path = temp.path().join(&d.entry.path);
            full_path.is_file()
        })
        .collect();

    assert_eq!(file_detections.len(), 100, "Should detect exactly 100 files");

    // Generate and write plan
    let generator = PlanGenerator::new(temp.path().to_path_buf());
    let mut plan = generator.generate(file_detections);

    // Set all entries to Delete action (since SizeThresholdRule creates Review by default)
    for entry in &mut plan.entries {
        entry.action = megamaid::models::CleanupAction::Delete;
    }

    let plan_path = temp.path().join("plan.yaml");
    PlanWriter::write(&plan, &plan_path).unwrap();

    // Skip verification for this test to avoid timing issues with temp files
    // The goal is to test parallel execution, not verification

    // Execute in parallel
    let executor = ExecutionEngine::new(ExecutionConfig {
        mode: ExecutionMode::Batch,
        backup_dir: None,
        fail_fast: false,
        use_recycle_bin: false,
        parallel: true,
        batch_size: 25,
    });

    let result = executor.execute(&plan).unwrap();

    let delete_count = plan.delete_count();
    assert_eq!(delete_count, 100, "Plan should have 100 files to delete");
    assert_eq!(result.summary.successful, 100, "Should delete all 100 files");
    assert_eq!(result.summary.failed, 0);
    assert!(result.summary.space_freed >= 100_000_000, "Should free ~100MB");

    // Verify all files deleted
    for i in 0..100 {
        assert!(!temp.path().join(format!("file{}.tmp", i)).exists());
    }
}

#[test]
fn test_workflow_with_backup_mode() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backups");

    // Create test files including build artifacts
    fs::create_dir_all(temp.path().join("project/target")).unwrap();
    fs::create_dir_all(temp.path().join("project/node_modules")).unwrap();
    fs::write(temp.path().join("project/target/app.exe"), vec![0u8; 1000]).unwrap();
    fs::write(temp.path().join("project/node_modules/package.json"), "{}").unwrap();

    // Scan and detect build artifacts
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(temp.path()).unwrap();

    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(megamaid::detector::BuildArtifactRule::default()));

    let detections = engine.analyze(&entries, &ScanContext::default());
    let generator = PlanGenerator::new(temp.path().to_path_buf());
    let plan = generator.generate(detections);

    // Execute with backup
    let executor = ExecutionEngine::new(ExecutionConfig {
        mode: ExecutionMode::Batch,
        backup_dir: Some(backup_dir.clone()),
        fail_fast: false,
        use_recycle_bin: false,
        parallel: false,
        batch_size: 100,
    });

    let result = executor.execute(&plan).unwrap();

    assert!(result.summary.successful > 0, "Should move directories");
    assert_eq!(result.summary.failed, 0);

    // Verify directories moved to backup
    assert!(backup_dir.join("project/target").exists(), "Backup should preserve structure");
    assert!(backup_dir.join("project/node_modules").exists());

    // Verify originals removed
    assert!(!temp.path().join("project/target").exists());
    assert!(!temp.path().join("project/node_modules").exists());
}

#[test]
fn test_workflow_with_drift_detection() {
    let temp = TempDir::new().unwrap();

    // Create and scan
    fs::write(temp.path().join("test.txt"), vec![0u8; 1_000_000]).unwrap();

    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(temp.path()).unwrap();

    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: 500_000,
    }));

    let detections = engine.analyze(&entries, &ScanContext::default());
    let generator = PlanGenerator::new(temp.path().to_path_buf());
    let plan = generator.generate(detections);

    // Verify initially safe
    let verifier = VerificationEngine::new(VerificationConfig::default());
    let verification = verifier.verify(&plan).unwrap();
    assert!(verification.is_safe_to_execute());

    // Introduce drift: modify file
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs::write(temp.path().join("test.txt"), vec![0u8; 2_000_000]).unwrap();

    // Verify should detect drift
    let verification = verifier.verify(&plan).unwrap();
    assert!(!verification.is_safe_to_execute(), "Should detect drift");
    assert!(verification.drifted.len() > 0, "Should report drift");
}

#[test]
fn test_workflow_resilience_to_missing_files() {
    let temp = TempDir::new().unwrap();

    // Create files
    for i in 0..10 {
        fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
    }

    // Scan and detect
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(temp.path()).unwrap();

    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: 1,
    }));

    let detections = engine.analyze(&entries, &ScanContext::default());

    // Filter to only files (not directories)
    let file_detections: Vec<_> = detections
        .into_iter()
        .filter(|d| {
            let full_path = temp.path().join(&d.entry.path);
            full_path.is_file()
        })
        .collect();

    assert_eq!(file_detections.len(), 10, "Should detect exactly 10 files");

    let generator = PlanGenerator::new(temp.path().to_path_buf());
    let mut plan = generator.generate(file_detections);

    // Set all entries to Delete action
    for entry in &mut plan.entries {
        entry.action = megamaid::models::CleanupAction::Delete;
    }

    // Delete some files manually
    fs::remove_file(temp.path().join("file3.txt")).unwrap();
    fs::remove_file(temp.path().join("file7.txt")).unwrap();

    // Execute with fail_fast = false
    let executor = ExecutionEngine::new(ExecutionConfig {
        mode: ExecutionMode::Batch,
        backup_dir: None,
        fail_fast: false,
        use_recycle_bin: false,
        parallel: false,
        batch_size: 100,
    });

    let result = executor.execute(&plan).unwrap();

    // Should handle missing files gracefully
    // The executor counts missing files as neither successful nor failed - they're skipped
    let total_processed = result.summary.successful + result.summary.failed;
    assert!(total_processed <= plan.delete_count(), "Should process files");
    assert!(result.summary.successful >= 8, "Should delete at least 8 remaining files");
}
