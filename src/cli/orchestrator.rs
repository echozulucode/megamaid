//! Command orchestration and execution.

use crate::cli::Commands;
use crate::detector::{DetectionEngine, ScanContext, SizeThresholdRule};
use crate::planner::{PlanGenerator, PlanWriter};
use crate::scanner::{FileScanner, ScanConfig};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

/// Runs the specified command.
pub fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Scan {
            path,
            output,
            max_depth,
            skip_hidden,
            large_file_threshold,
        } => run_scan(&path, &output, max_depth, skip_hidden, large_file_threshold),
        Commands::Stats { plan } => run_stats(&plan),
    }
}

/// Executes the scan command.
fn run_scan(
    path: &Path,
    output: &Path,
    max_depth: Option<usize>,
    skip_hidden: bool,
    large_file_threshold: u64,
) -> Result<()> {
    // Validate input path
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    println!("🔍 Scanning directory: {}", path.display());
    println!();

    // Configure scanner
    let config = ScanConfig {
        follow_links: false,
        max_depth,
        skip_hidden,
    };

    // Create progress bar
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Scanning filesystem...");

    // Scan the directory
    let scanner = FileScanner::new(config);
    let entries = scanner.scan(path).context("Failed to scan directory")?;

    spinner.finish_with_message(format!("✓ Scanned {} entries", entries.len()));
    println!();

    // Configure detection engine
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: large_file_threshold * 1_048_576,
    }));
    engine.add_rule(Box::new(crate::detector::BuildArtifactRule::default()));

    // Run detection
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Analyzing for cleanup candidates...");

    let context = ScanContext::default();
    let detections = engine.analyze(&entries, &context);

    spinner.finish_with_message(format!("✓ Found {} cleanup candidates", detections.len()));
    println!();

    // Generate plan
    let generator = PlanGenerator::new(path.to_path_buf());
    let plan = generator.generate(detections);

    // Write plan
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Writing plan to {}...", output.display()));

    PlanWriter::write(&plan, output).context("Failed to write cleanup plan")?;

    spinner.finish_with_message(format!("✓ Plan written to {}", output.display()));
    println!();

    // Print summary
    print_plan_summary(&plan);

    Ok(())
}

/// Executes the stats command.
fn run_stats(plan_path: &Path) -> Result<()> {
    // Read plan file
    let content = fs::read_to_string(plan_path)
        .context(format!("Failed to read plan file: {}", plan_path.display()))?;

    // Deserialize
    let plan: crate::models::CleanupPlan =
        toml::from_str(&content).context("Failed to parse plan file")?;

    println!("📊 Cleanup Plan Statistics");
    println!();
    print_plan_summary(&plan);

    Ok(())
}

/// Prints a summary of the cleanup plan.
fn print_plan_summary(plan: &crate::models::CleanupPlan) {
    println!("Base Path: {}", plan.base_path.display());
    println!("Version:   {}", plan.version);
    println!("Created:   {}", plan.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!();
    println!("Entries:   {}", plan.entries.len());
    println!("  • Delete: {}", plan.delete_count());
    println!("  • Review: {}", plan.review_count());
    println!("  • Keep:   {}", plan.keep_count());
    println!();
    println!("Total Size: {} MB", plan.total_size() / 1_048_576);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_run_scan_with_temp_dir() {
        let temp = TempDir::new().unwrap();

        // Create some test files
        fs::write(temp.path().join("test.txt"), "hello").unwrap();
        fs::create_dir_all(temp.path().join("target")).unwrap();

        let output_path = temp.path().join("plan.toml");

        let result = run_scan(temp.path(), &output_path, None, true, 100);

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_run_scan_nonexistent_path() {
        let output = PathBuf::from("plan.toml");
        let result = run_scan(Path::new("/nonexistent/path"), &output, None, true, 100);

        assert!(result.is_err());
    }

    #[test]
    fn test_run_stats_with_valid_plan() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.toml");

        // Create a minimal valid plan
        let plan_content = r#"
version = "0.1.0"
created_at = "2025-11-19T12:00:00Z"
base_path = "/test"

[[entries]]
path = "test.txt"
size = 1000
modified = "2025-11-19T12:00:00Z"
action = "delete"
rule_name = "test"
reason = "Test"
"#;

        fs::write(&plan_path, plan_content).unwrap();

        let result = run_stats(&plan_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_stats_invalid_toml() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.toml");

        fs::write(&plan_path, "invalid toml content [[[").unwrap();

        let result = run_stats(&plan_path);
        assert!(result.is_err());
    }
}
