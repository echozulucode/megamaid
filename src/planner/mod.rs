//! Plan generation and serialization.

pub mod generator;
pub mod writer;

pub use generator::PlanGenerator;
pub use writer::{PlanWriter, WriteError};
