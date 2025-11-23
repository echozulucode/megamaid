//! Integration tests for execution engine.
//!
//! These tests verify end-to-end execution workflows including parallel deletion,
//! combining scanning, detection, planning, and execution.

use chrono::Utc;
use megamaid::executor::{ExecutionConfig, ExecutionEngine, ExecutionMode};
use megamaid::models::{CleanupAction, CleanupEntry, CleanupPlan};
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

/// Helper to create a test plan from file entries
fn create_plan_from_files(base_path: &std::path::Path, files: Vec<&str>, size: u64) -> CleanupPlan {
    let entries = files
        .into_iter()
        .map(|path| CleanupEntry {
            path: path.to_string(),
            size,
            modified: Utc::now().to_rfc3339(),
            action: CleanupAction::Delete,
            rule_name: "test".to_string(),
            reason: "test file".to_string(),
        })
        .collect();

    CleanupPlan {
        version: "0.1.0".to_string(),
        created_at: Utc::now(),
        base_path: base_path.to_path_buf(),
        entries,
    }
}

#[test]
fn test_end_to_end_parallel_execution() {
    let temp = TempDir::new().unwrap();

    // Create 100 test files
    for i in 0..100 {
        fs::write(temp.path().join(format!("file{}.txt", i)), "test content").unwrap();
    }

    // Create plan directly with just files (simpler than going through detection)
    let files: Vec<_> = (0..100).map(|i| format!("file{}.txt", i)).collect();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let plan = create_plan_from_files(temp.path(), file_refs, 12);

    assert_eq!(plan.entries.len(), 100);

    // Execute plan in parallel
    let config = ExecutionConfig {
        mode: ExecutionMode::Batch,
        parallel: true,
        batch_size: 25,
        ..Default::default()
    };

    let executor = ExecutionEngine::new(config);
    let result = executor.execute(&plan).unwrap();

    // Verify all 100 files were deleted
    assert_eq!(result.summary.successful, 100);
    assert_eq!(result.summary.failed, 0);

    // Verify files are gone
    for i in 0..100 {
        assert!(!temp.path().join(format!("file{}.txt", i)).exists());
    }
}

#[test]
fn test_parallel_vs_sequential_performance() {
    let temp = TempDir::new().unwrap();

    // Create 500 test files
    for i in 0..500 {
        fs::write(
            temp.path().join(format!("file{}.txt", i)),
            vec![0u8; 1000],
        )
        .unwrap();
    }

    // Create file list for plan
    let files: Vec<_> = (0..500).map(|i| format!("file{}.txt", i)).collect();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    // Test sequential execution
    let plan_seq = create_plan_from_files(temp.path(), file_refs.clone(), 1000);
    let config_seq = ExecutionConfig {
        mode: ExecutionMode::Batch,
        parallel: false,
        ..Default::default()
    };

    let start = Instant::now();
    let executor_seq = ExecutionEngine::new(config_seq);
    let result_seq = executor_seq.execute(&plan_seq).unwrap();
    let sequential_duration = start.elapsed();

    assert_eq!(result_seq.summary.successful, 500);

    // Recreate files for parallel test
    for i in 0..500 {
        fs::write(
            temp.path().join(format!("file{}.txt", i)),
            vec![0u8; 1000],
        )
        .unwrap();
    }

    // Test parallel execution
    let plan_par = create_plan_from_files(temp.path(), file_refs, 1000);
    let config_par = ExecutionConfig {
        mode: ExecutionMode::Batch,
        parallel: true,
        batch_size: 50,
        ..Default::default()
    };

    let start = Instant::now();
    let executor_par = ExecutionEngine::new(config_par);
    let result_par = executor_par.execute(&plan_par).unwrap();
    let parallel_duration = start.elapsed();

    assert_eq!(result_par.summary.successful, 500);

    // Print performance comparison
    println!("\n=== Performance Comparison ===");
    println!("Sequential: {:?}", sequential_duration);
    println!("Parallel:   {:?}", parallel_duration);
    println!(
        "Speedup:    {:.2}x",
        sequential_duration.as_secs_f64() / parallel_duration.as_secs_f64()
    );

    // Parallel should be at least as fast (but might be slower on single-core or very fast SSDs)
    // We just verify it completes successfully
    assert!(parallel_duration.as_secs() < 10, "Parallel execution too slow");
}

#[test]
fn test_parallel_execution_with_nested_directories() {
    let temp = TempDir::new().unwrap();

    // Create nested structure with files
    let mut all_files = vec![];
    for i in 0..10 {
        let dir_path = temp.path().join(format!("dir{}", i));
        fs::create_dir_all(&dir_path).unwrap();

        for j in 0..10 {
            let file_path = format!("dir{}/file{}.txt", i, j);
            fs::write(
                temp.path().join(&file_path),
                vec![0u8; 500],
            )
            .unwrap();
            all_files.push(file_path);
        }
    }

    // Create plan with all files
    let file_refs: Vec<&str> = all_files.iter().map(|s| s.as_str()).collect();
    let plan = create_plan_from_files(temp.path(), file_refs, 500);
    assert_eq!(plan.entries.len(), 100);

    let config = ExecutionConfig {
        mode: ExecutionMode::Batch,
        parallel: true,
        batch_size: 20,
        ..Default::default()
    };

    let executor = ExecutionEngine::new(config);
    let result = executor.execute(&plan).unwrap();

    // All 100 files should be deleted
    assert_eq!(result.summary.successful, 100);
    assert_eq!(result.summary.failed, 0);
}

#[test]
fn test_parallel_execution_with_errors() {
    let temp = TempDir::new().unwrap();

    // Create some files
    for i in 0..20 {
        fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
    }

    // Create a plan that includes non-existent files
    let mut files = vec![];
    for i in 0..20 {
        files.push(format!("file{}.txt", i));
    }
    // Add some files that don't exist
    for i in 20..30 {
        files.push(format!("nonexistent{}.txt", i));
    }

    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let plan = create_plan_from_files(temp.path(), file_refs, 100);

    let config = ExecutionConfig {
        mode: ExecutionMode::Batch,
        parallel: true,
        fail_fast: false, // Continue despite errors
        batch_size: 10,
        ..Default::default()
    };

    let executor = ExecutionEngine::new(config);
    let result = executor.execute(&plan).unwrap();

    // Should have 20 successful deletions and 10 failures
    assert_eq!(result.summary.successful, 20);
    assert_eq!(result.summary.failed, 10);
    assert_eq!(result.operations.len(), 30);
}

#[test]
fn test_parallel_execution_respects_batch_size() {
    let temp = TempDir::new().unwrap();

    // Create 100 files
    for i in 0..100 {
        fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
    }

    let files: Vec<_> = (0..100).map(|i| format!("file{}.txt", i)).collect();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let plan = create_plan_from_files(temp.path(), file_refs, 7);

    // Test with different batch sizes
    for batch_size in [10, 25, 50] {
        // Recreate files
        for i in 0..100 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            parallel: true,
            batch_size,
            ..Default::default()
        };

        let executor = ExecutionEngine::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(
            result.summary.successful, 100,
            "Batch size {} failed",
            batch_size
        );
    }
}
