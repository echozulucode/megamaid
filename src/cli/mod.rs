//! Command-line interface and orchestration.

pub mod commands;
pub mod orchestrator;

pub use commands::{Cli, Commands};
pub use orchestrator::run_command;
