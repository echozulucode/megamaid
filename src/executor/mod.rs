//! Plan execution and deletion operations.
//!
//! This module provides functionality to safely execute cleanup plans with
//! multiple execution modes, backup support, and comprehensive error handling.

pub mod engine;

pub use engine::{
    ExecutionConfig, ExecutionEngine, ExecutionError, ExecutionMode, ExecutionResult,
    ExecutionSummary, OperationAction, OperationResult, OperationStatus,
};
