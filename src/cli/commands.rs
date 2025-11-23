//! Command-line argument definitions.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Megamaid - Storage cleanup analysis tool
#[derive(Parser, Debug)]
#[command(name = "megamaid")]
#[command(about = "Analyzes directories for cleanup candidates and generates actionable plans", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Configuration file path (overrides defaults)
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan a directory and generate a cleanup plan
    Scan {
        /// Directory to scan
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output plan file path
        #[arg(short, long, value_name = "FILE", default_value = "cleanup-plan.yaml")]
        output: PathBuf,

        /// Maximum directory depth to scan
        #[arg(short = 'd', long)]
        max_depth: Option<usize>,

        /// Skip hidden files and directories
        #[arg(long, default_value_t = true)]
        skip_hidden: bool,

        /// Minimum file size in MB to flag as large
        #[arg(long, default_value_t = 100)]
        large_file_threshold: u64,
    },

    /// Display statistics about a cleanup plan
    Stats {
        /// Path to cleanup plan file
        #[arg(value_name = "FILE")]
        plan: PathBuf,
    },

    /// Verify a cleanup plan against current filesystem state
    Verify {
        /// Path to cleanup plan file
        #[arg(value_name = "FILE")]
        plan: PathBuf,

        /// Output drift report to file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Fail fast on first drift detection
        #[arg(long)]
        fail_fast: bool,

        /// Skip modification time checks
        #[arg(long)]
        skip_mtime: bool,
    },

    /// Execute a cleanup plan
    Execute {
        /// Path to cleanup plan file
        #[arg(value_name = "FILE")]
        plan: PathBuf,

        /// Dry-run mode (simulate without deleting)
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode (prompt for each deletion)
        #[arg(short, long)]
        interactive: bool,

        /// Backup directory (move instead of delete)
        #[arg(long, value_name = "DIR")]
        backup_dir: Option<PathBuf>,

        /// Use system recycle bin
        #[arg(long)]
        recycle_bin: bool,

        /// Stop on first error
        #[arg(long)]
        fail_fast: bool,

        /// Skip verification before execution
        #[arg(long)]
        skip_verify: bool,

        /// Transaction log file path
        #[arg(long, value_name = "FILE", default_value = "execution-log.yaml")]
        log_file: PathBuf,

        /// Enable parallel execution (not compatible with interactive mode)
        #[arg(long)]
        parallel: bool,

        /// Batch size for parallel processing
        #[arg(long, default_value = "100")]
        batch_size: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_scan_command() {
        let args = vec!["megamaid", "scan", "/test/path"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Scan { path, output, .. } => {
                assert_eq!(path, PathBuf::from("/test/path"));
                assert_eq!(output, PathBuf::from("cleanup-plan.yaml"));
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_cli_parsing_scan_with_options() {
        let args = vec![
            "megamaid",
            "scan",
            "/test",
            "--output",
            "my-plan.yaml",
            "--max-depth",
            "5",
            "--large-file-threshold",
            "200",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Scan {
                path,
                output,
                max_depth,
                large_file_threshold,
                ..
            } => {
                assert_eq!(path, PathBuf::from("/test"));
                assert_eq!(output, PathBuf::from("my-plan.yaml"));
                assert_eq!(max_depth, Some(5));
                assert_eq!(large_file_threshold, 200);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_cli_parsing_stats_command() {
        let args = vec!["megamaid", "stats", "plan.yaml"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Stats { plan } => {
                assert_eq!(plan, PathBuf::from("plan.yaml"));
            }
            _ => panic!("Expected Stats command"),
        }
    }

    #[test]
    fn test_default_values() {
        let args = vec!["megamaid", "scan", "/test"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Scan {
                skip_hidden,
                large_file_threshold,
                max_depth,
                ..
            } => {
                assert!(skip_hidden);
                assert_eq!(large_file_threshold, 100);
                assert_eq!(max_depth, None);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_cli_parsing_verify_command() {
        let args = vec!["megamaid", "verify", "plan.yaml"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Verify {
                plan,
                output,
                fail_fast,
                skip_mtime,
            } => {
                assert_eq!(plan, PathBuf::from("plan.yaml"));
                assert_eq!(output, None);
                assert!(!fail_fast);
                assert!(!skip_mtime);
            }
            _ => panic!("Expected Verify command"),
        }
    }

    #[test]
    fn test_cli_parsing_verify_with_options() {
        let args = vec![
            "megamaid",
            "verify",
            "plan.yaml",
            "--output",
            "drift-report.txt",
            "--fail-fast",
            "--skip-mtime",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Verify {
                plan,
                output,
                fail_fast,
                skip_mtime,
            } => {
                assert_eq!(plan, PathBuf::from("plan.yaml"));
                assert_eq!(output, Some(PathBuf::from("drift-report.txt")));
                assert!(fail_fast);
                assert!(skip_mtime);
            }
            _ => panic!("Expected Verify command"),
        }
    }

    #[test]
    fn test_cli_parsing_execute_command() {
        let args = vec!["megamaid", "execute", "plan.yaml"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Execute {
                plan,
                dry_run,
                interactive,
                backup_dir,
                recycle_bin,
                fail_fast,
                skip_verify,
                log_file,
                parallel,
                batch_size,
            } => {
                assert_eq!(plan, PathBuf::from("plan.yaml"));
                assert!(!dry_run);
                assert!(!interactive);
                assert_eq!(backup_dir, None);
                assert!(!recycle_bin);
                assert!(!fail_fast);
                assert!(!skip_verify);
                assert_eq!(log_file, PathBuf::from("execution-log.yaml"));
                assert!(!parallel);
                assert_eq!(batch_size, 100);
            }
            _ => panic!("Expected Execute command"),
        }
    }

    #[test]
    fn test_cli_parsing_execute_with_options() {
        let args = vec![
            "megamaid",
            "execute",
            "plan.yaml",
            "--dry-run",
            "--fail-fast",
            "--skip-verify",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Execute {
                plan,
                dry_run,
                fail_fast,
                skip_verify,
                ..
            } => {
                assert_eq!(plan, PathBuf::from("plan.yaml"));
                assert!(dry_run);
                assert!(fail_fast);
                assert!(skip_verify);
            }
            _ => panic!("Expected Execute command"),
        }
    }

    #[test]
    fn test_cli_parsing_execute_with_backup() {
        let args = vec![
            "megamaid",
            "execute",
            "plan.yaml",
            "--backup-dir",
            "./backups",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Execute { backup_dir, .. } => {
                assert_eq!(backup_dir, Some(PathBuf::from("./backups")));
            }
            _ => panic!("Expected Execute command"),
        }
    }
}
