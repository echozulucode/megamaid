//! Cleanup candidate detection rules and engine.

pub mod engine;
pub mod rules;

pub use engine::{DetectionEngine, DetectionResult, ScanContext};
pub use rules::{BuildArtifactRule, DetectionRule, SizeThresholdRule};
