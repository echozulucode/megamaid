//! Command orchestration and execution.

use crate::cli::Commands;
use crate::detector::{DetectionEngine, ScanContext, SizeThresholdRule};
use crate::executor::{
    ExecutionConfig, ExecutionEngine, ExecutionMode, TransactionLogger, TransactionOptions,
    TransactionStatus,
};
use crate::planner::{PlanGenerator, PlanWriter};
use crate::scanner::{FileScanner, ScanConfig};
use crate::verifier::{DriftReporter, VerificationConfig, VerificationEngine};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};

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
        Commands::Verify {
            plan,
            output,
            fail_fast,
            skip_mtime,
        } => run_verify(&plan, output, fail_fast, skip_mtime),
        Commands::Execute {
            plan,
            dry_run,
            interactive,
            backup_dir,
            recycle_bin,
            fail_fast,
            skip_verify,
            log_file,
        } => run_execute(
            &plan,
            dry_run,
            interactive,
            backup_dir,
            recycle_bin,
            fail_fast,
            skip_verify,
            log_file,
        ),
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
    // NOTE: Rule order matters! First match wins.
    // Build artifacts should be detected before size checks so they're always marked
    // for deletion (and their children filtered out), regardless of size.
    let mut engine = DetectionEngine::empty();
    engine.add_rule(Box::new(crate::detector::BuildArtifactRule::default()));
    engine.add_rule(Box::new(SizeThresholdRule {
        threshold_bytes: large_file_threshold * 1_048_576,
    }));

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
        serde_yaml::from_str(&content).context("Failed to parse plan file")?;

    println!("📊 Cleanup Plan Statistics");
    println!();
    print_plan_summary(&plan);

    Ok(())
}

/// Executes the verify command.
fn run_verify(
    plan_path: &Path,
    output: Option<PathBuf>,
    fail_fast: bool,
    skip_mtime: bool,
) -> Result<()> {
    println!("📋 Verifying cleanup plan: {}", plan_path.display());
    println!();

    // Read plan file
    let content = fs::read_to_string(plan_path)
        .context(format!("Failed to read plan file: {}", plan_path.display()))?;

    // Deserialize
    let plan: crate::models::CleanupPlan =
        serde_yaml::from_str(&content).context("Failed to parse plan file")?;

    // Configure verification
    let config = VerificationConfig {
        check_mtime: !skip_mtime,
        check_size: true,
        fail_fast,
    };

    // Run verification
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Verifying entries...");

    let verifier = VerificationEngine::new(config);
    let result = verifier.verify(&plan)?;

    spinner.finish_with_message(format!(
        "✓ Verified {} of {} entries",
        result.verified, result.total_entries
    ));
    println!();

    // Print report
    let report = DriftReporter::generate_report(&result);
    println!("{}", report);

    // Write report file if requested
    if let Some(output_path) = output {
        DriftReporter::write_report(&result, &output_path)?;
        println!("📄 Drift report written to: {}", output_path.display());
        println!();
    }

    // Exit with error if drift detected
    if !result.is_safe_to_execute() {
        anyhow::bail!("Drift detected - plan is not safe to execute");
    }

    Ok(())
}

