//! Performance tests for large-scale file operations.
//!
//! These tests are marked with #[ignore] to prevent them from running
//! in normal test runs. Run them explicitly with:
//!
//! ```bash
//! cargo test --test performance_tests -- --ignored --nocapture
//! ```

use megamaid::detector::engine::{DetectionEngine, DetectionResult, ScanContext};
use megamaid::models::cleanup_plan::CleanupPlan;
use megamaid::models::file_entry::{EntryType, FileEntry};
use megamaid::planner::generator::PlanGenerator;
use megamaid::scanner::traversal::{FileScanner, ScanConfig};
use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};
use tempfile::TempDir;

/// Helper to create a large number of test files
fn create_test_files(base: &TempDir, count: usize) {
    println!("Creating {} test files...", count);
    let start = Instant::now();

    // Create files in batches of 1000 per directory to avoid
    // hitting directory size limits
    const FILES_PER_DIR: usize = 1000;

    for i in 0..count {
        let dir_num = i / FILES_PER_DIR;
        let dir_path = base.path().join(format!("batch_{}", dir_num));

        if i % FILES_PER_DIR == 0 {
            fs::create_dir_all(&dir_path).unwrap();
        }

        let file_path = dir_path.join(format!("file_{}.txt", i));
        fs::write(file_path, "test content").unwrap();

        // Progress indicator
        if (i + 1) % 10000 == 0 {
            println!("  Created {}/{} files...", i + 1, count);
        }
    }

    println!(
        "File creation complete in {:.2}s",
        start.elapsed().as_secs_f64()
    );
}

/// Creates mock FileEntry objects without filesystem access
fn create_mock_entries(count: usize) -> Vec<FileEntry> {
    (0..count)
        .map(|i| FileEntry {
            path: PathBuf::from(format!("file_{}.txt", i)),
            size: 1024,
            modified: SystemTime::now(),
            entry_type: EntryType::File,
            file_id: None,
        })
        .collect()
}

#[test]
#[ignore] // Run with: cargo test --test performance_tests -- --ignored
fn test_scan_10k_files_performance() {
    let temp = TempDir::new().unwrap();
    create_test_files(&temp, 10_000);

    println!("Starting scan of 10,000 files...");
    let start = Instant::now();

    let scanner = FileScanner::new(ScanConfig::default());
    let results = scanner.scan(temp.path()).unwrap();

    let duration = start.elapsed();
    let files_scanned = results.len();

    println!("Scanned {} files in {:?}", files_scanned, duration);
    println!(
        "Performance: {:.0} files/second",
        files_scanned as f64 / duration.as_secs_f64()
    );

    // Should complete in reasonable time
    assert!(
        duration.as_secs() < 10,
        "Should scan 10K files in <10s, took {:?}",
        duration
    );
    assert!(files_scanned >= 10_000);
}

#[test]
#[ignore] // Run with: cargo test --test performance_tests -- --ignored --nocapture
fn test_scan_100k_files_performance() {
    let temp = TempDir::new().unwrap();
    create_test_files(&temp, 100_000);

    println!("\nStarting scan of 100,000 files...");
    let start = Instant::now();

    let scanner = FileScanner::new(ScanConfig::default());
    let results = scanner.scan(temp.path()).unwrap();

    let duration = start.elapsed();
    let files_scanned = results.len();

    println!("Scanned {} files in {:?}", files_scanned, duration);
    println!(
        "Performance: {:.0} files/second",
        files_scanned as f64 / duration.as_secs_f64()
    );

    // Memory usage check (approximate)
    let entry_size = std::mem::size_of::<FileEntry>();
    let total_memory = entry_size * files_scanned;
    println!(
        "Approximate memory usage: {:.2} MB",
        total_memory as f64 / 1_048_576.0
    );

    // Performance target: should complete in <60 seconds
    assert!(
        duration.as_secs() < 60,
        "Should scan 100K files in <60s, took {:?}",
        duration
    );

    // Memory target: <100MB for 100K files
    assert!(
        total_memory < 100 * 1_048_576,
        "Memory usage should be <100MB, was {:.2}MB",
        total_memory as f64 / 1_048_576.0
    );

    assert!(files_scanned >= 100_000);
}

#[test]
#[ignore]
fn test_detection_100k_entries_performance() {
    println!("\nCreating 100,000 mock entries...");
    let entries = create_mock_entries(100_000);

    println!("Running detection on 100,000 entries...");
    let start = Instant::now();

    let engine = DetectionEngine::new();
    let detections = engine.analyze(&entries, &ScanContext::default());

    let duration = start.elapsed();

    println!("Analyzed {} entries in {:?}", entries.len(), duration);
    println!("Found {} detections", detections.len());
    println!(
        "Performance: {:.0} entries/second",
        entries.len() as f64 / duration.as_secs_f64()
    );

    // Detection should be very fast (<5 seconds for 100K entries)
    assert!(
        duration.as_secs() < 5,
        "Should analyze 100K entries in <5s, took {:?}",
        duration
    );
}

