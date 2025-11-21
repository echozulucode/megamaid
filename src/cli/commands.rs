//! Command-line argument definitions.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Megamaid - Storage cleanup analysis tool
#[derive(Parser, Debug)]
#[command(name = "megamaid")]
#[command(about = "Analyzes directories for cleanup candidates and generates actionable plans", long_about = None)]
#[command(version)]
pub struct Cli {
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
}
