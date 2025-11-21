//! Drift report generation for human-readable output.

use crate::verifier::engine::{DriftType, VerificationResult};
use std::io::Write;
use std::path::Path;

/// Generator for human-readable drift reports.
pub struct DriftReporter;

impl DriftReporter {
    /// Generate a human-readable drift report from verification results.
    pub fn generate_report(result: &VerificationResult) -> String {
        let mut report = String::new();

        report.push_str("# Plan Verification Report\n\n");

        // Summary
        report.push_str(&format!("Total entries: {}\n", result.total_entries));
        report.push_str(&format!("Verified: {}\n", result.verified));
        report.push_str(&format!("Drifted: {}\n", result.drifted.len()));
        report.push_str(&format!("Missing: {}\n", result.missing.len()));
        report.push_str(&format!(
            "Permission errors: {}\n\n",
            result.permission_errors.len()
        ));

        if result.is_safe_to_execute() {
            report.push_str("✅ SAFE TO EXECUTE\n\n");
        } else {
            report.push_str("⚠️  DRIFT DETECTED - NOT SAFE TO EXECUTE\n\n");
        }

        // Missing files
        if !result.missing.is_empty() {
            report.push_str("## Missing Files\n\n");
            report.push_str("The following files were in the plan but no longer exist:\n\n");
            for path in &result.missing {
                report.push_str(&format!("- {}\n", path.display()));
            }
            report.push('\n');
        }

        // Drifted files
        if !result.drifted.is_empty() {
            report.push_str("## Drifted Files\n\n");
            report.push_str("The following files have changed since the plan was created:\n\n");
            for drift in &result.drifted {
                report.push_str(&format!("### {}\n", drift.path.display()));
                let drift_type_str = match drift.drift_type {
                    DriftType::SizeMismatch => "Size Mismatch",
                    DriftType::ModificationTimeMismatch => "Modification Time Mismatch",
                };
                report.push_str(&format!("Type: {}\n", drift_type_str));
                report.push_str(&format!("Expected: {}\n", drift.expected));
                report.push_str(&format!("Actual: {}\n\n", drift.actual));
            }
        }

        // Permission errors (warnings)
        if !result.permission_errors.is_empty() {
            report.push_str("## Permission Warnings\n\n");
            report
                .push_str("The following files could not be verified due to permission errors.\n");
            report.push_str("These are warnings only and will not block execution:\n\n");
            for path in &result.permission_errors {
                report.push_str(&format!("- {}\n", path.display()));
            }
            report.push('\n');
        }

        // Recommendations
        if !result.is_safe_to_execute() {
            report.push_str("## Recommendations\n\n");
            report.push_str("The plan cannot be safely executed due to detected drift.\n");
            report.push_str("Consider one of the following actions:\n\n");
            report.push_str("1. Re-scan the directory to generate a fresh plan\n");
            report.push_str("2. Manually review the changes and update the plan file\n");
            report.push_str(
                "3. If changes are expected, use --skip-verify flag (not recommended)\n\n",
            );
        }

        report
    }

    /// Write a drift report to a file.
    pub fn write_report(result: &VerificationResult, path: &Path) -> std::io::Result<()> {
        let report = Self::generate_report(result);
        let mut file = std::fs::File::create(path)?;
        file.write_all(report.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verifier::engine::{DriftDetection, DriftType};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_clean_result() -> VerificationResult {
        VerificationResult {
            total_entries: 10,
            verified: 10,
            drifted: Vec::new(),
            missing: Vec::new(),
            permission_errors: Vec::new(),
        }
    }

    fn create_drifted_result() -> VerificationResult {
        VerificationResult {
            total_entries: 10,
            verified: 7,
            drifted: vec![
                DriftDetection {
                    path: PathBuf::from("/test/file1.txt"),
                    drift_type: DriftType::SizeMismatch,
                    expected: "1000 bytes".to_string(),
                    actual: "2000 bytes".to_string(),
                },
                DriftDetection {
                    path: PathBuf::from("/test/file2.txt"),
                    drift_type: DriftType::ModificationTimeMismatch,
                    expected: "2025-11-21T10:00:00Z".to_string(),
                    actual: "2025-11-21T11:00:00Z".to_string(),
                },
            ],
            missing: vec![PathBuf::from("/test/missing.txt")],
            permission_errors: vec![PathBuf::from("/test/locked.txt")],
        }
    }

    #[test]
    fn test_generate_clean_report() {
        let result = create_clean_result();
        let report = DriftReporter::generate_report(&result);

        assert!(report.contains("SAFE TO EXECUTE"));
        assert!(report.contains("Verified: 10"));
        assert!(report.contains("Drifted: 0"));
        assert!(report.contains("Missing: 0"));
    }

    #[test]
    fn test_generate_drifted_report() {
        let result = create_drifted_result();
        let report = DriftReporter::generate_report(&result);

        assert!(report.contains("DRIFT DETECTED"));
        assert!(report.contains("NOT SAFE TO EXECUTE"));
        assert!(report.contains("Drifted: 2"));
        assert!(report.contains("Missing: 1"));
        assert!(report.contains("Size Mismatch"));
        assert!(report.contains("Modification Time Mismatch"));
    }

    #[test]
    fn test_report_includes_missing_files() {
        let result = create_drifted_result();
        let report = DriftReporter::generate_report(&result);

        assert!(report.contains("Missing Files"));
        assert!(report.contains("missing.txt"));
    }

    #[test]
    fn test_report_includes_permission_warnings() {
        let result = create_drifted_result();
        let report = DriftReporter::generate_report(&result);

        assert!(report.contains("Permission Warnings"));
        assert!(report.contains("locked.txt"));
        assert!(report.contains("warnings only"));
    }

    #[test]
    fn test_report_includes_recommendations() {
        let result = create_drifted_result();
        let report = DriftReporter::generate_report(&result);

        assert!(report.contains("Recommendations"));
        assert!(report.contains("Re-scan"));
    }

    #[test]
    fn test_write_report_to_file() {
        let temp = TempDir::new().unwrap();
        let report_path = temp.path().join("drift-report.txt");

        let result = create_drifted_result();
        DriftReporter::write_report(&result, &report_path).unwrap();

        assert!(report_path.exists());

        let content = std::fs::read_to_string(&report_path).unwrap();
        assert!(content.contains("DRIFT DETECTED"));
    }

    #[test]
    fn test_clean_report_no_recommendations() {
        let result = create_clean_result();
        let report = DriftReporter::generate_report(&result);

        assert!(!report.contains("Recommendations"));
    }
}
