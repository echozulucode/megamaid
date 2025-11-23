//! Configuration management for Megamaid.
//!
//! This module provides configuration file loading, validation, and management.
//! Configuration files use YAML format and allow customization of scanner, detector,
//! executor, and output settings.
//!
//! # Example
//!
//! ```rust
//! use megamaid::config::{load_config, MegamaidConfig};
//!
//! # fn example() -> anyhow::Result<()> {
//! // Load from file
//! // let config = load_config("megamaid.yaml")?;
//!
//! // Or use defaults
//! let config = MegamaidConfig::default();
//!
//! assert_eq!(config.scanner.thread_count, 0); // Auto-detect
//! assert_eq!(config.executor.batch_size, 100);
//! # Ok(())
//! # }
//! ```

pub mod loader;
pub mod schema;
pub mod validation;

// Re-export commonly used types
pub use loader::{load_config, load_default_config, parse_config, write_config};
pub use schema::{
    BuildArtifactsConfig, BuiltInRulesConfig, CustomRule, DetectorConfig, ExecutionModeConfig,
    ExecutorConfig, MegamaidConfig, OutputConfig, ScannerConfig, SizeThresholdConfig,
    VerifierConfig,
};
pub use validation::validate_config;