/// Executes the execute command.
fn run_execute(
    plan_path: &Path,
    dry_run: bool,
    interactive: bool,
    backup_dir: Option<PathBuf>,
    recycle_bin: bool,
    fail_fast: bool,
    skip_verify: bool,
    log_file: PathBuf,
) -> Result<()> {
    println!("🗑️  Executing cleanup plan: {}", plan_path.display());
    println!();

    // Read plan file
    let content = fs::read_to_string(plan_path)
        .context(format!("Failed to read plan file: {}", plan_path.display()))?;

    // Deserialize
    let plan: crate::models::CleanupPlan =
        serde_yaml::from_str(&content).context("Failed to parse plan file")?;

    // Verify unless skipped
    if !skip_verify && !dry_run {
        println!("🔍 Verifying plan before execution...");
        let verifier = VerificationEngine::new(VerificationConfig::default());
        let verification = verifier.verify(&plan)?;

        if !verification.is_safe_to_execute() {
            let report = DriftReporter::generate_report(&verification);
            println!("{}", report);
            anyhow::bail!(
                "Drift detected - cannot execute. Use --skip-verify to override (not recommended)."
            );
        }
        println!("✓ Verification passed\n");
    }

    // Configure execution
    let mode = if dry_run {
        ExecutionMode::DryRun
    } else if interactive {
        ExecutionMode::Interactive
    } else {
        ExecutionMode::Batch
    };

    let config = ExecutionConfig {
        mode,
        backup_dir: backup_dir.clone(),
        fail_fast,
        use_recycle_bin: recycle_bin,
    };

    // Display mode
    if dry_run {
        println!("🔄 DRY RUN MODE - No files will be deleted");
        println!();
    } else if interactive {
        println!("💬 INTERACTIVE MODE - You will be prompted for each deletion");
        println!();
    } else if let Some(ref backup_path) = backup_dir {
        println!("📦 BACKUP MODE - Files will be moved to: {}", backup_path.display());
        println!();
    } else if recycle_bin {
        println!("♻️  RECYCLE BIN MODE - Files will be moved to recycle bin");
        println!();
    }

    // Count Delete actions
    let delete_count = plan
        .entries
        .iter()
        .filter(|e| e.action == crate::models::CleanupAction::Delete)
        .count();

    if delete_count == 0 {
        println!("No entries marked for deletion.");
        return Ok(());
    }

    println!("Processing {} deletion(s)...", delete_count);
    println!();

    // Create transaction logger
    let options = TransactionOptions {
        dry_run,
        backup_dir: backup_dir.clone(),
        use_recycle_bin: recycle_bin,
        fail_fast,
    };
    let mut logger = TransactionLogger::new(plan_path, log_file.clone(), options);

    println!("📋 Transaction ID: {}", logger.execution_id());
    println!();

    // Execute
    let executor = ExecutionEngine::new(config);
    let progress = ProgressBar::new(delete_count as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let result = executor.execute(&plan)?;

    // Log all operations
    for op in &result.operations {
        logger.log_operation(op);
        progress.inc(1);
        if op.status == crate::executor::OperationStatus::Failed {
            progress.set_message(format!("Failed: {}", op.path.display()));
        }
    }

    progress.finish_with_message("Done");
    println!();

    // Finalize transaction log
    let status = if result.summary.failed > 0 {
        TransactionStatus::Failed
    } else {
        TransactionStatus::Completed
    };
    logger.finalize(&result, status)?;

    // Print summary
    print_execution_summary(&result.summary, dry_run);
    println!();
    println!("📄 Transaction log: {}", log_file.display());

    // Exit with error if any failures
    if result.summary.failed > 0 {
        anyhow::bail!("{} operation(s) failed", result.summary.failed);
    }

    Ok(())
}

fn print_execution_summary(summary: &crate::executor::ExecutionSummary, dry_run: bool) {
    println!("Summary:");
    println!("  Total operations: {}", summary.total_operations);
    println!("  Successful: {}", summary.successful);
    println!("  Failed: {}", summary.failed);
    println!("  Skipped: {}", summary.skipped);
    println!(
        "  Space freed: {:.2} GB",
        summary.space_freed as f64 / 1_073_741_824.0
    );
    println!("  Duration: {:.2}s", summary.duration.as_secs_f64());

    if dry_run {
        println!();
        println!("This was a dry run. No files were actually deleted.");
    }
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

        let output_path = temp.path().join("plan.yaml");

        let result = run_scan(temp.path(), &output_path, None, true, 100);

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_run_scan_nonexistent_path() {
        let output = PathBuf::from("plan.yaml");
        let result = run_scan(Path::new("/nonexistent/path"), &output, None, true, 100);

        assert!(result.is_err());
    }

    #[test]
    fn test_run_stats_with_valid_plan() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.yaml");

        // Create a minimal valid plan
        let plan_content = r#"
version: "0.1.0"
created_at: "2025-11-19T12:00:00Z"
base_path: "/test"
entries:
  - path: "test.txt"
    size: 1000
    modified: "2025-11-19T12:00:00Z"
    action: delete
    rule_name: test
    reason: Test
"#;

        fs::write(&plan_path, plan_content).unwrap();

        let result = run_stats(&plan_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_stats_invalid_yaml() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.yaml");

        fs::write(&plan_path, "invalid: yaml: content: [[[").unwrap();

        let result = run_stats(&plan_path);
        assert!(result.is_err());
    }
}
