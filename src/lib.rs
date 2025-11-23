//! # Megamaid Storage Cleanup Tool
//!
//! A high-performance storage analysis and cleanup utility for Windows 11 (and Linux).
//!
//! ## Overview
//!
//! Megamaid scans directories to identify cleanup candidates (large files, build artifacts,
//! etc.) and generates a human-editable YAML plan file. The plan can be reviewed and
//! modified before execution, ensuring safe cleanup operations.
//!
//! ## Architecture
//!
//! - **Scanner**: Traverses file system using `walkdir`, collecting metadata
//! - **Detector**: Applies configurable rules to identify cleanup candidates
//! - **Planner**: Generates human-editable YAML cleanup plans
//! - **Verifier**: Detects filesystem drift before plan execution
//! - **CLI**: Command-line interface with progress reporting
//!
//! ## Complete Workflow Example
//!
//! ```no_run
//! use megamaid::{FileScanner, ScanConfig, DetectionEngine, ScanContext, PlanGenerator, PlanWriter};
//! use std::path::{Path, PathBuf};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Step 1: Scan a directory
//! let scanner = FileScanner::new(ScanConfig::default());
//! let entries = scanner.scan(Path::new("/path/to/scan"))?;
//!
//! println!("Scanned {} entries", entries.len());
//!
//! // Step 2: Detect cleanup candidates
//! let engine = DetectionEngine::new(); // Uses default rules (size + build artifacts)
//! let detections = engine.analyze(&entries, &ScanContext::default());
//!
//! println!("Found {} cleanup candidates", detections.len());
//!
//! // Step 3: Generate cleanup plan
//! let generator = PlanGenerator::new(PathBuf::from("/path/to/scan"));
//! let plan = generator.generate(detections);
//!
//! // Step 4: Write plan to YAML file
//! let plan_path = Path::new("cleanup-plan.yaml");
//! PlanWriter::write(&plan, plan_path)?;
//!
//! println!("Plan written to {}", plan_path.display());
//! # Ok(())
//! # }
//! ```
//!
//! ## Basic Example: Creating a FileEntry
//!
//! ```
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
//!
//! assert_eq!(entry.size, 1024);
//! assert!(entry.is_file());
//! ```

/// Core data models
pub mod models;

/// File system scanning and traversal
pub mod scanner;

/// Detection rules and engine for identifying cleanup candidates
pub mod detector;

/// Plan generation and serialization
pub mod planner;

/// Plan verification and drift detection
pub mod verifier;

/// Plan execution and deletion operations
pub mod executor;

/// Configuration management
pub mod config;

/// Command-line interface
pub mod cli;

// Re-export commonly used types
pub use cli::{run_command, Cli, Commands};
pub use config::{
    load_config, load_default_config, parse_config, validate_config, write_config,
    MegamaidConfig,
};
pub use detector::{
    BuildArtifactRule, DetectionEngine, DetectionResult, DetectionRule, ScanContext,
    SizeThresholdRule,
};
pub use executor::{
    ExecutionConfig, ExecutionEngine, ExecutionError, ExecutionMode, ExecutionResult,
    ExecutionSummary, ExecutionSummaryLog, LoggedOperation, OperationAction, OperationResult,
    OperationStatus, TransactionLog, TransactionLogger, TransactionOptions, TransactionStatus,
};
pub use models::{CleanupAction, CleanupEntry, CleanupPlan, EntryType, FileEntry};
pub use planner::{PlanGenerator, PlanWriter, WriteError};
pub use scanner::{FileScanner, ProgressReport, ScanConfig, ScanError, ScanProgress};
pub use verifier::{
    DriftDetection, DriftReporter, DriftType, VerificationConfig, VerificationEngine,
    VerificationError, VerificationResult,
};
