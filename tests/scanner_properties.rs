use megamaid::models::cleanup_plan::{CleanupAction, CleanupEntry, CleanupPlan};
use megamaid::planner::writer::PlanWriter;
use megamaid::scanner::traversal::{FileScanner, ScanConfig};
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a nested directory structure with given depth and width
fn create_nested_structure(base: &TempDir, depth: usize, width: usize) {
    fn create_level(
        path: &std::path::Path,
        current_depth: usize,
        target_depth: usize,
        width: usize,
    ) {
        if current_depth >= target_depth {
            return;
        }

        for i in 0..width {
            let dir_path = path.join(format!("dir_{}_{}", current_depth, i));
            fs::create_dir_all(&dir_path).unwrap();

            // Create a file in each directory
            let file_path = dir_path.join(format!("file_{}.txt", i));
            fs::write(file_path, "test content").unwrap();

            // Recurse to next level
            create_level(&dir_path, current_depth + 1, target_depth, width);
        }
    }

    create_level(base.path(), 0, depth, width);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_scanner_handles_arbitrary_paths(
        depth in 1usize..6,  // Limit depth to avoid test timeout
        width in 1usize..8,  // Limit width to avoid excessive file creation
    ) {
        let temp = TempDir::new().unwrap();
        create_nested_structure(&temp, depth, width);

        let scanner = FileScanner::new(ScanConfig::default());
        let result = scanner.scan(temp.path());

        // Should never panic or error on valid directory structure
        prop_assert!(result.is_ok());

        // Verify we got some results
        let entries = result.unwrap();
        prop_assert!(!entries.is_empty());
    }

    #[test]
    fn test_scanner_max_depth_respected(
        max_depth in 1usize..5,
        structure_depth in 5usize..10,
    ) {
        let temp = TempDir::new().unwrap();
        create_nested_structure(&temp, structure_depth, 2);

        let config = ScanConfig {
            max_depth: Some(max_depth),
            ..Default::default()
        };
        let scanner = FileScanner::new(config);
        let result = scanner.scan(temp.path()).unwrap();

        // All entries should be within max_depth from base
        for entry in result {
            let relative = entry.path.strip_prefix(temp.path()).unwrap();
            let depth = relative.components().count();
            prop_assert!(depth <= max_depth + 1); // +1 because we count files too
        }
    }

    #[test]
    fn test_plan_serialization_preserves_data(
        size in 0u64..10_000_000_000u64,  // Up to 10GB
        path in "[a-zA-Z0-9/_-]{1,50}",
    ) {
        use chrono::Utc;

        let entry = CleanupEntry {
            path: path.clone(),
            size,
            modified: Utc::now().to_rfc3339(),
            action: CleanupAction::Delete,
            rule_name: "test_rule".to_string(),
            reason: "test".to_string(),
        };

        let plan = CleanupPlan {
            version: "1.0".to_string(),
            created_at: Utc::now(),
            base_path: PathBuf::from("/test"),
            entries: vec![entry],
        };

        // Serialize to YAML
        let yaml_string = serde_yaml::to_string(&plan).unwrap();

        // Deserialize back
        let roundtrip: CleanupPlan = serde_yaml::from_str(&yaml_string).unwrap();

        // Verify data preservation
        prop_assert_eq!(&plan.entries[0].path, &roundtrip.entries[0].path);
        prop_assert_eq!(plan.entries[0].size, roundtrip.entries[0].size);
        prop_assert_eq!(plan.entries.len(), roundtrip.entries.len());
    }

    #[test]
    fn test_plan_write_read_roundtrip(
        sizes in prop::collection::vec(0u64..1_000_000u64, 1..20),
    ) {
        use chrono::Utc;

        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("test_plan.yaml");

        // Create plan with multiple entries
        let entries: Vec<CleanupEntry> = sizes
            .iter()
            .enumerate()
            .map(|(i, &size)| CleanupEntry {
                path: format!("file_{}.txt", i),
                size,
                modified: Utc::now().to_rfc3339(),
                action: CleanupAction::Delete,
                rule_name: "test_rule".to_string(),
                reason: "test".to_string(),
            })
            .collect();

        let original_plan = CleanupPlan {
            version: "1.0".to_string(),
            created_at: Utc::now(),
            base_path: temp.path().to_path_buf(),
            entries,
        };

        // Write plan
        PlanWriter::write(&original_plan, &plan_path).unwrap();

        // Read back
        let content = std::fs::read_to_string(&plan_path).unwrap();
        let loaded_plan: CleanupPlan = serde_yaml::from_str(&content).unwrap();

        // Verify
        prop_assert_eq!(original_plan.entries.len(), loaded_plan.entries.len());
        prop_assert_eq!(original_plan.version, loaded_plan.version);

        for (original, loaded) in original_plan.entries.iter().zip(loaded_plan.entries.iter()) {
            prop_assert_eq!(&original.path, &loaded.path);
            prop_assert_eq!(original.size, loaded.size);
        }
    }

    #[test]
    fn test_scanner_skip_hidden_consistent(
        skip_hidden in prop::bool::ANY,
        num_hidden in 0usize..5,
        num_visible in 1usize..5,
    ) {
        let temp = TempDir::new().unwrap();

        // Create hidden files
        for i in 0..num_hidden {
            fs::write(temp.path().join(format!(".hidden_{}.txt", i)), "secret").unwrap();
        }

        // Create visible files
        for i in 0..num_visible {
            fs::write(temp.path().join(format!("visible_{}.txt", i)), "public").unwrap();
        }

        let config = ScanConfig {
            skip_hidden,
            ..Default::default()
        };
        let scanner = FileScanner::new(config);
        let results = scanner.scan(temp.path()).unwrap();

        // Check if hidden files were found
        let has_hidden = results.iter().any(|e| {
            e.path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        });

        if skip_hidden {
            // Should not find any hidden files
            prop_assert!(!has_hidden, "Found hidden files when skip_hidden=true");
        } else {
            // Should find some hidden files if they exist
            if num_hidden > 0 {
                prop_assert!(has_hidden, "Did not find hidden files when skip_hidden=false and {} hidden files exist", num_hidden);
            }
        }
    }
}
