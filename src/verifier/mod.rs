//! Plan verification and drift detection.
//!
//! This module provides functionality to verify cleanup plans against the current
//! filesystem state, detecting any changes (drift) that have occurred since the
//! plan was created.

pub mod engine;
pub mod report;

pub use engine::{
    DriftDetection, DriftType, VerificationConfig, VerificationEngine, VerificationError,
    VerificationResult,
};
pub use report::DriftReporter;