#[test]
#[ignore]
fn test_plan_generation_100k_entries() {
    println!("\nCreating 100,000 mock detection results...");
    let entries = create_mock_entries(100_000);
    let detections: Vec<DetectionResult> = entries
        .into_iter()
        .map(|e| DetectionResult {
            entry: e,
            rule_name: "test".to_string(),
            reason: "test".to_string(),
        })
        .collect();

    println!("Generating plan from 100,000 detections...");
    let start = Instant::now();

    let generator = PlanGenerator::new(PathBuf::from("/test"));
    let plan = generator.generate(detections);

    let duration = start.elapsed();

    println!(
        "Generated plan for {} entries in {:?}",
        plan.entries.len(),
        duration
    );

    // Should be very fast (<5 seconds)
    assert!(
        duration.as_secs() < 5,
        "Should generate plan in <5s, took {:?}",
        duration
    );
    assert_eq!(plan.entries.len(), 100_000);
}

#[test]
#[ignore]
fn test_plan_generation_1m_entries() {
    println!("\nCreating 1,000,000 mock detection results...");
    let start = Instant::now();

    let entries = create_mock_entries(1_000_000);
    let detections: Vec<DetectionResult> = entries
        .into_iter()
        .map(|e| DetectionResult {
            entry: e,
            rule_name: "test".to_string(),
            reason: "test".to_string(),
        })
        .collect();

    println!("Mock data created in {:.2}s", start.elapsed().as_secs_f64());

    println!("Generating plan from 1,000,000 detections...");
    let start = Instant::now();

    let generator = PlanGenerator::new(PathBuf::from("/test"));
    let plan = generator.generate(detections);

    let duration = start.elapsed();

    println!(
        "Generated plan for {} entries in {:?}",
        plan.entries.len(),
        duration
    );

    // Target: <30 seconds for 1M entries
    assert!(
        duration.as_secs() < 30,
        "Should generate plan in <30s, took {:?}",
        duration
    );
    assert_eq!(plan.entries.len(), 1_000_000);
}

#[test]
#[ignore]
fn test_yaml_serialization_performance() {
    use megamaid::models::cleanup_plan::{CleanupAction, CleanupEntry};

    println!("\nCreating 100,000 entry plan...");

    let mut plan = CleanupPlan::new(PathBuf::from("/test"));

    for i in 0..100_000 {
        plan.add_entry(CleanupEntry::new(
            format!("file_{}.txt", i),
            1024,
            "2025-11-19T12:00:00Z".to_string(),
            CleanupAction::Delete,
            "test".to_string(),
            "Test reason".to_string(),
        ));
    }

    println!("Serializing plan to YAML...");
    let start = Instant::now();

    let yaml_string = serde_yaml::to_string(&plan).unwrap();

    let duration = start.elapsed();

    println!(
        "Serialized {} entries in {:?}",
        plan.entries.len(),
        duration
    );
    println!(
        "YAML size: {:.2} MB",
        yaml_string.len() as f64 / 1_048_576.0
    );

    // Target: <5 seconds for 100K entries
    assert!(
        duration.as_secs() < 5,
        "Should serialize in <5s, took {:?}",
        duration
    );

    // Test deserialization too
    println!("Deserializing plan from YAML...");
    let start = Instant::now();

    let _: CleanupPlan = serde_yaml::from_str(&yaml_string).unwrap();

    let duration = start.elapsed();
    println!("Deserialized in {:?}", duration);

    assert!(
        duration.as_secs() < 10,
        "Should deserialize in <10s, took {:?}",
        duration
    );
}

#[test]
fn test_scan_small_dataset() {
    // This is a non-ignored test that serves as a quick sanity check
    let temp = TempDir::new().unwrap();

    // Create 100 files
    for i in 0..100 {
        let file_path = temp.path().join(format!("file_{}.txt", i));
        fs::write(file_path, "test content").unwrap();
    }

    let start = Instant::now();
    let scanner = FileScanner::new(ScanConfig::default());
    let results = scanner.scan(temp.path()).unwrap();
    let duration = start.elapsed();

    println!("Scanned {} files in {:?}", results.len(), duration);

    assert!(results.len() >= 100);
    assert!(duration.as_secs() < 1); // Should be near-instant
}

