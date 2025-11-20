//! Integration tests for the file scanner.

use megamaid::scanner::{FileScanner, ScanConfig};
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

/// Creates a mock Rust project structure for testing.
fn create_mock_rust_project(temp: &TempDir) {
    let base = temp.path();

    // Create src directory
    fs::create_dir_all(base.join("src")).unwrap();
    fs::write(base.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(base.join("src/lib.rs"), "// lib").unwrap();

    // Create target directory (build artifacts)
    fs::create_dir_all(base.join("target/debug")).unwrap();
    fs::write(base.join("target/debug/myapp.exe"), "x".repeat(1000)).unwrap();

    // Create Cargo.toml
    fs::write(base.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    // Create tests directory
    fs::create_dir_all(base.join("tests")).unwrap();
    fs::write(base.join("tests/integration_test.rs"), "// test").unwrap();
}

#[test]
fn test_scan_realistic_project_structure() {
    let temp = TempDir::new().unwrap();
    create_mock_rust_project(&temp);

    let scanner = FileScanner::new(ScanConfig::default());
    let results = scanner.scan(temp.path()).unwrap();

    // Verify we found expected directories
    assert!(results.iter().any(|e| e.path.ends_with("src")));
    assert!(results.iter().any(|e| e.path.ends_with("target")));
    assert!(results.iter().any(|e| e.path.ends_with("Cargo.toml")));

    // Verify we found files in nested directories
    assert!(results.iter().any(|e| e.path.ends_with("main.rs")));
    assert!(results.iter().any(|e| e.path.ends_with("myapp.exe")));
}

#[test]
fn test_scan_large_file_count() {
    let temp = TempDir::new().unwrap();

    // Create 1000 small files
    for i in 0..1000 {
        fs::write(temp.path().join(format!("file_{}.txt", i)), "x").unwrap();
    }

    let start = Instant::now();
    let scanner = FileScanner::new(ScanConfig::default());
    let results = scanner.scan(temp.path()).unwrap();
    let duration = start.elapsed();

    // Should find all 1000 files plus the root directory
    assert_eq!(results.len(), 1001);
    assert!(
        duration.as_secs() < 2,
        "Should scan 1K files in <2s, took {:?}",
        duration
    );
}

#[test]
fn test_scan_with_subdirectories() {
    let temp = TempDir::new().unwrap();

    // Create a structure with multiple levels
    for i in 0..10 {
        let dir = temp.path().join(format!("dir_{}", i));
        fs::create_dir_all(&dir).unwrap();

        for j in 0..10 {
            fs::write(dir.join(format!("file_{}.txt", j)), "content").unwrap();
        }
    }

    let scanner = FileScanner::new(ScanConfig::default());
    let results = scanner.scan(temp.path()).unwrap();

    // Should find: 1 root + 10 dirs + 100 files = 111 entries
    assert_eq!(results.len(), 111);

    // Count files vs directories
    let file_count = results.iter().filter(|e| e.is_file()).count();
    let dir_count = results.iter().filter(|e| e.is_directory()).count();

    assert_eq!(file_count, 100);
    assert_eq!(dir_count, 11); // root + 10 subdirs
}

#[test]
fn test_scan_respects_max_depth() {
    let temp = TempDir::new().unwrap();

    // Create deep nesting: a/b/c/d/e
    // Note: max_depth counts from root, so depth 1 = root, depth 2 = a, depth 3 = a/b, etc.
    fs::create_dir_all(temp.path().join("a/b/c/d/e")).unwrap();
    fs::write(temp.path().join("a/file_1.txt"), "1").unwrap();
    fs::write(temp.path().join("a/b/file_2.txt"), "2").unwrap();
    fs::write(temp.path().join("a/b/c/file_3.txt"), "3").unwrap();
    fs::write(temp.path().join("a/b/c/d/file_4.txt"), "4").unwrap();
    fs::write(temp.path().join("a/b/c/d/e/file_5.txt"), "5").unwrap();

    let config = ScanConfig {
        max_depth: Some(4), // root=0, a=1, a/b=2, a/b/c=3, a/b/c/file_3.txt=4
        ..Default::default()
    };
    let scanner = FileScanner::new(config);
    let results = scanner.scan(temp.path()).unwrap();

    // Should find files up to depth 4
    assert!(results.iter().any(|e| e.path.ends_with("file_1.txt")));
    assert!(results.iter().any(|e| e.path.ends_with("file_2.txt")));
    assert!(results.iter().any(|e| e.path.ends_with("file_3.txt")));

    // Should NOT find files at depth 5 and beyond
    assert!(!results.iter().any(|e| e.path.ends_with("file_4.txt")));
    assert!(!results.iter().any(|e| e.path.ends_with("file_5.txt")));
}

#[test]
fn test_scan_skip_hidden_integration() {
    let temp = TempDir::new().unwrap();

    // Create visible and hidden files
    fs::write(temp.path().join("visible.txt"), "public").unwrap();
    fs::write(temp.path().join(".hidden"), "secret").unwrap();

    // Create hidden directory with contents
    fs::create_dir_all(temp.path().join(".hidden_dir")).unwrap();
    fs::write(temp.path().join(".hidden_dir/secret.txt"), "data").unwrap();

    let config = ScanConfig {
        skip_hidden: true,
        ..Default::default()
    };
    let scanner = FileScanner::new(config);
    let results = scanner.scan(temp.path()).unwrap();

    // Should find visible.txt
    assert!(results.iter().any(|e| e.path.ends_with("visible.txt")));

    // Should NOT find hidden files or directories
    assert!(!results.iter().any(|e| {
        e.path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.') && n != ".")
            .unwrap_or(false)
    }));
}
