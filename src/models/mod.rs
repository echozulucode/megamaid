//! Core data models for file system entries and cleanup plans.

pub mod cleanup_plan;
pub mod file_entry;

pub use cleanup_plan::{CleanupAction, CleanupEntry, CleanupPlan};
pub use file_entry::{EntryType, FileEntry};