#[test]
fn test_parallel_scan_performance() {
    // Test parallel scanning performance on realistic dataset
    let temp = TempDir::new().unwrap();
    create_test_files(&temp, 5_000);

    let start = Instant::now();
    let scanner =
        megamaid::scanner::ParallelScanner::new(megamaid::scanner::parallel::ScannerConfig {
            max_depth: None,
            skip_hidden: true,
            follow_symlinks: false,
            thread_count: 4,
        });
    let results = scanner.scan(temp.path()).unwrap();
    let duration = start.elapsed();

    println!("Parallel scanned {} files in {:?}", results.len(), duration);
    println!(
        "Throughput: {:.0} files/sec",
        results.len() as f64 / duration.as_secs_f64()
    );

    assert!(results.len() >= 5_000);
    // Should be significantly faster with parallelization
    assert!(duration.as_secs() < 5, "Should scan 5K files in <5s");
}

#[test]
fn test_complete_pipeline_performance() {
    // Test the entire workflow: scan -> detect -> plan -> verify
    use megamaid::detector::rules::SizeThresholdRule;
    use megamaid::detector::BuildArtifactRule;
    use megamaid::verifier::{VerificationConfig, VerificationEngine};

    let temp = TempDir::new().unwrap();

    // Create realistic project structure
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::create_dir_all(temp.path().join("target/debug")).unwrap();
    fs::create_dir_all(temp.path().join("node_modules/package1")).unwrap();

    // Create many files
    for i in 0..1_000 {
        fs::write(
            temp.path().join(format!("src/file_{}.rs", i)),
            "fn main() {}",
        )
        .unwrap();
    }
    for i in 0..500 {
        fs::write(
            temp.path().join(format!("target/debug/artifact_{}.o", i)),
            vec![0u8; 1000],
        )
        .unwrap();
    }

    let total_start = Instant::now();

    // Step 1: Scan
    let scan_start = Instant::now();
    let scanner = FileScanner::new(ScanConfig::default());
    let entries = scanner.scan(temp.path()).unwrap();
    let scan_duration = scan_start.elapsed();
    println!("Scan: {:?} ({} entries)", scan_duration, entries.len());

    // Step 2: Detect
    let detect_start = Instant::now();
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(BuildArtifactRule::default()));
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: 100,
    }));
    let detections = engine.analyze(&entries, &ScanContext::default());
    let detect_duration = detect_start.elapsed();
    println!(
        "Detect: {:?} ({} detections)",
        detect_duration,
        detections.len()
    );

    // Step 3: Generate plan
    let plan_start = Instant::now();
    let generator = PlanGenerator::new(temp.path().to_path_buf());
    let plan = generator.generate(detections);
    let plan_duration = plan_start.elapsed();
    println!(
        "Plan generation: {:?} ({} entries)",
        plan_duration,
        plan.entries.len()
    );

    // Step 4: Verify
    let verify_start = Instant::now();
    let verifier = VerificationEngine::new(VerificationConfig::default());
    let verification = verifier.verify(&plan).unwrap();
    let verify_duration = verify_start.elapsed();
    println!("Verify: {:?}", verify_duration);

    let total_duration = total_start.elapsed();
    println!("Total pipeline: {:?}", total_duration);

    assert!(verification.is_safe_to_execute());
    assert!(
        total_duration.as_secs() < 5,
        "Complete pipeline should finish in <5s"
    );
}

#[test]
fn test_memory_efficiency() {
    // Test memory efficiency with moderate dataset
    // Creates 5,000 entries and verifies memory usage stays reasonable

    let entries = create_mock_entries(5_000);

    // Run detection
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(megamaid::detector::rules::SizeThresholdRule {
        threshold_bytes: 1_048_576, // 1 MB
    }));

    let detections = engine.analyze(&entries, &ScanContext::default());

    // Generate plan
    let generator = PlanGenerator::new(PathBuf::from("/test"));
    let plan = generator.generate(detections);

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&plan).unwrap();

    // Verify sizes are reasonable
    let entries_size_estimate = entries.len() * std::mem::size_of::<FileEntry>();
    let plan_size_estimate =
        plan.entries.len() * std::mem::size_of::<megamaid::models::CleanupEntry>();

    println!(
        "Entries memory estimate: ~{} KB",
        entries_size_estimate / 1024
    );
    println!("Plan memory estimate: ~{} KB", plan_size_estimate / 1024);
    println!("YAML size: {} KB", yaml.len() / 1024);

    // For 5K entries, memory should be well under 10MB
    assert!(
        entries_size_estimate < 10_000_000,
        "Entries should use <10MB"
    );
    assert!(yaml.len() < 5_000_000, "YAML should be <5MB for 5K entries");
}
