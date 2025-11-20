//! # Megamaid Storage Cleanup Tool
//!
//! A high-performance storage analysis and cleanup utility for Windows 11 (and Linux).
//!
//! ## Overview
//!
//! Megamaid scans directories to identify cleanup candidates (large files, build artifacts,
//! etc.) and generates a human-editable TOML plan file. The plan can be reviewed and
//! modified before execution, ensuring safe cleanup operations.
//!
//! ## Architecture
//!
//! - **Scanner**: Traverses file system using `walkdir`, collecting metadata
//! - **Detector**: Applies configurable rules to identify cleanup candidates
//! - **Planner**: Generates human-editable TOML cleanup plans
//! - **CLI**: Command-line interface with progress reporting
//!
//! ## Example
//!
//! ```no_run
//! use megamaid::models::{FileEntry, EntryType};
//! use std::path::PathBuf;
//! use std::time::SystemTime;
//!
//! let entry = FileEntry::new(
//!     PathBuf::from("/test/file.txt"),
//!     1024,
//!     SystemTime::now(),
//!     EntryType::File,
//! );
//! ```

/// Core data models
pub mod models;

/// File system scanning and traversal
pub mod scanner;

/// Detection rules and engine for identifying cleanup candidates
pub mod detector;

/// Plan generation and serialization
pub mod planner;

/// Command-line interface
pub mod cli;

// Re-export commonly used types
pub use models::{CleanupAction, CleanupEntry, CleanupPlan, EntryType, FileEntry};
pub use scanner::{FileScanner, ProgressReport, ScanConfig, ScanError, ScanProgress};
pub use detector::{
    BuildArtifactRule, DetectionEngine, DetectionResult, DetectionRule, ScanContext,
    SizeThresholdRule,
};
pub use planner::{PlanGenerator, PlanWriter, WriteError};
pub use cli::{run_command, Cli, Commands};
